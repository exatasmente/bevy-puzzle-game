use bevy::prelude::*;

use crate::game::puzzle;
use crate::game::puzzle::components::GameHistory;
use crate::game::ui::game_over_menu::components::*;
use crate::game::ui::game_over_menu::styles::*;
use crate::game::ui::game_over_menu::systems::Pagination;
use crate::game::puzzle::components::GameTimer;
use crate::game::ui::game_over_menu::SpawnPaginationEvent;

pub fn spawn_pagination_itens(
    mut commands: Commands,
    game_history : Res<GameHistory>, 
    asset_server: Res<AssetServer>,
    mut pagination :ResMut<Pagination>,
    mut game_timer: ResMut<GameTimer>,
    mut spawn_pagination_event_reader: EventReader<SpawnPaginationEvent>,
) {

    if spawn_pagination_event_reader.iter().count() == 0 {
        return;
    }

    pagination.set_max_page(game_history.levels_played);
    let parent = pagination.get_entity().unwrap();
    commands.entity(parent).despawn_descendants();

    commands.entity(parent).with_children(
        |parent| {
            let _= &game_history.for_each_level(|index, level| {
            parent
            .spawn((
                ButtonBundle {
                    style: BUTTON_HISTORY_STYLE,
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                },
                LevelHistoryOption { index },
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle {
                    style: PAGINATION_TEXT_STYLE,
                    text: Text {
                        sections: vec![TextSection::new(
                            format!("Level {}, Scored : {}", index + 1, level.scored),
                            get_button_text_style(&asset_server),
                        )],
                        alignment: TextAlignment::Center,
                        ..default()
                    },
                    ..default()
                });
            });
        }, pagination.get_start_index(), pagination.get_items_per_page());

        build_pagination_element(&asset_server, parent, &mut pagination);
        build_back_button(&asset_server, &mut game_timer, parent);
    });


}


pub fn build_back_button(
    asset_server: &Res<AssetServer>,
    game_timer: &mut ResMut<GameTimer>,
    parent : &mut ChildBuilder,
) {

    let text = if game_timer.timer.finished() {
        "Restart"
    } else {
        "Continue"
    };
    
    parent
        .spawn((
            ButtonBundle {
                style: BUTTON_HISTORY_STYLE,
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            ContinueButton {},
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                style: Style { ..default() },
                text: Text {
                    sections: vec![TextSection::new(
                        text,
                        get_button_text_style(&asset_server),
                    )],
                    alignment: TextAlignment::Center,
                    ..default()
                },
                ..default()
            });
        });
}

fn build_pagination_element(
    asset_server: &Res<AssetServer>,
    parent : &mut ChildBuilder,
    mut pagination: &mut ResMut<Pagination>,
) {
    parent
    .spawn(NodeBundle {
        style: PAGINATION_CONTAINER_STYLE,
        ..default()
    })
    .with_children(|parent| {
        for index in 0..pagination.max_page {
            parent
                .spawn((
                    ButtonBundle {
                        style: BUTTON_PAGINATION_STYLE,
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    PaginationOption { index },
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        style: PAGINATION_TEXT_STYLE,
                        text: Text {
                            sections: vec![TextSection::new(
                                format!("{}", index + 1),
                                get_button_text_style(&asset_server),
                            )],
                            alignment: TextAlignment::Center,
                            ..default()
                        },
                        ..default()
                    });
                });
        }
    });
}
