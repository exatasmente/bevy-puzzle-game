
pub mod object;
mod systems;
use object::ObjectPlugin;

use object::objects::FoodBowl;

use bevy::prelude::*;

use crate::events::GameOver;
use crate::AppState;


use self::object::objects::ColorPuzzle;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // Events
            .add_event::<GameOver>()
            .add_plugin(ObjectPlugin)
            .register_type::<FoodBowl>();
    }
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum SimulationState {
    #[default]
    Running,
    Paused,
}
