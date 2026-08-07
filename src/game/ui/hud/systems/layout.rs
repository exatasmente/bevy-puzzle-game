//! Builds the in-game HUD.
//!
//! Score, streak and time used to be `Text2dBundle`s spawned in the world and
//! destroyed on every round, which meant the numbers could never animate — they
//! simply blinked to a new value. Here they are persistent UI nodes, so the
//! update systems can count them up, punch them and recolor them in place.

use bevy::prelude::*;

use crate::game::ui::hud::components::*;
use crate::game::ui::hud::styles::*;
use crate::theme;

pub fn spawn_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    build_hud(&mut commands, &asset_server);
}

pub fn build_hud(commands: &mut Commands, asset_server: &Res<AssetServer>) -> Entity {
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
                    style: TOP_BAR_STYLE,
                    ..default()
                })
                .with_children(|parent| {
                    spawn_stat::<ScoreValueText>(parent, asset_server, "PONTOS", "0", theme::ON_SURFACE);
                    // "SEQ", not "SEQUENCIA": at 320px the top bar holds three
                    // stats and a 48px button, and the longer word overflows.
                    spawn_stat::<StreakValueText>(parent, asset_server, "SEQ", "x0", theme::MUTED);
                    spawn_stat::<TimerValueText>(parent, asset_server, "TEMPO", "--", theme::ON_SURFACE);

                    // Pause. Previously an unpositioned, unsized transparent
                    // button; now a real, thumb-sized target.
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
                            parent.spawn(TextBundle {
                                text: Text::from_section("||", theme::text_button(asset_server))
                                    .with_alignment(TextAlignment::Center),
                                ..default()
                            });
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
                        TextBundle {
                            text: Text::from_section("NIVEL 1", theme::text_label(asset_server)),
                            ..default()
                        },
                        LevelValueText,
                    ));
                });

            // Goal gradient made visible: a bar that is visibly close to full
            // pulls harder than an unmarked distance.
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
                            background_color: theme::ACCENT.into(),
                            ..default()
                        },
                        LevelProgressFill,
                    ));
                });
        })
        .id()
}

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
            parent.spawn(TextBundle {
                text: Text::from_section(label, theme::text_label(asset_server))
                    .with_alignment(TextAlignment::Center),
                ..default()
            });
            parent.spawn((
                TextBundle {
                    text: Text::from_section(
                        initial_value,
                        theme::text(asset_server, theme::TEXT_MD, value_color),
                    )
                    .with_alignment(TextAlignment::Center),
                    ..default()
                },
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
                        style: BACK_BUTTON_STYLE,
                        background_color: BUTTON.into(),
                        ..default()
                    },
                    HistoryBackButtom,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section("VOLTAR", theme::text_button(&asset_server))
                            .with_alignment(TextAlignment::Center),
                        ..default()
                    });
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
