use bevy::prelude::*;

use crate::game::puzzle;
use crate::game::puzzle::components::GameHistory;
use crate::game::ui::game_over_menu::components::*;
use crate::game::ui::game_over_menu::styles::*;
use crate::game::ui::game_over_menu::systems::Pagination;
use crate::game::puzzle::components::GameTimer;

pub fn spawn_game_over_menu(
    mut commands: Commands, 
    asset_server: Res<AssetServer>, 
    game_history : Res<GameHistory>,
    mut pagination : ResMut<Pagination>,
    mut game_timer: ResMut<GameTimer>,
) {
    build_game_over_menu(&mut commands, &asset_server, &game_history, &mut pagination, &mut game_timer);
}



pub fn build_game_over_menu(
    commands: &mut Commands, 
    asset_server: &Res<AssetServer>, 
    game_history : &Res<GameHistory>, 
    pagination : &mut ResMut<Pagination>, 
    game_timer: &mut ResMut<GameTimer>,
) -> Entity {
    pagination.set_max_page(game_history.levels_played);
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
                            style: PAGINATION_TEXT_STYLE,
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
                }, pagination.get_start_index(), pagination.get_items_per_page());

                build_pagination_element(asset_server, parent, pagination);
                build_back_button(asset_server, game_timer, parent);
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

pub fn build_back_button(
    asset_server: &Res<AssetServer>,
    game_timer: &mut ResMut<GameTimer>,
    parent : &mut ChildBuilder,
) {

    let text = if game_timer.timer.finished() {
        "Restart"
    } else {
        "Continue"
    };
    
    parent
        .spawn((
            ButtonBundle {
                style: BUTTON_HISTORY_STYLE,
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            ContinueButton {},
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                style: Style { ..default() },
                text: Text {
                    sections: vec![TextSection::new(
                        text,
                        get_button_text_style(&asset_server),
                    )],
                    alignment: TextAlignment::Center,
                    ..default()
                },
                ..default()
            });
        });
}

fn build_pagination_element(
    asset_server: &Res<AssetServer>,
    parent : &mut ChildBuilder,
    mut pagination: &mut ResMut<Pagination>,
) {
    parent
    .spawn(NodeBundle {
        style: PAGINATION_CONTAINER_STYLE,
        background_color: BACKGROUND_COLOR.into(),
        ..default()
    })
    .with_children(|parent| {
        for index in 0..pagination.max_page {
            parent
                .spawn((
                    ButtonBundle {
                        style: BUTTON_PAGINATION_STYLE,
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    PaginationOption { index },
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        style: PAGINATION_TEXT_STYLE,
                        text: Text {
                            sections: vec![TextSection::new(
                                format!("{}", index + 1),
                                get_button_text_style(&asset_server),
                            )],
                            alignment: TextAlignment::Center,
                            ..default()
                        },
                        ..default()
                    });
                });
        }
    });
}

// References
// 1. UI Z-Index
// https://github.com/bevyengine/bevy/blob/latest/examples/ui/z_index.rs
