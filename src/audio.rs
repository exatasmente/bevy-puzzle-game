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
//! own when the player finally taps. Bevy 0.10 builds its context at startup,
//! from cpal, and never calls `resume`, so without help the game fetches its
//! sounds and plays none of them. The help lives in `docs/index.html`, which
//! wraps the `AudioContext` constructor before loading the module and resumes
//! the instances on the first input; see the comment there for why the listener
//! has to be in the capture phase. Nothing in this file can reach that context.
//!
//! Note the API here is Bevy 0.10's — `Res<Audio>` and `play_with_settings`.
//! `AudioBundle`, which every current example uses, arrived in 0.12 and does not
//! exist in this tree.

use bevy::prelude::*;

use crate::events::InteractionAnimationEvent;
use crate::feedback::BannerEvent;
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
            .add_startup_system(load_sounds)
            .add_system(play_pick_sounds)
            .add_system(play_level_sound)
            .add_system(reconcile_music)
            .add_system(follow_volume);
    }
}

/// How loud the game is, in steps. Persisted, because a player who turned the
/// sound down once meant it.
///
/// Steps rather than a continuous slider: Bevy 0.10's UI has no slider, and one
/// button that cycles is both easier to hit on a phone than a drag target and
/// honest about how little precision anyone wants here. Zero is off, which is
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

/// The music that is playing, and the music that is on its way out.
#[derive(Resource, Default)]
pub struct Music {
    playing: Option<(Track, Handle<AudioSink>)>,
    /// Sinks asked to stop that could not be reached yet — see below.
    stopping: Vec<Handle<AudioSink>>,
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
/// Two details of Bevy 0.10's audio are load bearing here, and both are quiet
/// failures rather than compile errors:
///
/// - `play_with_settings` hands back a **weak** handle, and `AudioSink`'s `Drop`
///   calls `detach()`. Letting that handle go for a *looping* track means music
///   that plays forever with nothing left that can reach it. So the handle is
///   upgraded to a strong one immediately, through `sinks.get_handle`, and kept
///   in this resource — the same thing Bevy's own `audio_control` example does.
/// - The `AudioSink` asset does not exist until `play_queued_audio_system` has
///   run, so `sinks.get` returns `None` for a frame or two after starting. A
///   stop therefore has to keep trying rather than fire once and give up, which
///   is what `stopping` is for: switching screens quickly would otherwise leave
///   the old track playing under the new one.
fn reconcile_music(
    audio: Res<Audio>,
    sinks: Res<Assets<AudioSink>>,
    sounds: Option<Res<Sounds>>,
    volume: Res<Volume>,
    app_state: Res<State<AppState>>,
    mut music: ResMut<Music>,
) {
    let Some(sounds) = sounds else {
        return;
    };

    // Retire anything still waiting to be stopped.
    music.stopping.retain(|handle| match sinks.get(handle) {
        Some(sink) => {
            sink.stop();
            false
        }
        None => true,
    });

    let wanted = if volume.is_silent() {
        None
    } else if app_state.get() == AppState::Game {
        Some(Track::Round)
    } else {
        Some(Track::Theme)
    };

    if music.playing.as_ref().map(|(track, _)| *track) == wanted {
        return;
    }

    if let Some((_, handle)) = music.playing.take() {
        music.stopping.push(handle);
    }

    let Some(track) = wanted else {
        return;
    };

    let (source, level) = match track {
        Track::Theme => (sounds.theme.clone(), THEME_VOLUME),
        Track::Round => (sounds.round.clone(), ROUND_VOLUME),
    };

    let handle = sinks.get_handle(
        audio.play_with_settings(
            source,
            PlaybackSettings::LOOP.with_volume(level * volume.scale()),
        ),
    );
    music.playing = Some((track, handle));
}

/// Follows the volume setting on the music that is already playing.
///
/// Changing a track's level has to happen on the live sink: the alternative is
/// stopping and starting it, which drops the player back to the beginning of the
/// loop every time they tap the button.
///
/// `Changed<Volume>` alone is not enough, because the sink asset can arrive a
/// frame or two *after* the change; the level is therefore reapplied until it
/// takes, which is what `applied` tracks.
fn follow_volume(
    sinks: Res<Assets<AudioSink>>,
    volume: Res<Volume>,
    music: Res<Music>,
    mut applied: Local<Option<f32>>,
) {
    let Some((track, handle)) = music.playing.as_ref() else {
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

    if let Some(sink) = sinks.get(handle) {
        sink.set_volume(level);
        *applied = Some(level);
    }
}

fn play_pick_sounds(
    audio: Res<Audio>,
    sounds: Option<Res<Sounds>>,
    volume: Res<Volume>,
    mut events: MessageReader<InteractionAnimationEvent>,
) {
    let Some(sounds) = sounds else {
        return;
    };

    // Read every event, not just the first: two picks in one frame should not
    // leave one of them silent.
    for event in events.iter() {
        if volume.is_silent() {
            continue;
        }

        let sound = if event.scored {
            sounds.hit.clone()
        } else {
            sounds.miss.clone()
        };

        audio.play_with_settings(sound, PlaybackSettings::ONCE.with_volume(0.85 * volume.scale()));
    }
}

fn play_level_sound(
    audio: Res<Audio>,
    sounds: Option<Res<Sounds>>,
    volume: Res<Volume>,
    mut events: MessageReader<BannerEvent>,
) {
    let Some(sounds) = sounds else {
        return;
    };

    for _ in events.iter() {
        if volume.is_silent() {
            continue;
        }

        // The banner is only ever a level up now, so it needs no filtering.
        audio.play_with_settings(sounds.level.clone(), PlaybackSettings::ONCE.with_volume(0.9 * volume.scale()));
    }
}
