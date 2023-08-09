use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use crate::AppState;
use crate::events::InteractionAnimationEvent;

pub struct InteractionAnimationPlugin;

#[derive(Component)]
pub struct InteractionAnimationTimer(Timer);

#[derive(Component)]
pub struct InteractionAnimation ;

impl Plugin for InteractionAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(animate_interaction.run_if(in_state(AppState::Game)))
        .add_system(handle_interaction_animation_events.run_if(in_state(AppState::Game)));
    }
}

pub fn handle_interaction_animation_events(
    mut commands: Commands,
    mut interaction_animation_events: EventReader<InteractionAnimationEvent>,
) {
    let event = interaction_animation_events.iter().next();

    if event.is_none() {
        return;
    }

    let event = event.unwrap();
    let shape =  shapes::Rectangle {
        origin: shapes::RectangleOrigin::Center,
        extents: Vec2::new(1.0, 1.0),
    };

    commands.spawn(( 
        ShapeBundle {
            path: GeometryBuilder::build_as(&shape),
            transform: Transform::from_xyz(
                event.x,
                event.y,
                1.0
            ),
            ..default()
        },
        Fill::color(Color::Rgba { red: 1.0, green: 1.0, blue: 1.0, alpha: 0.6 }),
        InteractionAnimationTimer(Timer::from_seconds(0.5, TimerMode::Once)),
    ));

}

pub fn animate_interaction(
    mut commands: Commands,
    mut query: Query<(Entity, &mut InteractionAnimationTimer, &mut Transform)>,
    time: Res<Time>,
) {
    for (entity, mut timer, mut transform) in query.iter_mut() {
        timer.0.tick(time.delta());
        
        if timer.0.finished() {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        transform.scale.x += 2.0 * timer.0.percent() * 0.1;
        transform.scale.y += 2.0 * timer.0.percent() * 0.1;
    
    }
}