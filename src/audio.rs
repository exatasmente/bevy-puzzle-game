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

const MUTED_KEY: &str = "color_puzzle.muted";

/// Both tracks sit under the effects on purpose: the music is there so silence
/// does not read as a broken build, and a hit still has to cut through it.
const THEME_VOLUME: f32 = 0.45;
const ROUND_VOLUME: f32 = 0.32;

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Muted>()
            .init_resource::<Music>()
            .add_startup_system(load_sounds)
            .add_system(play_pick_sounds)
            .add_system(play_level_sound)
            .add_system(reconcile_music);
    }
}

/// Whether the player has silenced the game. Persisted, because a player who
/// muted a game once meant it.
#[derive(Resource, Default)]
pub struct Muted(bool);

impl Muted {
    pub fn is_muted(&self) -> bool {
        self.0
    }

    pub fn toggle(&mut self) {
        self.0 = !self.0;
        storage::save(MUTED_KEY, if self.0 { "1" } else { "0" });
    }

    pub fn load() -> Self {
        Self(storage::load(MUTED_KEY).as_deref() == Some("1"))
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

fn load_sounds(mut commands: Commands, asset_server: Res<AssetServer>, mut muted: ResMut<Muted>) {
    *muted = Muted::load();

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

/// Keeps the music matching the screen and the mute setting.
///
/// One reconciling system rather than a pair of `OnEnter`/`OnExit` handlers per
/// state: the track depends on two things that change independently — which
/// screen is up and whether the player has muted — and reconciling covers both
/// without any ordering between them. Muting stops the music; unmuting starts
/// it again on whatever screen the player is now on.
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
    muted: Res<Muted>,
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

    let wanted = if muted.is_muted() {
        None
    } else if app_state.0 == AppState::Game {
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

    let (source, volume) = match track {
        Track::Theme => (sounds.theme.clone(), THEME_VOLUME),
        Track::Round => (sounds.round.clone(), ROUND_VOLUME),
    };

    let handle = sinks.get_handle(
        audio.play_with_settings(source, PlaybackSettings::LOOP.with_volume(volume)),
    );
    music.playing = Some((track, handle));
}

fn play_pick_sounds(
    audio: Res<Audio>,
    sounds: Option<Res<Sounds>>,
    muted: Res<Muted>,
    mut events: EventReader<InteractionAnimationEvent>,
) {
    let Some(sounds) = sounds else {
        return;
    };

    // Read every event, not just the first: two picks in one frame should not
    // leave one of them silent.
    for event in events.iter() {
        if muted.is_muted() {
            continue;
        }

        let sound = if event.scored {
            sounds.hit.clone()
        } else {
            sounds.miss.clone()
        };

        audio.play_with_settings(sound, PlaybackSettings::ONCE.with_volume(0.85));
    }
}

fn play_level_sound(
    audio: Res<Audio>,
    sounds: Option<Res<Sounds>>,
    muted: Res<Muted>,
    mut events: EventReader<BannerEvent>,
) {
    let Some(sounds) = sounds else {
        return;
    };

    for _ in events.iter() {
        if muted.is_muted() {
            continue;
        }

        // The banner is only ever a level up now, so it needs no filtering.
        audio.play_with_settings(sounds.level.clone(), PlaybackSettings::ONCE.with_volume(0.9));
    }
}
