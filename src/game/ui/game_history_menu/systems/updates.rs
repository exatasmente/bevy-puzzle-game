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
    mut relayout_events: MessageReader<crate::layout::RelayoutEvent>,
    mut spawn_pagination_event_writer: MessageWriter<SpawnPaginationEvent>,
) {
    if relayout_events.read().next().is_some() {
        spawn_pagination_event_writer.write(SpawnPaginationEvent);
    }
}

pub fn spawn_pagination_itens(
    mut commands: Commands,
    game_history: Res<GameHistory>,
    asset_server: Res<AssetServer>,
    mut pagination: ResMut<Pagination>,
    mut spawn_pagination_event_reader: MessageReader<SpawnPaginationEvent>,
    volume: Res<crate::audio::Volume>,
    window_query: Query<&Window>,
) {
    if spawn_pagination_event_reader.read().count() == 0 {
        return;
    }

    let width = window_query
        .single()
        .map(|window| theme::content_width(window.width()))
        .unwrap_or(theme::CONTENT_MAX_WIDTH);
    let label_width = history_card_label_width(width);

    pagination.set_max_page(game_history.levels_played);

    let Some(parent) = pagination.get_entity() else {
        return;
    };
    commands.entity(parent).despawn_related::<Children>();

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
                        (Button, history_card_style(width), BackgroundColor(BUTTON)),
                        LevelHistoryOption { index },
                    ))
                    .with_children(|parent| {
                        // The color the round was asking for.
                        parent.spawn((theme::tile_style(SWATCH_SIZE), BackgroundColor(level.get_correct_color())));

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
        build_actions(&asset_server, parent, width, volume.label());
    });
}

fn build_actions(
    asset_server: &Res<AssetServer>,
    parent: &mut ChildSpawnerCommands,
    width: f32,
    sound_label: String,
) {
    let text_width = theme::button_text_width(width);

    // The pause screen is where the mock-up put the sound control, and it is
    // the only screen a player reaches mid-run without losing anything. One
    // button cycling down through the steps, rather than a slider: Bevy 0.10
    // has no slider widget, and a drag target is the wrong shape for a thumb.
    parent
        .spawn((
            (Button, button_style(width), BackgroundColor(BUTTON)),
            SoundToggleButton,
        ))
        .with_children(|parent| {
            parent.spawn((
                theme::wrapped_text(sound_label, get_button_text_style(asset_server), text_width),
                SoundToggleLabel,
            ));
        });

    parent
        .spawn((
            (Button, button_style(width), BackgroundColor(theme::BUTTON_PRIMARY)),
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
            (
                Button,
                button_style(width),
                // The one destructive action on the screen, and the only red
                // button in the game.
                BackgroundColor(theme::BUTTON_DANGER),
            ),
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
    parent: &mut ChildSpawnerCommands,
    pagination: &mut ResMut<Pagination>,
    width: f32,
) {
    if pagination.max_page == 0 {
        return;
    }

    parent
        .spawn(pagination_container_style(width))
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
    parent: &mut ChildSpawnerCommands,
    asset_server: &Res<AssetServer>,
    label: &str,
    index: usize,
) {
    parent
        .spawn((
            (Button, button_pagination_style(), BackgroundColor(BUTTON)),
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

/// Writes the volume into the button's label without rebuilding the screen.
///
/// The screen used to be rebuilt on every press, which worked natively and did
/// nothing visible on the web: a `Text` despawned and respawned in the same
/// frame goes on rendering its old glyphs there, so the reading stayed at 100%
/// while the sound was plainly getting quieter. Writing the value into the
/// existing text avoids the teardown altogether, and is what a one-word change
/// deserved in the first place.
pub fn update_sound_label(
    volume: Res<crate::audio::Volume>,
    mut query: Query<&mut Text, With<SoundToggleLabel>>,
) {
    // Deliberately not gated on `volume.is_changed()`. Systems in a tuple run
    // in an arbitrary order, so this one can run *before* the button handler in
    // the very frame the value changes — and since the flag is only set for
    // that one frame, the label would then never catch up. Comparing the string
    // is immune to the ordering, and still writes only when it differs, which is
    // what keeps Bevy from re-laying out the text every frame.
    let label = volume.label();

    for mut text in query.iter_mut() {
        if text.0 != label {
            text.0 = label.clone();
        }
    }
}
