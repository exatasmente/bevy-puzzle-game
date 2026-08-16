//! The game's sound.
//!
//! Everything is driven off events the game already sends, rather than by
//! sprinkling `audio.play` through the systems that happen to notice something:
//! a pick sends `InteractionAnimationEvent` with `scored` on it, and a level up
//! sends a `BannerEvent`. Sound is a *reading* of what happened, so it belongs
//! downstream of the events that say what happened.
//!
//! Music is the exception, because it is not a reading of anything: it belongs
//! to the screen. `reconcile_music` keeps it matched to the state and the mute
//! setting.
//!
//! ## The browser
//!
//! Chrome will not start an `AudioContext` before a user gesture. It creates one
//! suspended, logs about it, and **leaves it there** — it does not resume on its
//! own when the player finally taps. Bevy builds its context at startup, from
//! cpal, and never calls `resume`, so without help the game fetches its sounds
//! and plays none of them. The help lives in `docs/index.html`, which wraps the
//! `AudioContext` constructor before loading the module and resumes the
//! instances on the first input; see the comment there for why the listener has
//! to be in the capture phase. Nothing in this file can reach that context.
//!
//! Sound is played by spawning an entity carrying `AudioPlayer` and
//! `PlaybackSettings`. `AudioSink` is a *component* that Bevy adds to that same
//! entity once the source has loaded — it is not an asset, so there is no handle
//! to keep alive and nothing to upgrade.

use bevy::audio::{AudioSinkPlayback, PlaybackMode};
use bevy::prelude::*;

use crate::events::InteractionAnimationEvent;
use crate::feedback::{BannerEvent, BannerKind};
use crate::storage;
use crate::AppState;

const VOLUME_KEY: &str = "color_puzzle.volume";

/// Both tracks sit under the effects on purpose: the music is there so silence
/// does not read as a broken build, and a hit still has to cut through it.
const THEME_VOLUME: f32 = 0.60;
const ROUND_VOLUME: f32 = 0.45;

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Volume>()
            .init_resource::<Music>()
            .add_systems(Startup, load_sounds)
            .add_systems(
                Update,
                (
                    play_pick_sounds,
                    play_level_sound,
                    reconcile_music,
                    follow_volume,
                ),
            );
    }
}

/// How loud the game is, in steps. Persisted, because a player who turned the
/// sound down once meant it.
///
/// Steps rather than a continuous slider: one button that cycles is both easier
/// to hit on a phone than a drag target and honest about how little precision
/// anyone wants here. Zero is off, which is
/// what the old mute toggle was — so the setting grew steps rather than gaining
/// a second control that could disagree with it.
#[derive(Resource)]
pub struct Volume(u8);

/// Off, and then four steps up to full.
pub const VOLUME_STEPS: u8 = 4;

impl Default for Volume {
    fn default() -> Self {
        Self(VOLUME_STEPS)
    }
}

impl Volume {
    /// The multiplier every sound in the game is scaled by.
    pub fn scale(&self) -> f32 {
        self.0 as f32 / VOLUME_STEPS as f32
    }

    pub fn is_silent(&self) -> bool {
        self.0 == 0
    }

    /// Steps down, wrapping back to full from off, so one button covers the
    /// whole range in either direction of travel the player imagines.
    pub fn cycle(&mut self) {
        self.0 = if self.0 == 0 { VOLUME_STEPS } else { self.0 - 1 };
        storage::save(VOLUME_KEY, &self.0.to_string());
    }

    /// The pause screen's label. ASCII only — the display font has no accents.
    pub fn label(&self) -> String {
        if self.is_silent() {
            "SOM: DESLIGADO".to_string()
        } else {
            format!("SOM: {}%", (self.scale() * 100.0).round() as u32)
        }
    }

    /// Reads the saved level, and **deliberately ignores the old
    /// `color_puzzle.muted` flag** this setting replaced.
    ///
    /// Carrying that flag over was the obvious courtesy and it was wrong here.
    /// The old control was a two-state toggle that cost nothing to tap out of
    /// curiosity, so plenty of browsers have `muted=1` sitting in them from a
    /// single idle press. Honouring it starts the game in silence — and while
    /// the pause screen does not repaint in the browser, the reading on the
    /// button never changes either, so there is no way to find out why. The
    /// game just appears to have no sound. That happened, and it is what this
    /// comment exists to stop happening again.
    ///
    /// A returning player who wanted silence hears one round of music and
    /// turns it down again, which is a far smaller harm than being stranded
    /// with a mute they cannot see and did not mean.
    pub fn load() -> Self {
        match storage::load(VOLUME_KEY).and_then(|v| v.parse::<u8>().ok()) {
            Some(level) => Self(level.min(VOLUME_STEPS)),
            None => Self::default(),
        }
    }
}

#[derive(Resource)]
pub struct Sounds {
    hit: Handle<AudioSource>,
    miss: Handle<AudioSource>,
    level: Handle<AudioSource>,
    theme: Handle<AudioSource>,
    round: Handle<AudioSource>,
}

fn load_sounds(mut commands: Commands, asset_server: Res<AssetServer>, mut volume: ResMut<Volume>) {
    *volume = Volume::load();

    commands.insert_resource(Sounds {
        hit: asset_server.load("sfx/hit.wav"),
        miss: asset_server.load("sfx/miss.wav"),
        level: asset_server.load("sfx/level.wav"),
        theme: asset_server.load("sfx/theme.wav"),
        round: asset_server.load("sfx/round.wav"),
    });
}

/// Which of the two loops belongs to the screen the player is on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Track {
    /// The menu, and every screen that is not a round in progress.
    Theme,
    /// A round in progress.
    Round,
}

/// The music that is playing, as the entity carrying its `AudioPlayer`.
#[derive(Resource, Default)]
pub struct Music {
    playing: Option<(Track, Entity)>,
}

/// Keeps the music matching the screen and the volume setting.
///
/// One reconciling system rather than a pair of `OnEnter`/`OnExit` handlers per
/// state: the track depends on two things that change independently — which
/// screen is up and whether the sound is off — and reconciling covers both
/// without any ordering between them. Turning the sound off stops the music;
/// turning it back up starts it again on whatever screen the player is now on.
///
/// Only *silence* is handled here. Every other volume change is a change to a
/// track that is already playing, and restarting the music from the top every
/// time someone taps the button would be worse than not having the button —
/// `follow_volume` adjusts the sink in place instead.
///
/// Stopping is now simply despawning the entity that carries the track, which
/// takes effect whether or not its sink has been created yet. The retry loop
/// this used to need — for the frame or two before the sink existed, and for the
/// weak handle that detached itself on drop — went away with the asset-based
/// `AudioSink` it was working around.
fn reconcile_music(
    mut commands: Commands,
    sounds: Option<Res<Sounds>>,
    volume: Res<Volume>,
    app_state: Res<State<AppState>>,
    mut music: ResMut<Music>,
) {
    let Some(sounds) = sounds else {
        return;
    };

    let wanted = if volume.is_silent() {
        None
    } else if *app_state.get() == AppState::Game {
        Some(Track::Round)
    } else {
        Some(Track::Theme)
    };

    if music.playing.as_ref().map(|(track, _)| *track) == wanted {
        return;
    }

    if let Some((_, entity)) = music.playing.take() {
        commands.entity(entity).despawn();
    }

    let Some(track) = wanted else {
        return;
    };

    let (source, level) = match track {
        Track::Theme => (sounds.theme.clone(), THEME_VOLUME),
        Track::Round => (sounds.round.clone(), ROUND_VOLUME),
    };

    let entity = commands
        .spawn((
            AudioPlayer::new(source),
            PlaybackSettings {
                mode: PlaybackMode::Loop,
                volume: bevy::audio::Volume::Linear(level * volume.scale()),
                ..default()
            },
        ))
        .id();
    music.playing = Some((track, entity));
}

/// Follows the volume setting on the music that is already playing.
///
/// Changing a track's level has to happen on the live sink: the alternative is
/// stopping and starting it, which drops the player back to the beginning of the
/// loop every time they tap the button.
///
/// `Changed<Volume>` alone is not enough, because the `AudioSink` component is
/// added a frame or two *after* the entity is spawned; the level is therefore
/// reapplied until it takes, which is what `applied` tracks.
fn follow_volume(
    mut sinks: Query<&mut AudioSink>,
    volume: Res<Volume>,
    music: Res<Music>,
    mut applied: Local<Option<f32>>,
) {
    let Some((track, entity)) = music.playing.as_ref() else {
        *applied = None;
        return;
    };

    let level = match track {
        Track::Theme => THEME_VOLUME,
        Track::Round => ROUND_VOLUME,
    } * volume.scale();

    if *applied == Some(level) {
        return;
    }

    if let Ok(mut sink) = sinks.get_mut(*entity) {
        sink.set_volume(bevy::audio::Volume::Linear(level));
        *applied = Some(level);
    }
}

fn play_pick_sounds(
    mut commands: Commands,
    sounds: Option<Res<Sounds>>,
    volume: Res<Volume>,
    mut events: MessageReader<InteractionAnimationEvent>,
) {
    let Some(sounds) = sounds else {
        return;
    };

    // Read every event, not just the first: two picks in one frame should not
    // leave one of them silent.
    for event in events.read() {
        if volume.is_silent() {
            continue;
        }

        let sound = if event.scored {
            sounds.hit.clone()
        } else {
            sounds.miss.clone()
        };

        // DESPAWN rather than ONCE: a one-shot entity that is never cleaned up
        // accumulates one corpse per pick for the length of the run.
        commands.spawn((
            AudioPlayer::new(sound),
            PlaybackSettings {
                mode: PlaybackMode::Despawn,
                volume: bevy::audio::Volume::Linear(0.85 * volume.scale()),
                ..default()
            },
        ));
    }
}

fn play_level_sound(
    mut commands: Commands,
    sounds: Option<Res<Sounds>>,
    volume: Res<Volume>,
    mut events: MessageReader<BannerEvent>,
) {
    let Some(sounds) = sounds else {
        return;
    };

    for event in events.read() {
        if volume.is_silent() {
            continue;
        }

        // Only the level up gets this sound. Banners now also announce
        // power-ups and goals, and giving those the level fanfare would spend
        // the loudest cue in the game on the smaller events until it stopped
        // meaning anything.
        if event.kind != BannerKind::LevelUp {
            continue;
        }

        commands.spawn((
            AudioPlayer::new(sounds.level.clone()),
            PlaybackSettings {
                mode: PlaybackMode::Despawn,
                volume: bevy::audio::Volume::Linear(0.9 * volume.scale()),
                ..default()
            },
        ));
    }
}
