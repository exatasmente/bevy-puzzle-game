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
    window_query: Query<&Window>,
) {
    let width = window_query
        .single()
        .map(|window| theme::content_width(window.width()))
        .unwrap_or(theme::CONTENT_MAX_WIDTH);

    build_game_over_menu(&mut commands, &asset_server, &game_history, &outcome, width);
}

pub fn build_game_over_menu(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    game_history: &Res<GameHistory>,
    outcome: &Res<LastRunOutcome>,
    width: f32,
) -> Entity {
    let text_width = theme::button_text_width(width);
    commands
        .spawn((
            (
                game_over_menu_style(),
                BackgroundColor(SCRIM),
                ZIndex(2),
            ),
            GameOverMenu,
        ))
        .with_children(|parent| {
            parent
                .spawn((game_over_menu_container_style(), BackgroundColor(SURFACE)))
                .with_children(|parent| {
                    parent.spawn(theme::wrapped_text(
                        "PONTOS",
                        get_label_text_style(asset_server),
                        width,
                    ));

                    // The headline number, in the celebration color when it is
                    // a record so the good news is legible before it is read.
                    let score_color = if outcome.is_record {
                        theme::ACCENT
                    } else {
                        theme::ON_SURFACE
                    };
                    let mut score_text = parent.spawn(theme::wrapped_text(
                        format!("{}", outcome.score),
                        theme::text_display(asset_server, score_color),
                        width,
                    ));
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
                    parent.spawn(theme::wrapped_text(
                        record_text,
                        theme::text(asset_server, theme::TEXT_SM, record_color),
                        width,
                    ));

                    let mut rows = vec![
                        ("DESAFIOS", format!("{}", game_history.levels_played)),
                        ("MAIOR SEQUENCIA", format!("{}", game_history.max_streak)),
                    ];

                    if game_history.game_mode == GameMode::TimeTrial {
                        rows.push(("TEMPO TOTAL", game_history.get_formatted_time()));
                    }

                    for (index, (label, value)) in rows.into_iter().enumerate() {
                        parent
                            .spawn(stat_row_style(width))
                            .with_children(|parent| {
                                // Label and value split the row, so neither can
                                // push the other off the edge.
                                parent.spawn((
                                    theme::wrapped_text(
                                        label,
                                        get_label_text_style(asset_server),
                                        width * 0.6,
                                    ),
                                    RevealIn::staggered(index),
                                ));
                                parent.spawn((
                                    theme::wrapped_text(
                                        value,
                                        get_resume_text_style(asset_server),
                                        width * 0.35,
                                    ),
                                    RevealIn::staggered(index),
                                ));
                            });
                    }

                    spawn_button(
                        parent,
                        asset_server,
                        "JOGAR NOVAMENTE",
                        primary_button_style(width),
                        text_width,
                        BUTTON_PRIMARY,
                        PlayAgainButton,
                    );
                    spawn_button(
                        parent,
                        asset_server,
                        "VER HISTORICO",
                        button_style(width),
                        text_width,
                        BUTTON,
                        GameOverHistoryButton,
                    );
                    spawn_button(
                        parent,
                        asset_server,
                        "MENU PRINCIPAL",
                        button_style(width),
                        text_width,
                        BUTTON,
                        MainMenuButton,
                    );
                });
        })
        .id()
}

fn spawn_button<M: Component>(
    parent: &mut ChildSpawnerCommands,
    asset_server: &Res<AssetServer>,
    label: &str,
    style: Node,
    text_width: f32,
    color: Color,
    marker: M,
) {
    parent
        .spawn((
            (Button, style, BackgroundColor(color)),
            marker,
        ))
        .with_children(|parent| {
            parent.spawn(theme::wrapped_text(
                label,
                get_button_text_style(asset_server),
                text_width,
            ));
        });
}

/// Rebuilds the summary for the new window size. See `relayout_main_menu`.
pub fn relayout_game_over_menu(
    mut commands: Commands,
    mut relayout_events: MessageReader<crate::layout::RelayoutEvent>,
    menu_query: Query<Entity, With<GameOverMenu>>,
    asset_server: Res<AssetServer>,
    game_history: Res<GameHistory>,
    outcome: Res<LastRunOutcome>,
    window_query: Query<&Window>,
) {
    if relayout_events.read().next().is_none() {
        return;
    }

    let Ok(window) = window_query.single() else {
        return;
    };

    for entity in menu_query.iter() {
        commands.entity(entity).despawn();
    }

    build_game_over_menu(
        &mut commands,
        &asset_server,
        &game_history,
        &outcome,
        theme::content_width(window.width()),
    );
}

pub fn despawn_game_over_menu(
    mut commands: Commands,
    game_over_menu_query: Query<Entity, With<GameOverMenu>>,
) {
    for entity in game_over_menu_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// The "fim de jogo" interstitial: one tap to continue, nothing to read.
pub fn spawn_resume_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window_query: Query<&Window>,
) {
    let width = window_query
        .single()
        .map(|window| theme::content_width(window.width()))
        .unwrap_or(theme::CONTENT_MAX_WIDTH);

    commands
        .spawn((
            (
                game_over_menu_style(),
                BackgroundColor(SCRIM),
                ZIndex(2),
            ),
            GameOverMenu,
        ))
        .with_children(|parent| {
            parent
                .spawn(game_over_menu_container_style())
                .with_children(|parent| {
                    parent.spawn(theme::wrapped_text(
                        "FIM DE JOGO",
                        get_title_text_style(&asset_server),
                        width,
                    ));
                    parent.spawn(theme::wrapped_text(
                        "TOQUE PARA CONTINUAR",
                        theme::text(&asset_server, theme::TEXT_SM, theme::MUTED),
                        width,
                    ));
                });
        });
}

pub fn despawn_resume_screen(
    mut commands: Commands,
    game_over_menu_query: Query<Entity, With<GameOverMenu>>,
) {
    for entity in game_over_menu_query.iter() {
        commands.entity(entity).despawn();
    }
}
