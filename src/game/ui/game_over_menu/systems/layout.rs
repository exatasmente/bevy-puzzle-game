//! The end-of-run screen.
//!
//! Ranked so the eye lands in the order that matters: what you scored, whether
//! it beat your best, the detail behind it, and then the one button that starts
//! another run. The retry is the primary action because the moment right after
//! a run ends is the only moment the player is certain to still be here.

use bevy::prelude::*;

use crate::feedback::{PopAnim, RevealIn};
use crate::game::puzzle::components::GameHistory;
use crate::game::puzzle::components::GameMode;
use crate::game::score::resources::LastRunOutcome;
use crate::game::ui::game_over_menu::components::*;
use crate::game::ui::game_over_menu::styles::*;
use crate::theme;

pub fn spawn_game_over_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_history: Res<GameHistory>,
    outcome: Res<LastRunOutcome>,
) {
    build_game_over_menu(&mut commands, &asset_server, &game_history, &outcome);
}

pub fn build_game_over_menu(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    game_history: &Res<GameHistory>,
    outcome: &Res<LastRunOutcome>,
) -> Entity {
    commands
        .spawn((
            NodeBundle {
                style: GAME_OVER_MENU_STYLE,
                background_color: SCRIM.into(),
                z_index: ZIndex::Local(2),
                ..default()
            },
            GameOverMenu,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: GAME_OVER_MENU_CONTAINER_STYLE,
                    background_color: SURFACE.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section("PONTOS", get_label_text_style(asset_server)),
                        ..default()
                    });

                    // The headline number, in the celebration color when it is
                    // a record so the good news is legible before it is read.
                    let score_color = if outcome.is_record {
                        theme::ACCENT
                    } else {
                        theme::ON_SURFACE
                    };
                    let mut score_text = parent.spawn(TextBundle {
                        text: Text::from_section(
                            format!("{}", outcome.score),
                            theme::text_display(asset_server, score_color),
                        ),
                        ..default()
                    });
                    if outcome.is_record {
                        // The loudest animation in the game, spent on the rarest
                        // moment it has.
                        score_text.insert(PopAnim::large());
                    }

                    // Either a celebration or a target. Never nothing: an
                    // end screen with no comparison gives the player no reason
                    // to go again.
                    let (record_text, record_color) = if outcome.is_record {
                        ("NOVO RECORDE!".to_string(), theme::ACCENT)
                    } else {
                        (format!("RECORDE {}", outcome.best), theme::MUTED)
                    };
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            record_text,
                            theme::text(asset_server, theme::TEXT_SM, record_color),
                        ),
                        ..default()
                    });

                    let mut rows = vec![
                        ("DESAFIOS", format!("{}", game_history.levels_played)),
                        ("MAIOR SEQUENCIA", format!("{}", game_history.max_streak)),
                    ];

                    if game_history.game_mode == GameMode::TimeTrial {
                        rows.push(("TEMPO TOTAL", game_history.get_formatted_time()));
                    }

                    for (index, (label, value)) in rows.into_iter().enumerate() {
                        parent
                            .spawn(NodeBundle {
                                style: STAT_ROW_STYLE,
                                ..default()
                            })
                            .with_children(|parent| {
                                parent.spawn((
                                    TextBundle {
                                        text: Text::from_section(
                                            label,
                                            get_label_text_style(asset_server),
                                        ),
                                        ..default()
                                    },
                                    RevealIn::staggered(index),
                                ));
                                parent.spawn((
                                    TextBundle {
                                        text: Text::from_section(
                                            value,
                                            get_resume_text_style(asset_server),
                                        ),
                                        ..default()
                                    },
                                    RevealIn::staggered(index),
                                ));
                            });
                    }

                    spawn_button(
                        parent,
                        asset_server,
                        "JOGAR NOVAMENTE",
                        PRIMARY_BUTTON_STYLE,
                        BUTTON_PRIMARY,
                        PlayAgainButton,
                    );
                    spawn_button(
                        parent,
                        asset_server,
                        "VER HISTORICO",
                        BUTTON_STYLE,
                        BUTTON,
                        GameOverHistoryButton,
                    );
                    spawn_button(
                        parent,
                        asset_server,
                        "MENU PRINCIPAL",
                        BUTTON_STYLE,
                        BUTTON,
                        MainMenuButton,
                    );
                });
        })
        .id()
}

fn spawn_button<M: Component>(
    parent: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
    label: &str,
    style: Style,
    color: Color,
    marker: M,
) {
    parent
        .spawn((
            ButtonBundle {
                style,
                background_color: color.into(),
                ..default()
            },
            marker,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section(label, get_button_text_style(asset_server))
                    .with_alignment(TextAlignment::Center),
                ..default()
            });
        });
}

pub fn despawn_game_over_menu(
    mut commands: Commands,
    game_over_menu_query: Query<Entity, With<GameOverMenu>>,
) {
    for entity in game_over_menu_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// The "fim de jogo" interstitial: one tap to continue, nothing to read.
pub fn spawn_resume_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            ButtonBundle {
                style: GAME_OVER_MENU_STYLE,
                background_color: SCRIM.into(),
                z_index: ZIndex::Local(2),
                ..default()
            },
            GameOverMenu,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: GAME_OVER_MENU_CONTAINER_STYLE,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "FIM DE JOGO",
                            get_title_text_style(&asset_server),
                        ),
                        ..default()
                    });
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "TOQUE PARA CONTINUAR",
                            theme::text(&asset_server, theme::TEXT_SM, theme::MUTED),
                        ),
                        ..default()
                    });
                });
        });
}

pub fn despawn_resume_screen(
    mut commands: Commands,
    game_over_menu_query: Query<Entity, With<GameOverMenu>>,
) {
    for entity in game_over_menu_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
