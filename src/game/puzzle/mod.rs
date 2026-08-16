pub mod components;
mod systems;


use systems::*;
use components::*;
use bevy::prelude::*;


pub struct PuzzlePlugin;

impl Plugin for PuzzlePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_message::<StartLevelEvent>()
            .add_message::<LastInteractionEvent>()
            .add_message::<RenderLevelHistoryEvent>()
            .add_message::<NewGameEvent>()
            .init_resource::<ColorPuzzle>()
            .init_resource::<GameHistory>()
            .init_resource::<GameTimer>()
            .init_resource::<PendingLevelStart>()
            .init_resource::<MemoryPhase>()
            .init_resource::<RoundIntro>()
            .register_type::<ColorPuzzle>()
            .add_systems(OnEnter(crate::AppState::Game), start_puzzle_level)
            .add_systems(OnExit(crate::AppState::Game), despaw_objects)
            .add_systems(Update, render_game_history.run_if(in_state(crate::AppState::LevelHistory)))
            // Not gated on a state: the "jogar novamente" button lives on the
            // GameOverResume screen, and gating this on GameOver alone meant the
            // event was read in a state it is never sent from. It is a no-op in
            // every frame without an event.
            .add_systems(Update, handle_new_game_event)
            .add_systems(Update, (
                tick_game_timer,
                store_last_interaction_state,
                spawn_objects,
                advance_pending_level,
                tick_round_intro,
                hide_memory_board,
                player_interaction,
            ).run_if(in_state(crate::AppState::Game)))
            .add_systems(Update, background_transition);

    }
}
