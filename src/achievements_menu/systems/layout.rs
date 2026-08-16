//! Builds the goals screen.

use bevy::prelude::*;

use crate::achievements_menu::components::*;
use crate::achievements_menu::styles::*;
use crate::game::achievements::{Achievement, Achievements};
use crate::theme;

pub fn spawn_achievements_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    achievements: Res<Achievements>,
    window_query: Query<&Window>,
) {
    let width = window_query
        .single()
        .map(|window| theme::content_width(window.width()))
        .unwrap_or(theme::CONTENT_MAX_WIDTH);

    build_achievements_menu(&mut commands, &asset_server, &achievements, width);
}

pub fn build_achievements_menu(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    achievements: &Res<Achievements>,
    width: f32,
) -> Entity {
    commands
        .spawn((
            (menu_style(), BackgroundColor(theme::BACKGROUND)),
            AchievementsMenu,
        ))
        .with_children(|parent| {
            parent.spawn(theme::wrapped_text(
                "METAS",
                theme::text_title(asset_server),
                width,
            ));

            // The count is the screen's headline: it says how much is left
            // without making the player total up the list themselves.
            parent.spawn(theme::wrapped_text(
                format!(
                    "{} DE {}",
                    achievements.unlocked_count(),
                    Achievements::total()
                ),
                theme::text(asset_server, theme::TEXT_SM, theme::MUTED),
                width,
            ));

            for achievement in Achievement::iter() {
                let unlocked = achievements.has(achievement);
                // Reached goals wear their colour; the rest stay legible but
                // plainly unfinished.
                let (chip, title_color) = if unlocked {
                    (achievement.accent(), theme::ON_SURFACE)
                } else {
                    (LOCKED_COLOR, theme::MUTED)
                };

                parent
                    .spawn((row_style(width), BackgroundColor(theme::SURFACE)))
                    .with_children(|parent| {
                        parent.spawn((chip_style(), BackgroundColor(chip)));

                        parent
                            .spawn(text_column_style())
                            .with_children(|parent| {
                                parent.spawn(theme::wrapped_text(
                                    achievement.title(),
                                    theme::text(asset_server, theme::TEXT_SM, title_color),
                                    width - CHIP - theme::SPACE_LG,
                                ));
                                parent.spawn(theme::wrapped_text(
                                    achievement.description().to_uppercase(),
                                    theme::text(asset_server, theme::TEXT_XS, theme::MUTED),
                                    width - CHIP - theme::SPACE_LG,
                                ));
                            });
                    });
            }

            parent
                .spawn((
                    (
                        Button,
                        theme::button_style(width),
                        BackgroundColor(theme::PRIMARY),
                    ),
                    AchievementsBackButton,
                ))
                .with_children(|parent| {
                    parent.spawn(theme::wrapped_text(
                        "VOLTAR",
                        theme::text_button(asset_server),
                        width,
                    ));
                });
        })
        .id()
}

pub fn despawn_achievements_menu(
    mut commands: Commands,
    query: Query<Entity, With<AchievementsMenu>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Rebuilds for a window that changed size. Runs in `PostUpdate` for the same
/// reason every other relayout does: it despawns live `Button` entities.
pub fn relayout_achievements_menu(
    mut commands: Commands,
    mut relayout_events: MessageReader<crate::layout::RelayoutEvent>,
    asset_server: Res<AssetServer>,
    achievements: Res<Achievements>,
    window_query: Query<&Window>,
    menu_query: Query<Entity, With<AchievementsMenu>>,
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

    build_achievements_menu(
        &mut commands,
        &asset_server,
        &achievements,
        theme::content_width(window.width()),
    );
}
