use bevy::prelude::*;

use crate::game::puzzle::components::GameMode;
use crate::game::score::resources::BestScores;
use crate::main_menu::components::*;
use crate::main_menu::styles::*;
use crate::theme;

pub fn spawn_main_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    best_scores: Res<BestScores>,
    window_query: Query<&Window>,
) {
    // Cards are laid out against the real window width so their labels can be
    // given a wrap width in pixels.
    let width = window_query
        .get_single()
        .map(|window| theme::content_width(window.width()))
        .unwrap_or(theme::CONTENT_MAX_WIDTH);

    build_main_menu(&mut commands, &asset_server, &best_scores, width);
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
    let text_width = mode_card_text_width(width);
    commands
        .spawn((
            NodeBundle {
                style: MAIN_MENU_STYLE,
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
                    parent.spawn(theme::wrapped_text(
                        "COLOR PUZZLE",
                        get_title_text_style(asset_server),
                        width,
                    ));
                    parent.spawn(theme::wrapped_text(
                        "ACHE A COR IGUAL AO FUNDO",
                        get_mode_description_text_style(asset_server),
                        width,
                    ));
                });

            for game_mode in GameMode::iter() {
                parent
                    .spawn((
                        ButtonBundle {
                            style: mode_card_style(width),
                            background_color: BUTTON.into(),
                            ..default()
                        },
                        PlayButton { game_mode },
                    ))
                    .with_children(|parent| {
                        parent.spawn(theme::wrapped_text(
                            game_mode.as_str().to_uppercase(),
                            get_mode_name_text_style(asset_server),
                            text_width,
                        ));
                        // Say what the mode is before the player commits to it.
                        parent.spawn(theme::wrapped_text(
                            game_mode.description().to_uppercase(),
                            get_mode_description_text_style(asset_server),
                            text_width,
                        ));

                        // Show the target before the run rather than only after
                        // it: the number to beat is what the run is for.
                        let best = best_scores.get(game_mode);
                        if best > 0 {
                            parent.spawn(theme::wrapped_text(
                                format!("RECORDE {}", best),
                                get_best_score_text_style(asset_server),
                                text_width,
                            ));
                        }
                    });
            }
        })
        .id()
}
