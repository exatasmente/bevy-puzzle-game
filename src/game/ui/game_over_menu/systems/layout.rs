use bevy::prelude::*;

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
                    // Final Score Text
                    parent.spawn((
                        TextBundle {
                            text: Text {
                                sections: vec![TextSection::new(
                                    format!("Total Score: {}", &game_history.total_score),
                                    get_final_score_text_style(&asset_server),
                                )],
                                alignment: TextAlignment::Center,
                                ..default()
                            },
                            ..default()
                        },
                        FinalScoreText {},
                    ));
                    // Restart Button

                    let _= &game_history.for_each_level(|index, level| {
                        parent
                        .spawn((
                            ButtonBundle {
                                style: BUTTON_STYLE,
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

                    parent
                        .spawn((
                            ButtonBundle {
                                style: BUTTON_STYLE,
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            RestartButton {},
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle {
                                style: Style { ..default() },
                                text: Text {
                                    sections: vec![TextSection::new(
                                        "Restart",
                                        get_button_text_style(&asset_server),
                                    )],
                                    alignment: TextAlignment::Center,
                                    ..default()
                                },
                                ..default()
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

// References
// 1. UI Z-Index
// https://github.com/bevyengine/bevy/blob/latest/examples/ui/z_index.rs
