pub mod components;
pub mod objects;
mod systems;


use systems::*;
use components::*;

use bevy::prelude::*;

use self::objects::FoodBowl;
pub struct ObjectPlugin;

impl Plugin for ObjectPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(spawn_objects.in_schedule(OnEnter(super::AppState::Game)))
            .add_system(object_movement.run_if(in_state(super::AppState::Game)))
            .add_system(object_interaction.run_if(in_state(super::AppState::Game)))
            .add_system(interact_with_food_bowl.run_if(in_state(super::AppState::Game)));
    }
}
