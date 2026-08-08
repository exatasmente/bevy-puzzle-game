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
            .init_resource::<PendingLevelStart>()
            .init_resource::<MemoryPhase>()
            .register_type::<ColorPuzzle>()
            .add_system(start_puzzle_level.in_schedule(OnEnter(crate::AppState::Game)))
            .add_system(despaw_objects.in_schedule(OnExit(crate::AppState::Game)))
            .add_system(render_game_history.run_if(in_state(crate::AppState::LevelHistory)))
            // Not gated on a state: the "jogar novamente" button lives on the
            // GameOverResume screen, and gating this on GameOver alone meant the
            // event was read in a state it is never sent from. It is a no-op in
            // every frame without an event.
            .add_system(handle_new_game_event)
            .add_systems((
                tick_game_timer,
                store_last_interaction_state,
                spawn_objects,
                advance_pending_level,
                hide_memory_board,
                player_interaction,
            ).in_set(OnUpdate(crate::AppState::Game)))
            .add_system(background_transition);

    }
}
