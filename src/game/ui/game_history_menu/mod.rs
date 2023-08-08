mod components;
mod styles;
mod systems;

use systems::interactions::*;
use systems::layout::*;
use systems::updates::*;

use crate::AppState;
use bevy::prelude::*;
use crate::Pagination;

pub struct GameHistoryMenuPlugin;
pub struct SpawnPaginationEvent;


impl Plugin for GameHistoryMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<Pagination>()
            .add_event::<SpawnPaginationEvent>()
            // OnEnter State Systems
            .add_system(spawn_game_history_menu.in_schedule(OnEnter(AppState::History)))
            .add_system(interact_with_level_history_option.run_if(in_state(AppState::History)))
            .add_system(interact_with_continue_button.run_if(in_state(AppState::History)))
            .add_system(interact_with_pagination_button.run_if(in_state(AppState::History)))
            .add_system(spawn_pagination_itens.run_if(in_state(AppState::History)))
            // // OnExit State Systems
            .add_system(despawn_game_history_menu.in_schedule(OnExit(AppState::History)));
    }
}
