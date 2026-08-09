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

/// Rebuilds the round list when the window changes size.
///
/// The list is already built from an event, so a relayout is just that event
/// again — no second code path to keep in step with the first.
pub fn relayout_game_history_menu(
    mut relayout_events: EventReader<crate::layout::RelayoutEvent>,
    mut spawn_pagination_event_writer: EventWriter<SpawnPaginationEvent>,
) {
    if relayout_events.iter().next().is_some() {
        spawn_pagination_event_writer.send(SpawnPaginationEvent);
    }
}

pub fn spawn_pagination_itens(
    mut commands: Commands,
    game_history: Res<GameHistory>,
    asset_server: Res<AssetServer>,
    mut pagination: ResMut<Pagination>,
    mut spawn_pagination_event_reader: EventReader<SpawnPaginationEvent>,
    muted: Res<crate::audio::Muted>,
    window_query: Query<&Window>,
) {
    if spawn_pagination_event_reader.iter().count() == 0 {
        return;
    }

    let width = window_query
        .get_single()
        .map(|window| theme::content_width(window.width()))
        .unwrap_or(theme::CONTENT_MAX_WIDTH);
    let label_width = history_card_label_width(width);

    pagination.set_max_page(game_history.levels_played);

    let Some(parent) = pagination.get_entity() else {
        return;
    };
    commands.entity(parent).despawn_descendants();

    commands.entity(parent).with_children(|parent| {
        parent.spawn(theme::wrapped_text(
            "PAUSA",
            get_title_text_style(&asset_server),
            width,
        ));

        if game_history.levels_played == 0 {
            parent.spawn(theme::wrapped_text(
                "NENHUM DESAFIO AINDA",
                get_label_text_style(&asset_server),
                width,
            ));
        }

        game_history.for_each_level(
            |index, level| {
                let scored = level.scored;

                parent
                    .spawn((
                        ButtonBundle {
                            style: history_card_style(width),
                            background_color: BUTTON.into(),
                            ..default()
                        },
                        LevelHistoryOption { index },
                    ))
                    .with_children(|parent| {
                        // The color the round was asking for.
                        parent.spawn(NodeBundle {
                            style: theme::tile_style(SWATCH_SIZE),
                            background_color: level.get_correct_color().into(),
                            ..default()
                        });

                        parent.spawn(theme::wrapped_text(
                            format!("DESAFIO {}", index + 1),
                            get_button_text_style(&asset_server),
                            label_width,
                        ));

                        // "OK"/"X" rather than a check mark: the display font
                        // has no glyph for one, and it would render blank.
                        parent.spawn(theme::wrapped_text(
                            if scored { "OK" } else { "X" },
                            theme::text(
                                &asset_server,
                                theme::TEXT_SM,
                                if scored { theme::SUCCESS } else { theme::DANGER },
                            ),
                            40.0,
                        ));
                    });
            },
            pagination.get_start_index(),
            pagination.get_items_per_page(),
        );

        // No rounds, no pager: an empty run would otherwise show "PAGINA 1 DE
        // 1" directly under "NENHUM DESAFIO AINDA".
        if game_history.levels_played > 0 {
            build_pagination_element(&asset_server, parent, &mut pagination, width);
        }
        build_actions(&asset_server, parent, width, muted.is_muted());
    });
}

fn build_actions(
    asset_server: &Res<AssetServer>,
    parent: &mut ChildBuilder,
    width: f32,
    muted: bool,
) {
    let text_width = theme::button_text_width(width);

    // The pause screen is where the mock-up put the sound control, and it is
    // the only screen a player reaches mid-run without losing anything.
    parent
        .spawn((
            ButtonBundle {
                style: button_style(width),
                background_color: BUTTON.into(),
                ..default()
            },
            SoundToggleButton,
        ))
        .with_children(|parent| {
            parent.spawn(theme::wrapped_text(
                if muted { "SOM: MUDO" } else { "SOM: LIGADO" },
                get_button_text_style(asset_server),
                text_width,
            ));
        });

    parent
        .spawn((
            ButtonBundle {
                style: button_style(width),
                background_color: theme::BUTTON_PRIMARY.into(),
                ..default()
            },
            ContinueButton,
        ))
        .with_children(|parent| {
            parent.spawn(theme::wrapped_text(
                "CONTINUAR",
                get_button_text_style(asset_server),
                text_width,
            ));
        });

    // Without this there is no way to finish an Infinite run: that mode has no
    // clock, so it could never reach the summary screen and its best score
    // could never be recorded.
    parent
        .spawn((
            ButtonBundle {
                style: button_style(width),
                // The one destructive action on the screen, and the only red
                // button in the game.
                background_color: theme::BUTTON_DANGER.into(),
                ..default()
            },
            EndRunButton,
        ))
        .with_children(|parent| {
            parent.spawn(theme::wrapped_text(
                "ENCERRAR PARTIDA",
                get_button_text_style(asset_server),
                text_width,
            ));
        });
}

fn build_pagination_element(
    asset_server: &Res<AssetServer>,
    parent: &mut ChildBuilder,
    pagination: &mut ResMut<Pagination>,
    width: f32,
) {
    if pagination.max_page == 0 {
        return;
    }

    parent
        .spawn(NodeBundle {
            style: pagination_container_style(width),
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

            parent.spawn(theme::wrapped_text(
                format!(
                    "PAGINA {} DE {}",
                    pagination.current_page + 1,
                    pagination.max_page
                ),
                get_label_text_style(asset_server),
                // The two arrows and the gaps between them take the rest.
                width - theme::TOUCH_TARGET * 2.0 - theme::SPACE_SM * 2.0,
            ));

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
            parent.spawn(theme::wrapped_text(
                label,
                get_pagination_button_text_style(asset_server),
                theme::TOUCH_TARGET,
            ));
        });
}
