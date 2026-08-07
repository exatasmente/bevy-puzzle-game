use bevy::prelude::*;

use crate::game::ui::game_history_menu::components::*;
use crate::game::ui::game_history_menu::styles::*;
use crate::game::ui::game_history_menu::SpawnPaginationEvent;
use crate::pagination::Pagination;

pub fn spawn_game_history_menu(
    mut commands: Commands,
    mut pagination: ResMut<Pagination>,
    mut spawn_pagination_event_writer: EventWriter<SpawnPaginationEvent>,
) {
    commands
        .spawn((
            NodeBundle {
                style: HISTORY_MENU_STYLE,
                background_color: SCRIM.into(),
                z_index: ZIndex::Local(2),
                ..default()
            },
            GameHistoryMenu,
        ))
        .with_children(|parent| {
            let pagination_container = parent
                .spawn((
                    NodeBundle {
                        style: HISTORY_MENU_CONTAINER_STYLE,
                        background_color: SURFACE.into(),
                        ..default()
                    },
                    PaginationContainer,
                ))
                .id();

            pagination.set_entity(pagination_container);
        });

    spawn_pagination_event_writer.send(SpawnPaginationEvent);
}

pub fn despawn_game_history_menu(
    mut commands: Commands,
    game_history_menu_query: Query<Entity, With<GameHistoryMenu>>,
    mut pagination: ResMut<Pagination>,
) {
    for entity in game_history_menu_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    pagination.clear_entity();
}
