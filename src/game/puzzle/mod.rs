pub mod components;
mod systems;


use systems::*;
use components::*;
use bevy::prelude::*;


pub struct PuzzlePlugin;

impl Plugin for PuzzlePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(start_puzzle_level.in_schedule(OnEnter(super::AppState::Game)))
            .add_system(background_transition.run_if(in_state(super::AppState::Game)))
            .add_system(render_remaining_time.run_if(in_state(super::AppState::Game)))
            .add_system(spawn_objects.run_if(in_state(super::AppState::Game)))
            .add_system(player_interaction.run_if(in_state(super::AppState::Game)))
            .add_system(despaw_objects.in_schedule(OnExit(super::AppState::Game)))
            .add_event::<StartLevelEvent>()
            .init_resource::<ColorPuzzle>()
            .register_type::<ColorPuzzle>();
    }
}
