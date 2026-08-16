pub mod achievements;
pub mod puzzle;
pub mod score;
pub mod ui;

use puzzle::PuzzlePlugin;
use score::ScorePlugin;
use ui::GameUIPlugin;

use achievements::{check_achievements, load_achievements, note_mode_played, Achievements};
use crate::AppState;

use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((GameUIPlugin, ScorePlugin, PuzzlePlugin))
            .init_resource::<Achievements>()
            .add_systems(Startup, load_achievements)
            .add_systems(OnEnter(AppState::Game), note_mode_played)
            .add_systems(
                Update,
                check_achievements.run_if(in_state(AppState::Game)),
            );
    }
}
