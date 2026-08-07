use bevy::prelude::*;

use crate::game::puzzle::components::GameMode;
use crate::game::score::resources::BestScores;
use crate::main_menu::components::*;
use crate::main_menu::styles::*;

pub fn spawn_main_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    best_scores: Res<BestScores>,
) {
    build_main_menu(&mut commands, &asset_server, &best_scores);
}

pub fn despawn_main_menu(mut commands: Commands, main_menu_query: Query<Entity, With<MainMenu>>) {
    for entity in main_menu_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn build_main_menu(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    best_scores: &Res<BestScores>,
) -> Entity {
    commands
        .spawn((
            NodeBundle {
                style: MAIN_MENU_STYLE,
                ..default()
            },
            MainMenu,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: TITLE_STYLE,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "COLOR PUZZLE",
                            get_title_text_style(asset_server),
                        )
                        .with_alignment(TextAlignment::Center),
                        ..default()
                    });
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "ACHE A COR IGUAL AO FUNDO",
                            get_mode_description_text_style(asset_server),
                        )
                        .with_alignment(TextAlignment::Center),
                        ..default()
                    });
                });

            for game_mode in GameMode::iter() {
                parent
                    .spawn((
                        ButtonBundle {
                            style: MODE_CARD_STYLE,
                            background_color: BUTTON.into(),
                            ..default()
                        },
                        PlayButton { game_mode },
                    ))
                    .with_children(|parent| {
                        parent.spawn(TextBundle {
                            text: Text::from_section(
                                game_mode.as_str().to_uppercase(),
                                get_mode_name_text_style(asset_server),
                            )
                            .with_alignment(TextAlignment::Center),
                            ..default()
                        });
                        // Say what the mode is before the player commits to it.
                        parent.spawn(TextBundle {
                            text: Text::from_section(
                                game_mode.description(),
                                get_mode_description_text_style(asset_server),
                            )
                            .with_alignment(TextAlignment::Center),
                            ..default()
                        });

                        // Show the target before the run rather than only after
                        // it: the number to beat is what the run is for.
                        let best = best_scores.get(game_mode);
                        if best > 0 {
                            parent.spawn(TextBundle {
                                text: Text::from_section(
                                    format!("RECORDE {}", best),
                                    get_best_score_text_style(asset_server),
                                )
                                .with_alignment(TextAlignment::Center),
                                ..default()
                            });
                        }
                    });
            }
        })
        .id()
}
