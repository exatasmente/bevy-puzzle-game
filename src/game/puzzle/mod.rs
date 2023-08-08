pub mod components;
mod systems;


use systems::*;
use components::*;
use bevy::prelude::*;


pub struct PuzzlePlugin;

impl Plugin for PuzzlePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<StartLevelEvent>()
            .add_event::<LastInteractionEvent>()
            .add_event::<RenderLevelHistoryEvent>()
            .add_event::<NewGameEvent>()
            .init_resource::<ColorPuzzle>()
            .init_resource::<GameHistory>()
            .init_resource::<GameTimer>()
            .register_type::<ColorPuzzle>()
            .add_system(start_puzzle_level.in_schedule(OnEnter(super::AppState::Game)))
            .add_system(despaw_objects.in_schedule(OnExit(super::AppState::Game)))
            .add_systems((
                render_game_history,
            ).in_set(OnUpdate(super::AppState::LevelHistory)))
            .add_systems((
                handle_new_game_event,
            ).in_set(OnUpdate(super::AppState::GameOver)))
            .add_systems((
                background_transition,
                render_remaining_time,
                store_last_interaction_state,
                spawn_objects,
                player_interaction,
            ).in_set(OnUpdate(super::AppState::Game)));
            
    }
}
