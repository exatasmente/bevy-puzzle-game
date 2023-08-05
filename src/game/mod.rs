
pub mod object;
pub mod puzzle;
mod systems;

use puzzle::PuzzlePlugin;
use object::ObjectPlugin;

use bevy::prelude::*;

use crate::events::GameOver;
use crate::AppState;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // Events
            .add_event::<GameOver>()
            .add_plugin(ObjectPlugin)
            .add_plugin(PuzzlePlugin);
    }
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum SimulationState {
    #[default]
    Running,
    Paused,
}
