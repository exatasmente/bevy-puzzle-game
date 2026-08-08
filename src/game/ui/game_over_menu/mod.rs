mod components;
mod styles;
mod systems;

use crate::game::score::RecordOutcomeSet;
use crate::AppState;
use bevy::prelude::*;
use systems::interactions::*;
use systems::layout::*;

pub struct GameOverMenuPlugin;

impl Plugin for GameOverMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // OnEnter State Systems. Ordered after the run outcome is recorded
            // so the screen reads this run's numbers, not the previous run's.
            .add_system(
                spawn_game_over_menu
                    .after(RecordOutcomeSet)
                    .in_schedule(OnEnter(AppState::GameOverResume)),
            )
            .add_system(spawn_resume_screen.in_schedule(OnEnter(AppState::GameOver)))
            .add_system(interact_with_game_over_resume_button.run_if(in_state(AppState::GameOver)))
            .add_systems(
                (
                    interact_with_play_again_button,
                    interact_with_history_button,
                    interact_with_main_menu_button,
                    relayout_game_over_menu,
                )
                    .in_set(OnUpdate(AppState::GameOverResume)),
            )
            // OnExit State Systems
            .add_system(despawn_game_over_menu.in_schedule(OnExit(AppState::GameOverResume)))
            .add_system(despawn_resume_screen.in_schedule(OnExit(AppState::GameOver)));
    }
}
