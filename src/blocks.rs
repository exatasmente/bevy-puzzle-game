
use bevy_inspector_egui::InspectorOptions;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::walls::BottomWall;
pub struct BlocksPlugin;

impl Plugin for BlocksPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(steup_game)
        .add_system(handle_block_intersections_with_bottom_wall)
        .add_system(camera_follow)
        .add_system(block_moviment);
    }
}

#[derive(Component)]
struct Block {
    start_point: Vec2,
    on_ground : bool,
}

#[derive(Component, InspectorOptions)]
struct GameState {
    game_over: bool,
    n_of_blocks: u32,
    score: u32,
    start_point: f32
}

fn steup_game(mut commands: Commands) {
    commands.spawn(GameState {
        game_over : false,
        n_of_blocks : 0,
        score : 0, 
        start_point: crate::PIXELS_PER_METER * 0.50,
    });

    spawn_next_block(commands, crate::PIXELS_PER_METER * 0.50);
}

fn block_moviment(
    keyboard: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut query: Query<(&mut Transform, &Block),With<Block>>,
    time: Res<Time>,
) {    

    for (mut transform, block) in query.iter_mut() {
        if !block.on_ground {
            if keyboard.pressed(KeyCode::A) {
                transform.translation.x -= 100.0 * time.delta_seconds();
            } 
        
            if keyboard.pressed(KeyCode::D) {
                transform.translation.x += 100.0 * time.delta_seconds();
            }
            break;
        }
    }

    
}


fn spawn_next_block(mut commands: Commands, start_point: f32) {
    let shape = shapes::Rectangle {
        extents: Vec2::new(
            crate::PIXELS_PER_METER * 0.09,
            crate::PIXELS_PER_METER * 0.09,
        ),
        origin: shapes::RectangleOrigin::Center,
    };

    let block_pos = Vec2::new(
        0.0,
        start_point,
    );

    commands
        .spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape),
                ..default()
            },
            Fill::color(Color::BLACK),
            Stroke::new(Color::TEAL, 2.0),
        ))
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(
            shape.extents.x / 2.0,
            shape.extents.y / 2.0,
        ))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(Restitution::coefficient(0.01))
        .insert(Transform::from_xyz(block_pos.x, block_pos.y, 0.0))
        .insert(Block {
            start_point: block_pos,
            on_ground: false,
        });
}
fn camera_follow(
    query: Query<(&Transform, &Block), With<Block>>,
    mut camera_query: Query<&mut Transform, (Without<Block>, With<Camera>)>,
) {
    for (mut transform, block) in query.iter() {
        if !block.on_ground {
            let mut camera_transform = camera_query.single_mut();
            camera_transform.translation.x = transform.translation.x;
            camera_transform.translation.y = transform.translation.y;
        }
    }
}


fn handle_block_intersections_with_bottom_wall(
    rapier_context: Res<RapierContext>,
    mut query_state: Query<&mut GameState>,
    mut query: Query<(Entity, &mut Block, &Transform), With<Block>>,
    query_bottom_wall: Query<Entity, With<crate::BottomWall>>,
    mut commands: Commands,
) {
    let mut should_spawn_ball = false;
    let shape = shapes::Rectangle {
        extents: Vec2::new(
            crate::PIXELS_PER_METER * 0.09,
            crate::PIXELS_PER_METER * 0.0001,
        ),
        origin: shapes::RectangleOrigin::Center,
    };

    let mut game_state = query_state.single_mut();
    let mut last_block_start_point = game_state.start_point;
    for entity_bottom_wall in query_bottom_wall.iter() {
        for (entity_ball, mut block, transform) in query.iter_mut() {
            if !block.on_ground && rapier_context.intersection_pair(entity_bottom_wall, entity_ball) == Some(true) {
                commands.entity(entity_ball)
                    .remove::<Block>()
                    .remove::<RigidBody>()
                    .remove::<Collider>();

                commands
                    .spawn((
                        ShapeBundle {
                            path: GeometryBuilder::build_as(&shape),
                            ..default()
                        },
                        Fill::color(Color::NONE),
                        Stroke::new(Color::NONE, 1.0),
                    ))
                    .insert(Sensor)
                    .insert(BottomWall)
                    .insert(Collider::cuboid(
                        shape.extents.x / 2.0,
                        shape.extents.y / 2.0,
                    ))
                    .insert(Transform::from_xyz(transform.translation.x, transform.translation.y + (crate::PIXELS_PER_METER * 0.05) , 0.0));
                    last_block_start_point = transform.translation.y;
                block.on_ground = true;
                should_spawn_ball = true;
            }
        }
    }

    if should_spawn_ball {
        game_state.score += 1;
        game_state.n_of_blocks += 1;
        if last_block_start_point < game_state.start_point {
            last_block_start_point = game_state.start_point;
        }
        game_state.start_point = last_block_start_point + (crate::PIXELS_PER_METER * 0.10);
        spawn_next_block(commands, game_state.start_point);
    }

}