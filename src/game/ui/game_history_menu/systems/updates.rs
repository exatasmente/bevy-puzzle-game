//! Rebuilds the paginated list of past rounds.
//!
//! The list used to read `Level 1, Scored : true` — English, and a boolean where
//! the interesting information is *which color you were looking for*. Each row
//! now shows that color and a plain OK/X outcome, so the screen works as a
//! review of what went wrong rather than a log.

use bevy::prelude::*;

use crate::game::puzzle::components::GameHistory;
use crate::game::ui::game_history_menu::components::*;
use crate::game::ui::game_history_menu::styles::*;
use crate::game::ui::game_history_menu::SpawnPaginationEvent;
use crate::pagination::Pagination;
use crate::theme;

pub fn spawn_pagination_itens(
    mut commands: Commands,
    game_history: Res<GameHistory>,
    asset_server: Res<AssetServer>,
    mut pagination: ResMut<Pagination>,
    mut spawn_pagination_event_reader: EventReader<SpawnPaginationEvent>,
) {
    if spawn_pagination_event_reader.iter().count() == 0 {
        return;
    }

    pagination.set_max_page(game_history.levels_played);

    let Some(parent) = pagination.get_entity() else {
        return;
    };
    commands.entity(parent).despawn_descendants();

    commands.entity(parent).with_children(|parent| {
        parent.spawn(TextBundle {
            text: Text::from_section("PAUSA", get_title_text_style(&asset_server))
                .with_alignment(TextAlignment::Center),
            ..default()
        });

        if game_history.levels_played == 0 {
            parent.spawn(TextBundle {
                text: Text::from_section(
                    "NENHUM DESAFIO AINDA",
                    get_label_text_style(&asset_server),
                )
                .with_alignment(TextAlignment::Center),
                ..default()
            });
        }

        game_history.for_each_level(
            |index, level| {
                let scored = level.scored;

                parent
                    .spawn((
                        ButtonBundle {
                            style: HISTORY_CARD_STYLE,
                            background_color: BUTTON.into(),
                            ..default()
                        },
                        LevelHistoryOption { index },
                    ))
                    .with_children(|parent| {
                        // The color the round was asking for.
                        parent.spawn(NodeBundle {
                            style: SWATCH_STYLE,
                            background_color: level.get_correct_color().into(),
                            ..default()
                        });

                        parent.spawn(TextBundle {
                            text: Text::from_section(
                                format!("DESAFIO {}", index + 1),
                                get_button_text_style(&asset_server),
                            ),
                            ..default()
                        });

                        // "OK"/"X" rather than a check mark: the display font
                        // has no glyph for one, and it would render blank.
                        parent.spawn(TextBundle {
                            text: Text::from_section(
                                if scored { "OK" } else { "X" },
                                theme::text(
                                    &asset_server,
                                    theme::TEXT_SM,
                                    if scored { theme::SUCCESS } else { theme::DANGER },
                                ),
                            ),
                            ..default()
                        });
                    });
            },
            pagination.get_start_index(),
            pagination.get_items_per_page(),
        );

        build_pagination_element(&asset_server, parent, &mut pagination);
        build_actions(&asset_server, parent);
    });
}

fn build_actions(asset_server: &Res<AssetServer>, parent: &mut ChildBuilder) {
    parent
        .spawn((
            ButtonBundle {
                style: BUTTON_STYLE,
                background_color: BUTTON.into(),
                ..default()
            },
            ContinueButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section("CONTINUAR", get_button_text_style(asset_server))
                    .with_alignment(TextAlignment::Center),
                ..default()
            });
        });

    // Without this there is no way to finish an Infinite run: that mode has no
    // clock, so it could never reach the summary screen and its best score
    // could never be recorded.
    parent
        .spawn((
            ButtonBundle {
                style: BUTTON_STYLE,
                background_color: BUTTON.into(),
                ..default()
            },
            EndRunButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section("ENCERRAR PARTIDA", get_button_text_style(asset_server))
                    .with_alignment(TextAlignment::Center),
                ..default()
            });
        });
}

fn build_pagination_element(
    asset_server: &Res<AssetServer>,
    parent: &mut ChildBuilder,
    pagination: &mut ResMut<Pagination>,
) {
    if pagination.max_page == 0 {
        return;
    }

    parent
        .spawn(NodeBundle {
            style: PAGINATION_CONTAINER_STYLE,
            ..default()
        })
        .with_children(|parent| {
            spawn_pagination_button(
                parent,
                asset_server,
                "<",
                if pagination.current_page > 0 {
                    pagination.current_page - 1
                } else {
                    0
                },
            );

            parent.spawn(TextBundle {
                text: Text::from_section(
                    format!(
                        "PAGINA {} DE {}",
                        pagination.current_page + 1,
                        pagination.max_page
                    ),
                    get_label_text_style(asset_server),
                )
                .with_alignment(TextAlignment::Center),
                ..default()
            });

            spawn_pagination_button(
                parent,
                asset_server,
                ">",
                if pagination.current_page + 1 < pagination.max_page {
                    pagination.current_page + 1
                } else {
                    pagination.current_page
                },
            );
        });
}

fn spawn_pagination_button(
    parent: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
    label: &str,
    index: usize,
) {
    parent
        .spawn((
            ButtonBundle {
                style: BUTTON_PAGINATION_STYLE,
                background_color: BUTTON.into(),
                ..default()
            },
            PaginationOption { index },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section(label, get_pagination_button_text_style(asset_server))
                    .with_alignment(TextAlignment::Center),
                ..default()
            });
        });
}
