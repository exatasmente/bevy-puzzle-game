//! Builds the in-game HUD.
//!
//! Score, streak and time used to be `Text2dBundle`s spawned in the world and
//! destroyed on every round, which meant the numbers could never animate — they
//! simply blinked to a new value. Here they are persistent UI nodes, so the
//! update systems can count them up, punch them and recolor them in place.

use bevy::prelude::*;

use crate::game::puzzle::components::ColorPuzzle;
use crate::game::ui::hud::components::*;
use crate::game::ui::hud::styles::*;
use crate::theme;

pub fn spawn_hud(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    puzzle: Res<ColorPuzzle>,
) {
    build_hud(&mut commands, &asset_server, puzzle.max_lives());
}

/// `lives` is the mode's full complement, and zero in a timed mode — the row of
/// markers is built once, at its final length, because the number of lives a
/// run can hold never changes mid-run.
pub fn build_hud(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    lives: usize,
) -> Entity {
    commands
        .spawn((
            NodeBundle {
                style: HUD_ROOT_STYLE,
                ..default()
            },
            HudRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: HUD_PANEL_STYLE,
                    background_color: HUD_PANEL_COLOR.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: TOP_BAR_STYLE,
                            ..default()
                        })
                        .with_children(|parent| {
                            // Each stat gets its own color, so the eye can find
                            // the one it wants without reading the labels.
                            spawn_stat::<ScoreValueText>(
                                parent,
                                asset_server,
                                "PONTOS",
                                "0",
                                theme::PRIMARY,
                            );
                            spawn_divider(parent);
                            // "SEQ", not "SEQUENCIA": at 320px the top bar holds
                            // three stats and a 48px button, and the longer word
                            // squeezes the others out.
                            spawn_stat::<StreakValueText>(
                                parent,
                                asset_server,
                                "SEQ",
                                "x0",
                                theme::MUTED,
                            );
                            spawn_divider(parent);
                            spawn_stat::<TimerValueText>(
                                parent,
                                asset_server,
                                "TEMPO",
                                "--",
                                theme::INFO,
                            );

                            // Pause. Previously an unpositioned, unsized
                            // transparent button; now a real, thumb-sized
                            // target.
                            parent
                                .spawn((
                                    ButtonBundle {
                                        style: ICON_BUTTON_STYLE,
                                        background_color: BUTTON.into(),
                                        ..default()
                                    },
                                    HistoryButtom,
                                ))
                                .with_children(|parent| {
                                    parent.spawn(theme::wrapped_text(
                                        "||",
                                        theme::text(
                                            asset_server,
                                            theme::TEXT_SM,
                                            theme::PRIMARY,
                                        ),
                                        theme::TOUCH_TARGET,
                                    ));
                                });
                        });

                    // Level row: the current level and how close the next one is.
                    parent
                        .spawn(NodeBundle {
                            style: LEVEL_ROW_STYLE,
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                theme::wrapped_text(
                                    "NIVEL 1",
                                    theme::text_label(asset_server),
                                    theme::CONTENT_MAX_WIDTH,
                                ),
                                LevelValueText,
                            ));

                            if lives > 0 {
                                spawn_lives_row(parent, lives);
                            }
                        });

                    // Goal gradient made visible: a bar that is visibly close to
                    // full pulls harder than an unmarked distance.
                    parent
                        .spawn(NodeBundle {
                            style: PROGRESS_TRACK_STYLE,
                            background_color: PROGRESS_TRACK_COLOR.into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                NodeBundle {
                                    style: PROGRESS_FILL_STYLE,
                                    background_color: theme::PRIMARY.into(),
                                    ..default()
                                },
                                LevelProgressFill,
                            ));
                        });
                });
        })
        .id()
}

/// The run's lives, as one marker each.
///
/// All of them are spawned lit; `update_lives_pips` is what puts them out. That
/// keeps the "how many are left" decision in one place rather than splitting it
/// between the builder and the updater.
fn spawn_lives_row(parent: &mut ChildBuilder, lives: usize) {
    parent
        .spawn((
            NodeBundle {
                style: LIVES_ROW_STYLE,
                ..default()
            },
            LivesRow,
        ))
        .with_children(|parent| {
            for index in 0..lives {
                parent.spawn((
                    NodeBundle {
                        style: LIVES_PIP_STYLE,
                        background_color: theme::DANGER.into(),
                        ..default()
                    },
                    LivesPip { index },
                ));
            }
        });
}

/// A hairline between two stats.
fn spawn_divider(parent: &mut ChildBuilder) {
    parent.spawn(NodeBundle {
        style: STAT_DIVIDER_STYLE,
        background_color: theme::OUTLINE.into(),
        ..default()
    });
}

/// Widest a stat's label or value may be before it wraps. Stats flex, so this
/// is a ceiling rather than the column width.
const STAT_TEXT_WIDTH: f32 = 96.0;

/// A label stacked over a value, tagged with the marker used to update it.
fn spawn_stat<M: Component + Default>(
    parent: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
    label: &str,
    initial_value: &str,
    value_color: Color,
) {
    parent
        .spawn(NodeBundle {
            style: STAT_STYLE,
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(theme::wrapped_text(
                label,
                theme::text_label(asset_server),
                STAT_TEXT_WIDTH,
            ));
            parent.spawn((
                theme::wrapped_text(
                    initial_value,
                    theme::text(asset_server, theme::TEXT_MD, value_color),
                    STAT_TEXT_WIDTH,
                ),
                M::default(),
            ));
        });
}

pub fn spawn_back_button(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            NodeBundle {
                style: BACK_BUTTON_ROOT_STYLE,
                ..default()
            },
            BackButtonRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    ButtonBundle {
                        style: theme::button_style(BACK_BUTTON_WIDTH),
                        background_color: BUTTON.into(),
                        ..default()
                    },
                    HistoryBackButtom,
                ))
                .with_children(|parent| {
                    parent.spawn(theme::wrapped_text(
                        "VOLTAR",
                        theme::text_button(&asset_server),
                        theme::button_text_width(BACK_BUTTON_WIDTH),
                    ));
                });
        });
}

pub fn despawn_hud(mut commands: Commands, hud_query: Query<Entity, With<HudRoot>>) {
    for entity in hud_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn despawn_back_button(mut commands: Commands, query: Query<Entity, With<BackButtonRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
