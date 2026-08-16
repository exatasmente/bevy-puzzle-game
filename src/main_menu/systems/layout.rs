use bevy::camera::ClearColorConfig;
use bevy::prelude::*;

use crate::game::puzzle::components::{level_for_score, GameMode};
use crate::game::score::resources::{BestScores, SavedRun};
use crate::main_menu::components::*;
use crate::main_menu::styles::*;
use crate::systems::BackgroundTranstion;
use crate::theme;

pub fn spawn_main_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    best_scores: Res<BestScores>,
    saved_run: Res<SavedRun>,
    window_query: Query<&Window>,
) {
    // Cards are laid out against the real window width so their labels can be
    // fitted to a pixel width.
    let (width, height) = window_query
        .single()
        .map(|window| (theme::content_width(window.width()), window.height()))
        .unwrap_or((theme::CONTENT_MAX_WIDTH, 720.0));

    build_main_menu(&mut commands, &asset_server, &best_scores, &saved_run, width, height);
}

/// Puts the app's own background back after a run.
///
/// The camera's clear color is whatever the last round tinted it, and nothing
/// used to reset it — so the menu inherited the color of the board the player
/// just left.
pub fn reset_background(
    mut camera_query: Query<(&mut Camera, &mut BackgroundTranstion), With<Camera2d>>,
) {
    let Ok((mut camera, mut transition)) = camera_query.single_mut() else {
        return;
    };

    transition.set_solid(theme::BACKGROUND);
    camera.clear_color = ClearColorConfig::Custom(theme::BACKGROUND);
}

/// Rebuilds the menu for the new window size.
///
/// Cheaper options were considered and rejected: the card widths are baked into
/// dozens of nodes, and the fitted font size of every label depends on them, so
/// there is nothing to patch in place — the screen has to be built again.
///
/// **This runs in `PostUpdate`, and any system that despawns a live `Button`
/// must.** Bevy 0.10's `bevy_ui::accessibility::button_changed` is registered
/// with a plain `add_system`, so it sits unordered in `Update` and queues an
/// `insert(AccessibilityNode)` for every button it has not tagged yet. If a
/// despawn is queued earlier in that same schedule, the despawn is applied
/// first and the insert then hits a dead entity — which is `B0003`, a hard
/// panic in 0.10, and a bare `RuntimeError: unreachable` in the browser.
/// Running here puts our commands in a later schedule than the a11y ones, so
/// the insert always lands on a live entity.
///
/// There is no feature to switch this off: `bevy_ui` depends on `bevy_a11y`
/// unconditionally and adds the plugin itself.
pub fn relayout_main_menu(
    mut commands: Commands,
    mut relayout_events: MessageReader<crate::layout::RelayoutEvent>,
    main_menu_query: Query<Entity, With<MainMenu>>,
    asset_server: Res<AssetServer>,
    best_scores: Res<BestScores>,
    saved_run: Res<SavedRun>,
    window_query: Query<&Window>,
) {
    if relayout_events.read().next().is_none() {
        return;
    }

    let Ok(window) = window_query.single() else {
        return;
    };

    for entity in main_menu_query.iter() {
        commands.entity(entity).despawn();
    }

    build_main_menu(
        &mut commands,
        &asset_server,
        &best_scores,
        &saved_run,
        theme::content_width(window.width()),
        window.height(),
    );
}

pub fn despawn_main_menu(mut commands: Commands, main_menu_query: Query<Entity, With<MainMenu>>) {
    for entity in main_menu_query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn build_main_menu(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    best_scores: &Res<BestScores>,
    saved_run: &Res<SavedRun>,
    width: f32,
    height: f32,
) -> Entity {
    // The card is a bordered wrapper around a padded row, so the text column has
    // the wrapper's padding and the chip's width taken off it.
    let text_width = mode_card_text_width(width);
    // The resume card, when there is one, is a card like the others and has to
    // be counted when the list is fitted to the window.
    let resume = saved_run.get();
    let cards = GameMode::iter().count() + usize::from(resume.is_some());
    let card_height = mode_card_height(height, cards);
    let chip_size = mode_chip_size(card_height);

    commands
        .spawn((
            (main_menu_style(), BackgroundColor(theme::BACKGROUND)),
            MainMenu,
        ))
        .with_children(|parent| {
            parent
                .spawn(title_style(height))
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

            // Offered first, and only when there is a run worth coming back
            // to: a player who left mid-run is here to finish it, not to read
            // the mode list again.
            if let Some(run) = resume {
                let game_mode = run.game_mode;

                // The lives are part of what the player is coming back to, so
                // the card says how many are left rather than making the
                // resumed run reveal it.
                let footnote = if game_mode.starting_lives().is_some() {
                    format!("PONTOS: {} - VIDAS: {}", run.score, run.lives)
                } else {
                    format!("PONTOS: {}", run.score)
                };

                spawn_card(
                    parent,
                    asset_server,
                    game_mode.accent(),
                    width,
                    card_height,
                    chip_size,
                    text_width,
                    "CONTINUAR",
                    &format!("{} - NIVEL {}", game_mode.as_str().to_uppercase(), level_for_score(run.score)),
                    Some(footnote),
                    ContinueRunButton {
                        game_mode,
                        score: run.score,
                        lives: run.lives,
                    },
                );
            }

            for game_mode in GameMode::iter() {
                // Show the target before the run rather than only after it: the
                // number to beat is what the run is for.
                let best = best_scores.get(game_mode);
                let footnote = (best > 0).then(|| format!("RECORDE: {}", best));

                spawn_card(
                    parent,
                    asset_server,
                    game_mode.accent(),
                    width,
                    card_height,
                    chip_size,
                    text_width,
                    &game_mode.as_str().to_uppercase(),
                    // Say what the mode is before the player commits to it.
                    &game_mode.description().to_uppercase(),
                    footnote,
                    PlayButton { game_mode },
                );
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

/// One card in the menu list: a colored chip, a name, a line of explanation and
/// an optional number underneath.
#[allow(clippy::too_many_arguments)]
fn spawn_card<M: Component>(
    parent: &mut ChildSpawnerCommands,
    asset_server: &Res<AssetServer>,
    accent: Color,
    width: f32,
    card_height: f32,
    chip_size: f32,
    text_width: f32,
    title: &str,
    description: &str,
    footnote: Option<String>,
    marker: M,
) {
    parent
        .spawn((
            (Button, theme::outlined_style(width), BackgroundColor(card_border(accent))),
            marker,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    mode_card_inner_style(card_height),
                    BackgroundColor(theme::SURFACE),
                ))
                .with_children(|parent| {
                    // The card's marker. The mock-up puts an icon here; the
                    // display font has no glyph for one and there is no icon
                    // asset, so the color carries the identity on its own.
                    parent.spawn((theme::tile_style(chip_size), BackgroundColor(accent)));

                    parent
                        .spawn(mode_card_text_style(text_width))
                        .with_children(|parent| {
                            parent.spawn(theme::wrapped_text(
                                title,
                                get_mode_name_text_style(asset_server),
                                text_width,
                            ));
                            parent.spawn(theme::wrapped_text(
                                description,
                                get_mode_description_text_style(asset_server),
                                text_width,
                            ));

                            if let Some(footnote) = footnote {
                                parent.spawn(theme::wrapped_text(
                                    footnote,
                                    get_best_score_text_style(asset_server),
                                    text_width,
                                ));
                            }
                        });
                });
        });
}
