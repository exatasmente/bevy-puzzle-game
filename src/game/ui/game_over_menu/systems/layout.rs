use bevy::{
    prelude::*,
    input::mouse::{MouseScrollUnit, MouseWheel},
};

use crate::game::puzzle::components::GameHistory;
use crate::game::ui::game_over_menu::components::*;
use crate::game::ui::game_over_menu::styles::*;

pub fn spawn_game_over_menu(mut commands: Commands, asset_server: Res<AssetServer>, game_history : Res<GameHistory>) {
    build_game_over_menu(&mut commands, &asset_server, &game_history);
}



pub fn build_game_over_menu(commands: &mut Commands, asset_server: &Res<AssetServer>, game_history : &Res<GameHistory>) -> Entity {
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
            parent
                .spawn(NodeBundle {
                    style: GAME_OVER_MENU_CONTAINER_STYLE,
                    background_color: BACKGROUND_COLOR.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                ..default()
                            },
                            ScrollingList::default(),
                        ))
                        .with_children(|parent| {
                            let _= &game_history.for_each_level(|index, level| {
                                parent
                                .spawn((
                                    ButtonBundle {
                                        style: BUTTON_HISTORY_STYLE,
                                        background_color: NORMAL_BUTTON.into(),
                                        ..default()
                                    },
                                    LevelHistoryOption { index },
                                ))
                                .with_children(|parent| {
                                    parent.spawn(TextBundle {
                                        style: Style { ..default() },
                                        text: Text {
                                            sections: vec![TextSection::new(
                                                format!("Level {}, Scored : {}", index + 1, level.scored),
                                                get_button_text_style(&asset_server),
                                            )],
                                            alignment: TextAlignment::Center,
                                            ..default()
                                        },
                                        ..default()
                                    });
                                });
                            });
                        });
                });
        })
        .id();

    game_over_menu_entity
}

pub fn despawn_game_over_menu(
    mut commands: Commands,
    game_over_menu_query: Query<Entity, With<GameOverMenu>>,
) {
    if let Ok(game_over_menu_entity) = game_over_menu_query.get_single() {
        commands.entity(game_over_menu_entity).despawn_recursive();
    }
}


#[derive(Component, Default)]
pub struct ScrollingList {
    position: f32,
}

pub fn mouse_scroll(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query_list: Query<(&mut ScrollingList, &mut Style, &Parent, &Node)>,
    query_node: Query<&Node>,
) {
    for mouse_wheel_event in mouse_wheel_events.iter() {
        for (mut scrolling_list, mut style, parent, list_node) in &mut query_list {
            let items_height = list_node.size().y;
            let container_height = query_node.get(parent.get()).unwrap().size().y;

            let max_scroll = (items_height - container_height).max(0.);

            let dy = match mouse_wheel_event.unit {
                MouseScrollUnit::Line => mouse_wheel_event.y * 20.,
                MouseScrollUnit::Pixel => mouse_wheel_event.y,
            };

            scrolling_list.position += dy;
            scrolling_list.position = scrolling_list.position.clamp(-max_scroll, 0.);
            style.position.top = Val::Px(scrolling_list.position);
        }
    }
}

// References
// 1. UI Z-Index
// https://github.com/bevyengine/bevy/blob/latest/examples/ui/z_index.rs
