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
            .add_system(spawn_game_over_menu.in_schedule(OnEnter(AppState::GameOver)))
            .add_system(interact_with_history_button.run_if(in_state(AppState::GameOver)))
            .add_system(interact_with_main_menu_button.run_if(in_state(AppState::GameOver)))
            // // OnExit State Systems
            .add_system(despawn_game_over_menu.in_schedule(OnExit(AppState::GameOver)));
    }
}
