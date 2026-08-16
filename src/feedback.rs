//! Reusable feedback primitives: scale punches, floating text, screen shake and
//! full-width banners.
//!
//! These exist so a reward is never delivered silently. The rules of thumb the
//! rest of the code follows:
//!
//! * every player action gets a response inside ~100ms, and a *correct* action
//!   looks nothing like an incorrect one;
//! * anything the player earns (a point, three seconds, a level) animates from
//!   where it was earned toward where it is tracked, so the gain is legible;
//! * the rarest events get the loudest treatment, so loudness keeps meaning.

use bevy::prelude::*;

use crate::theme;

pub struct FeedbackPlugin;

impl Plugin for FeedbackPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ScreenShakeEvent>()
            .add_message::<BannerEvent>()
            .add_system(animate_pop)
            .add_system(animate_floating_text)
            .add_system(handle_screen_shake_events)
            .add_system(animate_screen_shake)
            .add_system(handle_banner_events)
            .add_system(animate_banner)
            .add_system(animate_reveal_in);
    }
}

// --- Scale punch -----------------------------------------------------------

/// Briefly scales an entity up and lets it settle back. Bevy's UI layout owns
/// `Transform::translation` but leaves `scale` alone, so this works on UI nodes
/// and world entities alike.
#[derive(Component)]
pub struct PopAnim {
    timer: Timer,
    strength: f32,
}

impl PopAnim {
    pub fn new(strength: f32) -> Self {
        Self {
            timer: Timer::from_seconds(0.28, TimerMode::Once),
            strength,
        }
    }

    /// A point scored, a streak ticking up.
    pub fn small() -> Self {
        Self::new(0.25)
    }

    /// A level up, a new record.
    pub fn large() -> Self {
        Self::new(0.6)
    }
}

pub fn animate_pop(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut PopAnim, &mut Transform)>,
) {
    for (entity, mut pop, mut transform) in query.iter_mut() {
        pop.timer.tick(time.delta());

        // Ease out: all of the punch up front, settling toward 1.0.
        let remaining = 1.0 - pop.timer.percent();
        let scale = 1.0 + pop.strength * remaining * remaining;
        transform.scale = Vec3::new(scale, scale, 1.0);

        if pop.timer.finished() {
            transform.scale = Vec3::ONE;
            commands.entity(entity).remove::<PopAnim>();
        }
    }
}

// --- Floating text ---------------------------------------------------------

/// World-space text that rises and fades from where it was earned. This is what
/// makes an invisible reward (the TimeTrial `+3s`) perceptible.
#[derive(Component)]
pub struct FloatingText {
    timer: Timer,
    velocity: f32,
}

pub fn spawn_floating_text(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    value: impl Into<String>,
    color: Color,
    position: Vec2,
) {
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(value.into(), theme::text(asset_server, theme::TEXT_MD, color))
                .with_alignment(TextAlignment::Center),
            transform: Transform::from_xyz(position.x, position.y, 5.0),
            ..default()
        },
        FloatingText {
            timer: Timer::from_seconds(0.9, TimerMode::Once),
            velocity: 90.0,
        },
    ));
}

pub fn animate_floating_text(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut FloatingText, &mut Transform, &mut Text)>,
) {
    for (entity, mut floating, mut transform, mut text) in query.iter_mut() {
        floating.timer.tick(time.delta());

        transform.translation.y += floating.velocity * time.delta_secs();

        // Hold the value legible for the first half, then fade.
        let progress = floating.timer.percent();
        let alpha = if progress < 0.5 {
            1.0
        } else {
            1.0 - (progress - 0.5) / 0.5
        };
        for section in text.sections.iter_mut() {
            section.style.color.set_a(alpha);
        }

        if floating.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// --- Staggered reveal ------------------------------------------------------

/// Fades text in after a delay.
///
/// Used to bring the end-of-run stats in one line at a time. Revealing results
/// in sequence holds attention through the summary instead of dumping it all at
/// once and inviting an immediate tap-away.
#[derive(Component)]
pub struct RevealIn {
    delay: f32,
    elapsed: f32,
}

impl RevealIn {
    /// `index` is the row's position in the sequence.
    pub fn staggered(index: usize) -> Self {
        Self {
            delay: index as f32 * 0.12,
            elapsed: 0.0,
        }
    }
}

pub fn animate_reveal_in(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut RevealIn, &mut Text)>,
) {
    for (entity, mut reveal, mut text) in query.iter_mut() {
        reveal.elapsed += time.delta_secs();

        let alpha = ((reveal.elapsed - reveal.delay) / 0.22).clamp(0.0, 1.0);
        for section in text.sections.iter_mut() {
            section.style.color.set_a(alpha);
        }

        if alpha >= 1.0 {
            commands.entity(entity).remove::<RevealIn>();
        }
    }
}

// --- Screen shake ----------------------------------------------------------

pub struct ScreenShakeEvent {
    pub strength: f32,
}

impl ScreenShakeEvent {
    /// A miss. Enough to feel, not enough to hurt readability.
    pub fn miss() -> Self {
        Self { strength: 6.0 }
    }
}

/// Lives on the camera alongside `BackgroundTranstion`.
#[derive(Component, Default)]
pub struct ScreenShake {
    timer: Timer,
    strength: f32,
}

pub fn handle_screen_shake_events(
    mut events: MessageReader<ScreenShakeEvent>,
    mut query: Query<&mut ScreenShake>,
) {
    let Some(event) = events.iter().next() else {
        return;
    };

    for mut shake in query.iter_mut() {
        shake.timer = Timer::from_seconds(0.22, TimerMode::Once);
        shake.strength = event.strength;
    }
}

pub fn animate_screen_shake(
    time: Res<Time>,
    mut query: Query<(&mut ScreenShake, &mut Transform)>,
) {
    for (mut shake, mut transform) in query.iter_mut() {
        if shake.timer.finished() {
            // Snap back to a known origin: the camera never moves otherwise, and
            // `viewport_to_world_2d` hit-testing depends on this transform.
            transform.translation.x = 0.0;
            transform.translation.y = 0.0;
            continue;
        }

        shake.timer.tick(time.delta());

        let decay = 1.0 - shake.timer.percent();
        let amount = shake.strength * decay;
        transform.translation.x = (rand::random::<f32>() * 2.0 - 1.0) * amount;
        transform.translation.y = (rand::random::<f32>() * 2.0 - 1.0) * amount;
    }
}

// --- Banner ----------------------------------------------------------------

/// A short, loud, centered announcement: "NIVEL 3", "EM CHAMAS!".
pub struct BannerEvent {
    pub text: String,
    pub color: Color,
    pub size: f32,
}

impl BannerEvent {
    /// The only banner left is the level-up. The streak milestones used to send
    /// a smaller one, and they interrupted the board-reading they were meant to
    /// celebrate.
    pub fn large(text: impl Into<String>, color: Color) -> Self {
        Self {
            text: text.into(),
            color,
            size: theme::TEXT_XL,
        }
    }
}

#[derive(Component)]
pub struct Banner {
    timer: Timer,
}

pub fn handle_banner_events(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut events: MessageReader<BannerEvent>,
    existing: Query<Entity, With<Banner>>,
) {
    let Some(event) = events.iter().next() else {
        return;
    };

    // Only ever one banner on screen; a newer announcement replaces an older one.
    for entity in existing.iter() {
        commands.entity(entity).despawn_recursive();
    }

    commands
        .spawn((
            Style {
                    position_type: PositionType::Absolute,
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                // Purely decorative: no `Interaction`, so it never eats a tap.
                z_index: ZIndex::Global(50),
                ..default()
            },
            Banner {
                timer: Timer::from_seconds(1.1, TimerMode::Once),
            },
        ))
        .with_children(|parent| {
            parent.spawn(theme::wrapped_text(
                event.text.clone(),
                theme::text(&asset_server, event.size, event.color),
                theme::CONTENT_MAX_WIDTH,
            ));
        });
}

pub fn animate_banner(
    mut commands: Commands,
    time: Res<Time>,
    mut banner_query: Query<(Entity, &mut Banner, &Children)>,
    mut text_query: Query<(&mut Text, &mut Transform)>,
) {
    for (entity, mut banner, children) in banner_query.iter_mut() {
        banner.timer.tick(time.delta());

        let progress = banner.timer.percent();
        let alpha = if progress < 0.65 {
            1.0
        } else {
            1.0 - (progress - 0.65) / 0.35
        };
        // Overshoot then settle, so it arrives with force.
        let scale = 1.0 + 0.35 * (1.0 - progress).powi(3);

        for child in children.iter() {
            if let Ok((mut text, mut transform)) = text_query.get_mut(*child) {
                for section in text.sections.iter_mut() {
                    section.style.color.set_a(alpha);
                }
                transform.scale = Vec3::new(scale, scale, 1.0);
            }
        }

        if banner.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
