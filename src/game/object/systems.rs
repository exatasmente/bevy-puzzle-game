use bevy::{prelude::*};
use bevy_prototype_lyon::prelude::*;

use super::components::Object;
use super::objects::FoodType;
use crate::game::player::components::{Player, PlayerStats};
use crate::game::object::objects::FoodBowl;
use crate::game::object::components::PlayerCanInteract;

pub fn object_movement(
    mut object_query: Query<(&mut Transform, &mut Object), With<Object>>,    
) {
    for (mut transform, mut object) in object_query.iter_mut() {
        let new_position = object.move_object(transform.translation);

        transform.translation = transform.translation.lerp(new_position, 0.1);
    }
}

pub fn object_interaction(
    mut commands: Commands,
    mut object_query: Query<(&Transform, &mut Object, Entity), (With<Object>, Without<Player>)>,
    player_query: Query<&Transform, With<Player>>,

) {
    for player_transform in player_query.iter() {
        for (object_transform,_object, object_entity) in object_query.iter_mut() {
            if object_transform.translation.distance(player_transform.translation) < 10.0 {
                commands.entity(object_entity).insert(PlayerCanInteract);
            } else {
                commands.entity(object_entity).remove::<PlayerCanInteract>();
            }
            
        }
    }
}


pub fn spawn_objects(
    mut commands: Commands,
    
) {
    let shape =  shapes::Rectangle {
        extents: Vec2::new(
            65.0,
            65.0,
        ),
        origin: shapes::RectangleOrigin::Center,
    };

    commands
        .spawn(( 
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape),
                transform: Transform::from_xyz(0.0, -300.0, 0.0),
                ..default()
            },
            Fill::color(Color::BLUE),
            Object::new(0.0, Vec2 { x: 0.0, y: 0.0 }, 0),
            FoodBowl::new(100, 100, FoodType::Solid)
        )
    );

}

pub fn interact_with_food_bowl(
    _commands: Commands,
    mut object_query: Query<(&mut Fill, &mut FoodBowl), (With<Object>, With<PlayerCanInteract>, Without<Player>)>,
    mut player_query: Query<&mut PlayerStats>,
    keyboard_input: Res<Input<KeyCode>>,
) {

    if keyboard_input.just_pressed(KeyCode::E) {
        let mut player = player_query.single_mut();

        if !player.is_hungry() && !player.is_thirsty() {
            return;
        }

        let mut total_food_eaten = 0;
        let mut is_water = false;

        for (mut fill, mut food_bowl) in object_query.iter_mut() {
            if food_bowl.is_empty() {
                continue;
            }

            is_water = false;

            if food_bowl.is_water() {
                is_water = true;

            }
            if food_bowl.get_food() < (if is_water { player.get_thirst() } else { player.get_hunger() }) {
                total_food_eaten += food_bowl.get_food();
            } else {
                total_food_eaten += if is_water { player.get_thirst() } else { player.get_hunger() };
            }

            food_bowl.remove_food(total_food_eaten);
            fill.color = Color::rgb(0.0, 10.0, 150.0);
            
            break;
        }

        if is_water {
            player.decrease_thirst(total_food_eaten);
        } else {
            player.decrease_hunger(total_food_eaten);
        }
        
    }
}