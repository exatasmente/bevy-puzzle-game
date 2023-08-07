use bevy::prelude::*;

use crate::game::ui::hud::components::*;
use crate::game::ui::hud::styles::*;

pub fn spawn_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    build_hud(&mut commands, &asset_server);
}

pub fn build_hud(commands: &mut Commands, asset_server: &Res<AssetServer>) -> Entity {
    let hud_entity = commands
        .spawn((
            ButtonBundle {
                style: BUTTON_STYLE,
                background_color: TRANSPARENT_BUTTON.into(),
                ..default()
            },
            HistoryButtom,
            HUD {},
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text {
                    sections: vec![TextSection::new(
                        "||",
                        get_text_style(&asset_server),
                    )],
                    alignment: TextAlignment::Center,
                    ..default()
                },
                ..default()
            });
        })
        .id();

    hud_entity
}

pub fn spawn_back_button(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            ButtonBundle {
                style: BUTTON_STYLE,
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            HistoryBackButtom,
            HUD {},
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                style: Style { ..default() },
                text: Text {
                    sections: vec![TextSection::new(
                        "Voltar",
                        get_text_style(&asset_server),
                    )],
                    alignment: TextAlignment::Center,
                    ..default()
                },
                ..default()
            });
        });
}

pub fn despawn_hud(mut commands: Commands, hud_query: Query<Entity, With<HUD>>) {
    for entity in hud_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn despawn_back_button(mut commands: Commands, hud_query: Query<Entity, With<HUD>>) {
    for entity in hud_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
