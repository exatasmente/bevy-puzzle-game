mod components;
mod styles;
mod systems;

use systems::interactions::*;
use systems::layout::*;
use systems::updates::*;

use crate::AppState;
use bevy::prelude::*;
use crate::game::ui::game_over_menu::systems::Pagination;
pub struct GameOverMenuPlugin;



impl Plugin for GameOverMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // OnEnter State Systems
            .add_system(spawn_game_over_menu.in_schedule(OnEnter(AppState::GameOver)))
            .add_systems(
                (
                    interact_with_level_history_option,
                    interact_with_continue_button,
                    interact_with_pagination_button,
                    update_final_score_text,
                )
                    .in_set(OnUpdate(AppState::GameOver)),
            )
            .init_resource::<Pagination>()
            // // OnExit State Systems
            .add_system(despawn_game_over_menu.in_schedule(OnExit(AppState::GameOver)));
    }
}
