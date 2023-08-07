use bevy::prelude::*;

use crate::game::ui::game_over_menu::components::*;
use crate::game::ui::game_over_menu::styles::*;
use crate::pagination::Pagination;
use crate::game::ui::game_over_menu::SpawnPaginationEvent;

pub fn spawn_game_over_menu(
    mut commands: Commands, 
    mut pagination : ResMut<Pagination>,
    mut spawn_pagination_event_writer: EventWriter<SpawnPaginationEvent>,
) {
    build_game_over_menu(&mut commands,  &mut pagination);
    spawn_pagination_event_writer.send(SpawnPaginationEvent);
}

#[derive(Component)]
pub struct PaginationContainer;

pub fn build_game_over_menu(
    commands: &mut Commands, 
    pagination : &mut ResMut<Pagination>, 
) -> Entity {
    let game_over_menu_entity = commands
        .spawn((
            NodeBundle {
                style: GAME_OVER_MENU_STYLE,
                z_index: ZIndex::Local(2), // See Ref. 1
                ..default()
            },
            GameOverMenu {},
        ))
        .with_children(|parent| {
            let pagination_container = parent
                .spawn((NodeBundle {
                    style: GAME_OVER_MENU_CONTAINER_STYLE,
                    background_color: BACKGROUND_COLOR.into(),
                    ..default()
                },  PaginationContainer {})).id();

            pagination.set_entity(pagination_container);


        })
        .id();

    game_over_menu_entity
}

pub fn despawn_game_over_menu(
    mut commands: Commands,
    game_over_menu_query: Query<Entity, With<GameOverMenu>>,
    mut pagination : ResMut<Pagination>
) {
    if let Ok(game_over_menu_entity) = game_over_menu_query.get_single() {
        commands.entity(game_over_menu_entity).despawn_recursive();
        pagination.clear_entity();
    }
}

