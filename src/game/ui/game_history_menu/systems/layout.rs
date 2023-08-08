use bevy::prelude::*;

use crate::game::ui::game_history_menu::components::*;
use crate::game::ui::game_history_menu::styles::*;
use crate::pagination::Pagination;
use crate::game::ui::game_history_menu::SpawnPaginationEvent;

pub fn spawn_game_history_menu(
    mut commands: Commands, 
    mut pagination : ResMut<Pagination>,
    mut spawn_pagination_event_writer: EventWriter<SpawnPaginationEvent>,
) {
    commands
        .spawn((
            NodeBundle {
                style: GAME_OVER_MENU_STYLE,
                z_index: ZIndex::Local(2), // See Ref. 1
                ..default()
            },
            GameHistoryMenu {},
        ))
        .with_children(|parent| {
            let pagination_container = parent
                .spawn((NodeBundle {
                    style: GAME_OVER_MENU_CONTAINER_STYLE,
                    background_color: BACKGROUND_COLOR.into(),
                    ..default()
                },  PaginationContainer {})).id();

            pagination.set_entity(pagination_container);
        });
        
    spawn_pagination_event_writer.send(SpawnPaginationEvent);
}

#[derive(Component)]
pub struct PaginationContainer;

pub fn despawn_game_history_menu(
    mut commands: Commands,
    game_history_menu_query: Query<Entity, With<GameHistoryMenu>>,
    mut pagination : ResMut<Pagination>
) {
    if let Ok(game_history_menu_entity) = game_history_menu_query.get_single() {
        commands.entity(game_history_menu_entity).despawn_recursive();
        pagination.clear_entity();
    }
}

