use bevy::prelude::*;
use bevy::core_pipeline::bloom::BloomSettings;
use bevy::window::PrimaryWindow;
use bevy_utils::Duration;

use super::components::Player;
use super::components::PlayerBundle;
use super::components::PlayerStats;

use crate::animation::AnimationController;
use crate::player::components::PlayerState;
use crate::animation::AnimationBundle;
pub const PLAYER_SIZE: f32 = 60.0; // This is the player sprite size.

pub fn spawn_player(
    mut commands: Commands,
    mut camera_query: Query<Entity, With<Camera>>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("cat_tileset.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(90.0, 90.0), 4, 8, None, Some(Vec2::new(0.0, 0.0)));
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let entity = commands.spawn((PlayerBundle::new(AnimationBundle::new(
        texture_atlas_handle.clone(),
        Transform::from_translation(Vec3::new(0.0,0.0, 0.0)),
        (0..4).collect(),
        Timer::new(Duration::from_secs_f32(0.19), TimerMode::Repeating),
        Some(0)
    ), 0.3))).id();

    let camera = camera_query.single_mut();
    commands.entity(camera).despawn();

    let camera_bundle = Camera2dBundle {
        camera: Camera {
            hdr: true,
            ..default()
        },
        ..default()
    };
    
    commands.spawn((camera_bundle, BloomSettings {
        ..BloomSettings::OLD_SCHOOL
    }, crate::movement::Follow::new(entity, 0.5, true, 1.0)));


}

pub fn despawn_player(mut commands: Commands, player_query: Query<Entity, With<Player>>) {
    if let Ok(player_entity) = player_query.get_single() {
        commands.entity(player_entity).despawn();
    }
}

pub fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<(&mut Transform, &mut AnimationController, &mut Player), With<Player>>,
    time: Res<Time>,
) {

    let (mut transform, mut animation_controller, mut player) = if player_query.is_empty() {
        return;
    } else {
        player_query.single_mut()
    };

    let mut direction = Vec3::ZERO;
    let mut new_state = player.current_state;

    if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
        transform.translation.x -= player.get_speed();
        new_state = PlayerState::WalkLeft;
    } else if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
        transform.translation.x += player.get_speed();
        new_state = PlayerState::WalkRight;
    } else if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
        transform.translation.y += player.get_speed();
        new_state = PlayerState::WalkBackward;
    } else if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S) {
        transform.translation.y -= player.get_speed();
        new_state = PlayerState::WalkForward;
    } else if keyboard_input.just_released(KeyCode::A) || keyboard_input.just_released(KeyCode::D) || keyboard_input.just_released(KeyCode::W) || keyboard_input.just_released(KeyCode::S) {
        new_state = PlayerState::Idle;
    }

    if player.current_state != new_state {
        player.set_state(new_state);
        animation_controller.set_frame_indices(player.get_state_frames());
    }
}

pub fn confine_player_movement(
    mut player_query: Query<&mut Transform, With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(mut player_transform) = player_query.get_single_mut() {
        let window = window_query.get_single().unwrap();

        let half_player_size = PLAYER_SIZE / 2.0; // 32.0
        let x_min = 0.0 + half_player_size;
        let x_max = window.width() - half_player_size;
        let y_min = 0.0 + half_player_size;
        let y_max = window.height() - half_player_size;

        let mut translation = player_transform.translation;

        // Bound the player x position
        if translation.x < x_min {
            translation.x = x_min;
        } else if translation.x > x_max {
            translation.x = x_max;
        }
        // Bound the players y position.
        if translation.y < y_min {
            translation.y = y_min;
        } else if translation.y > y_max {
            translation.y = y_max;
        }

        player_transform.translation = translation;
    }
}

pub fn update_player_tirsty(
    mut player_query: Query<(&mut PlayerStats, &mut Player), With<Player>>,
    time: Res<Time>,
) {
    if let Ok((mut player_stats, mut player)) = player_query.get_single_mut() {
        player_stats.stats_timer.tick(time.delta());

        if player_stats.stats_timer.finished() {
            player_stats.stats_timer.reset();
            player_stats.increase_thirst(1);
        }
        
    }
}