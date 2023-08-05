pub mod components;
pub mod objects;
mod systems;


use systems::*;
use objects::StartLevelEvent;
use objects::ColorPuzzle;

use bevy::prelude::*;


pub struct ObjectPlugin;

impl Plugin for ObjectPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(start_puzzle_level.in_schedule(OnEnter(super::AppState::Game)))
            .add_system(background_transition.run_if(in_state(super::AppState::Game)))
            .add_system(render_remaining_time.run_if(in_state(super::AppState::Game)))
            .add_system(spawn_objects.run_if(in_state(super::AppState::Game)))
            .add_system(player_interaction.run_if(in_state(super::AppState::Game)))
            .add_system(object_movement.run_if(in_state(super::AppState::Game)))
            .add_system(despaw_objects.in_schedule(OnExit(super::AppState::Game)))
            .add_event::<StartLevelEvent>()
            .init_resource::<ColorPuzzle>()
            .register_type::<ColorPuzzle>();
    }
}
