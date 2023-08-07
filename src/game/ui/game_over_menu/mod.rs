mod components;
mod styles;
mod systems;

use systems::interactions::*;
use systems::layout::*;
use systems::updates::*;

use crate::AppState;
use bevy::prelude::*;
use crate::Pagination;
pub struct GameOverMenuPlugin;
pub struct SpawnPaginationEvent;


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
                    spawn_pagination_itens,
                )
                    .in_set(OnUpdate(AppState::GameOver)),
            )
            .init_resource::<Pagination>()
            .add_event::<SpawnPaginationEvent>()
            // // OnExit State Systems
            .add_system(despawn_game_over_menu.in_schedule(OnExit(AppState::GameOver)));
    }
}
