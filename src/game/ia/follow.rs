use bevy::prelude::*;
use crate::game::player::components::Player;

#[derive(Component,Debug, Clone, Copy)]
pub struct FollowPlayer {
    pub speed: f32,
}


pub fn follow_player_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut FollowPlayer), Without<Player>>,
    player_query: Query<&Transform, (With<Player>, Without<FollowPlayer>)>,
) {
    let player_transform = player_query.single();

    for (mut transform, mut follow) in query.iter_mut() {
        
        let direction = player_transform.translation - transform.translation;
        let distance = direction.length();
        let direction = direction.normalize();

        if  distance < 10.0 {
            follow.speed = 0.0;
            continue;
        }

        if distance < 50.0 {
            follow.speed = 1.0;
        }

        if distance > 100.0 {
            continue;
        }

        let translation = &mut transform.translation;
        translation.x += direction.x * follow.speed * time.delta_seconds();
        translation.y += direction.y * follow.speed * time.delta_seconds();
    }
}