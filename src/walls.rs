use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct WallsPlugin;

impl Plugin for WallsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_walls);
    }
}

#[derive(Component)]
pub struct BottomWall;

fn spawn_walls(mut commands: Commands) {
    //Spawn outer wall
    //Spawn top and bottom wall
    let shape_top_and_bottom_wall = shapes::Rectangle {
        extents: Vec2::new(
            crate::PIXELS_PER_METER * 0.73,
            crate::PIXELS_PER_METER * 0.05,
        ),
        origin: shapes::RectangleOrigin::Center,
    };

    //Spawn bottom wall
    let bottom_wall_pos = Vec2::new(0.0, 0.0);
    commands
        .spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape_top_and_bottom_wall),
                ..default()
            },
            Fill::color(Color::TEAL),
        ))
        .insert(RigidBody::KinematicPositionBased)
        .insert(Sensor)
        .insert(Collider::cuboid(
            shape_top_and_bottom_wall.extents.x / 2.0,
            shape_top_and_bottom_wall.extents.y / 2.0,
        ))
        .insert(Transform::from_xyz(
            bottom_wall_pos.x,
            bottom_wall_pos.y,
            0.0,
        ))
        .insert(BottomWall);

    //Spawn bottom wall
    let bottom_wall_pos = Vec2::new(0.0, 0.0);
    commands
        .spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape_top_and_bottom_wall),
                ..default()
            },
            Fill::color(Color::TEAL),
        ))
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::cuboid(
            shape_top_and_bottom_wall.extents.x / 2.0,
            shape_top_and_bottom_wall.extents.y / 2.0,
        ))
        .insert(Transform::from_xyz(
            bottom_wall_pos.x,
            bottom_wall_pos.y,
            0.0,
        ))
        .insert(BottomWall);

}
