mod components;
mod styles;
mod systems;

use systems::interactions::*;
use systems::layout::*;
use crate::AppState;
use bevy::prelude::*;
pub struct GameOverMenuPlugin;


impl Plugin for GameOverMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // OnEnter State Systems
            .add_system(spawn_game_over_menu.in_schedule(OnEnter(AppState::GameOverResume)))
            .add_system(spawn_resume_screen.in_schedule(OnEnter(AppState::GameOver)))
            .add_system(interact_with_game_over_resume_button.run_if(in_state(AppState::GameOver)))
            .add_system(interact_with_history_button.run_if(in_state(AppState::GameOverResume)))
            .add_system(interact_with_main_menu_button.run_if(in_state(AppState::GameOverResume)))
            // // OnExit State Systems
            .add_system(despawn_game_over_menu.in_schedule(OnExit(AppState::GameOverResume)))
            .add_system(despawn_resume_screen.in_schedule(OnExit(AppState::GameOver)));
    }
}
