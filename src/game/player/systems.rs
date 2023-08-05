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
pub const PLAYER_SIZE: f32 = 88.0; // This is the player sprite size.

pub fn spawn_player(
    mut commands: Commands,
    mut camera_query: Query<Entity, With<Camera>>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("cat_tileset.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(88.0, 90.0), 4, 8, None, Some(Vec2::new(0.0, 0.0)));
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let entity = commands.spawn(PlayerBundle::new(AnimationBundle::new(
        texture_atlas_handle.clone(),
        Transform::from_translation(Vec3::new(64.0,64.0, 1.0)),
        (0..4).collect(),
        Timer::new(Duration::from_secs_f32(0.25), TimerMode::Repeating),
        Some(0),
        vec![]
    ), 0.7)).id();

    let camera = camera_query.single_mut();
    commands.entity(camera).despawn();

    let camera_bundle = Camera2dBundle::default();
        
    commands.spawn((camera_bundle, crate::movement::Follow::new(entity, 0.5, true, 1.0)));


}

pub fn despawn_player(mut commands: Commands, player_query: Query<Entity, With<Player>>) {
    if let Ok(player_entity) = player_query.get_single() {
        commands.entity(player_entity).despawn();
    }
}

pub fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<(&mut Transform, &mut AnimationController, &mut Player), With<Player>>,
    _time: Res<Time>,
) {

    let (mut transform, mut animation_controller, mut player) = if player_query.is_empty() {
        return;
    } else {
        player_query.single_mut()
    };

    let _direction = Vec3::ZERO;
    let mut new_state = player.current_state;

    let mut new_x = transform.translation.x;
    let mut new_y = transform.translation.y;

    let mut update_transform = false;
    let mut x_offset = 0.0;
    let mut y_offset = 0.0;

    if keyboard_input.pressed(KeyCode::LShift) {
        player.speed = 2.0;
    } else {
        player.speed = 1.0;
    }

    if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
        new_x = transform.translation.x - player.get_speed();
        new_state = PlayerState::WalkLeft;
        update_transform = true;
    } else if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
        new_x = transform.translation.x + player.get_speed();
        new_state = PlayerState::WalkRight;
        update_transform = true;
    } else if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
        new_y = transform.translation.y + player.get_speed();
        new_state = PlayerState::WalkBackward;
        update_transform = true;
    } else if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S) {
        new_y = transform.translation.y - player.get_speed();
        update_transform = true;
        new_state = PlayerState::WalkForward;
    } else if keyboard_input.just_released(KeyCode::A) || keyboard_input.just_released(KeyCode::D) || keyboard_input.just_released(KeyCode::W) || keyboard_input.just_released(KeyCode::S) {
        new_state = PlayerState::Idle;

    }

    if player.current_state != new_state {
        player.set_state(new_state);
        animation_controller.set_frame_indices(player.get_state_frames());
    }

    

    

}


pub fn update_player_tirsty(
    mut player_query: Query<(&mut PlayerStats, &mut Player), With<Player>>,
    time: Res<Time>,
) {
    if let Ok((mut player_stats, _player)) = player_query.get_single_mut() {
        player_stats.stats_timer.tick(time.delta());

        if player_stats.stats_timer.finished(){
            player_stats.stats_timer.reset();
            player_stats.increase_thirst(10);
            player_stats.increase_hunger(10);
        }
        
    }
}