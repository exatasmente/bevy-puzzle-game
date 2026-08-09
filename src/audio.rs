//! The game's sound.
//!
//! Everything is driven off events the game already sends, rather than by
//! sprinkling `audio.play` through the systems that happen to notice something:
//! a pick sends `InteractionAnimationEvent` with `scored` on it, and a level up
//! sends a `BannerEvent`. Sound is a *reading* of what happened, so it belongs
//! downstream of the events that say what happened.
//!
//! ## The browser
//!
//! Chrome will not start an `AudioContext` before a user gesture. It creates one
//! suspended and logs about it, and resumes on the first real tap — which in
//! this game is the tap that starts a run. So the console warning is back (it is
//! why the plugin was switched off in the first place) and sound works from the
//! first tap onward. There is no way to have both in Bevy 0.10: the plugin
//! builds its context at startup.
//!
//! Note the API here is Bevy 0.10's — `Res<Audio>` and `play_with_settings`.
//! `AudioBundle`, which every current example uses, arrived in 0.12 and does not
//! exist in this tree.

use bevy::prelude::*;

use crate::events::InteractionAnimationEvent;
use crate::feedback::BannerEvent;
use crate::storage;

const MUTED_KEY: &str = "color_puzzle.muted";

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Muted>()
            .add_startup_system(load_sounds)
            .add_system(play_pick_sounds)
            .add_system(play_level_sound);
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
}

fn load_sounds(mut commands: Commands, asset_server: Res<AssetServer>, mut muted: ResMut<Muted>) {
    *muted = Muted::load();

    commands.insert_resource(Sounds {
        hit: asset_server.load("sfx/hit.wav"),
        miss: asset_server.load("sfx/miss.wav"),
        level: asset_server.load("sfx/level.wav"),
    });
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

        audio.play_with_settings(sound, PlaybackSettings::ONCE.with_volume(0.6));
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
        audio.play_with_settings(sounds.level.clone(), PlaybackSettings::ONCE.with_volume(0.7));
    }
}
