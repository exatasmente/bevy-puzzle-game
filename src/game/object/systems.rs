use bevy::prelude::*;
use super::components::Object;

pub fn object_movement(
    mut object_query: Query<(&mut Transform, &mut Object), With<Object>>,    
) {
    for (mut transform, mut object) in object_query.iter_mut() {
        let new_position = object.move_object(transform.translation);

        transform.translation = transform.translation.lerp(new_position, 0.1);
    }
}
