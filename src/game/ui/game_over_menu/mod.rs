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
            .add_systems(
                OnEnter(AppState::GameOverResume),
                spawn_game_over_menu.after(RecordOutcomeSet),
            )
            .add_systems(OnEnter(AppState::GameOver), spawn_resume_screen)
            .add_systems(
                Update,
                interact_with_game_over_resume_button.run_if(in_state(AppState::GameOver)),
            )
            .add_systems(
                Update,
                (
                    interact_with_play_again_button,
                    interact_with_history_button,
                    interact_with_main_menu_button,
                    interact_with_share_button,
                )
                    .run_if(in_state(AppState::GameOverResume)),
            )
            .add_systems(
                PostUpdate,
                relayout_game_over_menu.run_if(in_state(AppState::GameOverResume)),
            )
            // OnExit State Systems
            .add_systems(OnExit(AppState::GameOverResume), despawn_game_over_menu)
            .add_systems(OnExit(AppState::GameOver), despawn_resume_screen);
    }
}
