use bevy::prelude::*;

use crate::game::puzzle::components::GameHistory;
use crate::game::puzzle::components::GameMode;
use crate::game::ui::game_over_menu::components::*;
use crate::game::ui::game_over_menu::styles::*;

pub fn spawn_game_over_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_history : Res<GameHistory>,
) {
    build_game_over_menu(&mut commands, &asset_server, &game_history);
 
}

pub fn build_game_over_menu(
    commands: &mut Commands, 
    asset_server : &Res<AssetServer>,
    game_history : &Res<GameHistory>,
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

            parent.spawn(NodeBundle {
                style: GAME_OVER_MENU_CONTAINER_STYLE,
                background_color: BACKGROUND_COLOR.into(),
                ..default()
            }).with_children(|parent| {

                let labels =  vec![
                    format!("DESAFIOS JOGADOS  {}", game_history.levels_played),
                    format!("TOTAL DE PONTOS   {}", game_history.total_score),
                    format!("MAIOR SEQ ACERTOS {}", game_history.max_streak),
                ];

                for label in labels {
                    // spawn a text bundle with the text style form game_over_menu_styles method, and the text content "DESAFIOS: {}"
                    parent.spawn(TextBundle {
                        style: GAME_OVER_RESUME_TEXT_STYLE,
                        text: Text {
                            sections: vec![TextSection::new(
                                label,
                                get_resume_text_style(&asset_server),
                            )],
                            ..default()
                        },
                        ..default()
                    });
                }
                if game_history.game_mode == GameMode::TimeTrial {
                    parent.spawn(TextBundle {
                        style: GAME_OVER_RESUME_TEXT_STYLE,
                        text: Text {
                            sections: vec![TextSection::new(
                                format!("TEMPO TOTAL       {}", game_history.get_formatted_time()),
                                get_resume_text_style(&asset_server),
                            )],
                            ..default()
                        },
                        ..default()
                    });
                }
              
            parent
                .spawn((
                    ButtonBundle {
                        style: BUTTON_STYLE,
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    MainMenuButton {},
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        style: Style { ..default() },
                        text: Text {
                            sections: vec![TextSection::new(
                                "Menu Principal",
                                get_button_text_style(&asset_server),
                            )],
                            alignment: TextAlignment::Center,
                            ..default()
                        },
                        ..default()
                    });
                });
            parent
                .spawn((
                    ButtonBundle {
                        style: BUTTON_STYLE,
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    GameOverHistoryButton {},
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        style: Style { ..default() },
                        text: Text {
                            sections: vec![TextSection::new(
                                "Ver Historico",
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

pub fn spawn_resume_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn((
            ButtonBundle {
                style: GAME_OVER_MENU_STYLE,
                z_index: ZIndex::Local(2), // See Ref. 1
                ..default()
            },
            GameOverMenu {},
        ))
        .with_children(|parent| {
            parent.spawn(NodeBundle {
                style: GAME_OVER_MENU_CONTAINER_STYLE,
                background_color: BACKGROUND_COLOR.into(),
                ..default()
            }).with_children(|parent| {
                parent.spawn(TextBundle {
                    style: GAME_OVER_TEXT_STYLE,
                    text: Text {
                        sections: vec![TextSection::new(
                            "Fim de Jogo",
                            get_title_text_style(&asset_server),
                        ),],
                        ..default()
                    },
                    ..default()
                });

                parent.spawn(TextBundle {
                    style: GAME_OVER_TEXT_STYLE,
                    text: Text {
                        sections: vec![TextSection::new(
                            "Pressione para continuar",
                            get_resume_text_style(&asset_server),
                        ),],
                        ..default()
                    },
                    ..default()
                });
            });
        });

}

pub fn despawn_resume_screen(
    mut commands: Commands,
    game_over_menu_query: Query<Entity, With<GameOverMenu>>,
) {
    if let Ok(game_over_menu_entity) = game_over_menu_query.get_single() {
        commands.entity(game_over_menu_entity).despawn_recursive();
    }
}