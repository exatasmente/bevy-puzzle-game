use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::prelude::*;

use crate::game::puzzle::components::GameMode;
use crate::game::score::resources::BestScores;
use crate::main_menu::components::*;
use crate::main_menu::styles::*;
use crate::systems::BackgroundTranstion;
use crate::theme;

pub fn spawn_main_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    best_scores: Res<BestScores>,
    window_query: Query<&Window>,
) {
    // Cards are laid out against the real window width so their labels can be
    // fitted to a pixel width.
    let width = window_query
        .get_single()
        .map(|window| theme::content_width(window.width()))
        .unwrap_or(theme::CONTENT_MAX_WIDTH);

    build_main_menu(&mut commands, &asset_server, &best_scores, width);
}

/// Puts the app's own background back after a run.
///
/// The camera's clear color is whatever the last round tinted it, and nothing
/// used to reset it — so the menu inherited the color of the board the player
/// just left.
pub fn reset_background(
    mut camera_query: Query<(&mut Camera2d, &mut BackgroundTranstion), With<Camera>>,
) {
    let Ok((mut camera, mut transition)) = camera_query.get_single_mut() else {
        return;
    };

    transition.reset();
    transition.set_start_color(theme::BACKGROUND);
    transition.set_end_color(theme::BACKGROUND);
    camera.clear_color = ClearColorConfig::Custom(theme::BACKGROUND);
}

/// Rebuilds the menu for the new window size.
///
/// Cheaper options were considered and rejected: the card widths are baked into
/// dozens of nodes, and the fitted font size of every label depends on them, so
/// there is nothing to patch in place — the screen has to be built again.
pub fn relayout_main_menu(
    mut commands: Commands,
    mut relayout_events: EventReader<crate::layout::RelayoutEvent>,
    main_menu_query: Query<Entity, With<MainMenu>>,
    asset_server: Res<AssetServer>,
    best_scores: Res<BestScores>,
    window_query: Query<&Window>,
) {
    if relayout_events.iter().next().is_none() {
        return;
    }

    let Ok(window) = window_query.get_single() else {
        return;
    };

    for entity in main_menu_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    build_main_menu(
        &mut commands,
        &asset_server,
        &best_scores,
        theme::content_width(window.width()),
    );
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
    width: f32,
) -> Entity {
    // The card is a bordered wrapper around a padded row, so the text column has
    // the wrapper's padding and the chip's width taken off it.
    let text_width = mode_card_text_width(width);

    commands
        .spawn((
            NodeBundle {
                style: MAIN_MENU_STYLE,
                background_color: theme::BACKGROUND.into(),
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
                    // "COLOR" plain, "PUZZLE" running through the palette — the
                    // wordmark from the mock-up, built from text sections rather
                    // than an image so it stays crisp at any size.
                    parent.spawn(theme::wrapped_sections(
                        wordmark(),
                        theme::font(asset_server),
                        theme::TEXT_XL,
                        width,
                    ));
                    parent.spawn(theme::wrapped_text(
                        "ACHE O QUADRADO DIFERENTE",
                        get_mode_description_text_style(asset_server),
                        width,
                    ));
                });

            for game_mode in GameMode::iter() {
                let accent = game_mode.accent();

                parent
                    .spawn((
                        ButtonBundle {
                            style: theme::outlined_style(width),
                            background_color: card_border(accent).into(),
                            ..default()
                        },
                        PlayButton { game_mode },
                    ))
                    .with_children(|parent| {
                        parent
                            .spawn(NodeBundle {
                                style: mode_card_inner_style(),
                                background_color: theme::SURFACE.into(),
                                ..default()
                            })
                            .with_children(|parent| {
                                // The mode's marker. The mock-up puts an icon
                                // here; the display font has no glyph for one
                                // and there is no icon asset, so the color
                                // carries the identity on its own.
                                parent.spawn(NodeBundle {
                                    style: theme::tile_style(MODE_CHIP_SIZE),
                                    background_color: accent.into(),
                                    ..default()
                                });

                                parent
                                    .spawn(NodeBundle {
                                        style: mode_card_text_style(text_width),
                                        ..default()
                                    })
                                    .with_children(|parent| {
                                        parent.spawn(theme::wrapped_text(
                                            game_mode.as_str().to_uppercase(),
                                            get_mode_name_text_style(asset_server),
                                            text_width,
                                        ));
                                        // Say what the mode is before the player
                                        // commits to it.
                                        parent.spawn(theme::wrapped_text(
                                            game_mode.description().to_uppercase(),
                                            get_mode_description_text_style(asset_server),
                                            text_width,
                                        ));

                                        // Show the target before the run rather
                                        // than only after it: the number to beat
                                        // is what the run is for.
                                        let best = best_scores.get(game_mode);
                                        if best > 0 {
                                            parent.spawn(theme::wrapped_text(
                                                format!("RECORDE: {}", best),
                                                get_best_score_text_style(asset_server),
                                                text_width,
                                            ));
                                        }
                                    });
                            });
                    });
            }
        })
        .id()
}

/// The wordmark, one section per letter of "PUZZLE".
fn wordmark() -> Vec<(String, Color)> {
    let letters = [
        theme::PRIMARY,
        theme::PRIMARY,
        theme::INFO,
        theme::SUCCESS,
        theme::LIME,
        theme::ACCENT,
    ];

    let mut sections = vec![("COLOR ".to_string(), theme::ON_SURFACE)];
    for (letter, color) in "PUZZLE".chars().zip(letters) {
        sections.push((letter.to_string(), color));
    }

    sections
}
