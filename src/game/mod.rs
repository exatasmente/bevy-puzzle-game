pub mod puzzle;
pub mod score;
pub mod ui;

use puzzle::PuzzlePlugin;
use score::ScorePlugin;
use ui::GameUIPlugin;

use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((GameUIPlugin, ScorePlugin, PuzzlePlugin));
    }
}
