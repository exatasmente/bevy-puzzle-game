use bevy::prelude::*;

use crate::game::ui::game_history_menu::components::*;
use crate::game::ui::game_history_menu::styles::*;
use crate::game::ui::game_history_menu::SpawnPaginationEvent;
use crate::pagination::Pagination;

pub fn spawn_game_history_menu(
    mut commands: Commands,
    mut pagination: ResMut<Pagination>,
    mut spawn_pagination_event_writer: MessageWriter<SpawnPaginationEvent>,
) {
    commands
        .spawn((
            (
                history_menu_style(),
                BackgroundColor(SCRIM),
                ZIndex(2),
            ),
            GameHistoryMenu,
        ))
        .with_children(|parent| {
            let pagination_container = parent
                .spawn((
                    (history_menu_container_style(), BackgroundColor(SURFACE)),
                    PaginationContainer,
                ))
                .id();

            pagination.set_entity(pagination_container);
        });

    spawn_pagination_event_writer.write(SpawnPaginationEvent);
}

pub fn despawn_game_history_menu(
    mut commands: Commands,
    game_history_menu_query: Query<Entity, With<GameHistoryMenu>>,
    mut pagination: ResMut<Pagination>,
) {
    for entity in game_history_menu_query.iter() {
        commands.entity(entity).despawn();
    }

    pagination.clear_entity();
}
