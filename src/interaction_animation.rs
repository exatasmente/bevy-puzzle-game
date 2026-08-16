//! World-space response to a pick.
//!
//! A correct pick and a wrong pick must be distinguishable at a glance and
//! before the player has time to wonder. Correct: an expanding green ring and a
//! floating "+1" where they tapped. Wrong: a red cross, a short screen shake,
//! and a brief outline around the square they should have picked — a miss that
//! teaches is worth more than a miss that only punishes.

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::events::InteractionAnimationEvent;
use crate::feedback::{spawn_floating_text, FloatingText, ScreenShakeEvent};
use crate::theme;
use crate::AppState;

pub struct InteractionAnimationPlugin;

#[derive(Component)]
pub struct InteractionAnimationTimer(Timer);

#[derive(Component)]
pub struct InteractionAnimation;

/// Outline shown around the correct square after a miss.
#[derive(Component)]
pub struct AnswerReveal(Timer);

impl Plugin for InteractionAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(animate_interaction.run_if(in_state(AppState::Game)))
            .add_system(fade_answer_reveal.run_if(in_state(AppState::Game)))
            .add_system(handle_interaction_animation_events.run_if(in_state(AppState::Game)))
            // These animations are driven only while playing, so leaving the
            // state mid-animation would otherwise strand them on screen frozen.
            .add_system(despawn_effects.in_schedule(OnExit(AppState::Game)));
    }
}

pub fn handle_interaction_animation_events(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut interaction_animation_events: MessageReader<InteractionAnimationEvent>,
    mut screen_shake_event_writer: MessageWriter<ScreenShakeEvent>,
) {
    let Some(event) = interaction_animation_events.iter().next() else {
        return;
    };

    if event.scored {
        spawn_success_ring(&mut commands, event.position);
        spawn_floating_text(
            &mut commands,
            &asset_server,
            "+1",
            theme::SUCCESS,
            event.position,
        );

        if event.bonus_seconds > 0.0 {
            // The time bonus used to be applied straight to the timer with no
            // visible trace, so the single most valuable reward in TimeTrial was
            // also the least noticeable one.
            spawn_floating_text(
                &mut commands,
                &asset_server,
                format!("+{:.0}s", event.bonus_seconds),
                theme::ACCENT,
                Vec2::new(event.position.x, event.position.y + 34.0),
            );
        }
    } else {
        spawn_miss_cross(&mut commands, event.position);
        screen_shake_event_writer.send(ScreenShakeEvent::miss());

        if let Some(correct_position) = event.correct_position {
            spawn_answer_reveal(&mut commands, correct_position, &event.correct_corners);
        }
    }
}

fn spawn_success_ring(commands: &mut Commands, position: Vec2) {
    let ring = shapes::Circle {
        radius: 26.0,
        center: Vec2::ZERO,
    };

    commands.spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(&ring),
            transform: Transform::from_xyz(position.x, position.y, 4.0),
            ..default()
        },
        Stroke::new(theme::SUCCESS, 5.0),
        InteractionAnimation,
        InteractionAnimationTimer(Timer::from_seconds(0.45, TimerMode::Once)),
    ));
}

fn spawn_miss_cross(commands: &mut Commands, position: Vec2) {
    // Two bars crossed at right angles, rotated 45 degrees: an X without needing
    // a glyph the display font does not have.
    for angle in [std::f32::consts::FRAC_PI_4, -std::f32::consts::FRAC_PI_4] {
        let bar = shapes::Rectangle {
            origin: shapes::RectangleOrigin::Center,
            extents: Vec2::new(40.0, 6.0),
        };

        commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&bar),
                transform: Transform {
                    translation: Vec3::new(position.x, position.y, 4.0),
                    rotation: Quat::from_rotation_z(angle),
                    ..default()
                },
                ..default()
            },
            Fill::color(theme::DANGER),
            InteractionAnimation,
            InteractionAnimationTimer(Timer::from_seconds(0.4, TimerMode::Once)),
        ));
    }
}

fn spawn_answer_reveal(commands: &mut Commands, position: Vec2, corners: &[Vec2]) {
    // The outline traces the piece itself: the board is cut into polygons of
    // different shapes, and a stand-in rectangle would point at roughly the
    // right place while telling the player the wrong thing about what they
    // missed.
    let outline = shapes::Polygon {
        points: corners.to_vec(),
        closed: true,
    };

    commands.spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(&outline),
            transform: Transform::from_xyz(position.x, position.y, 3.0),
            ..default()
        },
        Stroke::new(Color::WHITE, 4.0),
        AnswerReveal(Timer::from_seconds(0.7, TimerMode::Once)),
    ));
}

pub fn animate_interaction(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut InteractionAnimationTimer,
        &mut Transform,
        Option<&mut Stroke>,
        Option<&mut Fill>,
    )>,
    time: Res<Time>,
) {
    for (entity, mut timer, mut transform, stroke, fill) in query.iter_mut() {
        timer.0.tick(time.delta());

        if timer.0.finished() {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        let progress = timer.0.percent();

        // Expand quickly, then ease off.
        let scale = 1.0 + 1.6 * progress.sqrt();
        transform.scale = Vec3::new(scale, scale, 1.0);

        let alpha = 1.0 - progress;
        if let Some(mut stroke) = stroke {
            stroke.color.set_a(alpha);
        }
        if let Some(mut fill) = fill {
            fill.color.set_a(alpha);
        }
    }
}

/// Clears every in-flight pick effect when play stops.
pub fn despawn_effects(
    mut commands: Commands,
    effects: Query<
        Entity,
        Or<(
            With<InteractionAnimationTimer>,
            With<AnswerReveal>,
            With<FloatingText>,
        )>,
    >,
) {
    for entity in effects.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn fade_answer_reveal(
    mut commands: Commands,
    mut query: Query<(Entity, &mut AnswerReveal, &mut Stroke)>,
    time: Res<Time>,
) {
    for (entity, mut reveal, mut stroke) in query.iter_mut() {
        reveal.0.tick(time.delta());

        if reveal.0.finished() {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        // Blink rather than fade: it has to read as an annotation on the board,
        // not as another particle.
        let blink = if (reveal.0.percent() * 6.0) as u32 % 2 == 0 {
            1.0
        } else {
            0.25
        };
        stroke.color.set_a(blink);
    }
}
