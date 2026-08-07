//! Keeps the HUD in step with the run, and makes each change felt.
//!
//! Every system here follows the same shape: read the authoritative value from
//! a resource, compare it to what is currently displayed, and when it differs,
//! animate the difference rather than replacing it silently.

use bevy::prelude::*;

use crate::feedback::PopAnim;
use crate::game::puzzle::components::{ColorPuzzle, GameHistory, GameTimer};
use crate::game::ui::hud::components::*;
use crate::theme;

/// Seconds left at which the timer starts warning.
const WARNING_SECONDS: f32 = 10.0;
/// Seconds left at which it starts pulsing.
const CRITICAL_SECONDS: f32 = 5.0;

/// How fast the displayed score chases the real one, in points per second.
/// Fast enough to never lag behind play, slow enough that the eye catches the
/// movement and reads it as a gain.
const SCORE_COUNT_SPEED: f32 = 14.0;

pub fn update_score_text(
    mut commands: Commands,
    puzzle: Res<ColorPuzzle>,
    time: Res<Time>,
    mut displayed: Local<f32>,
    mut last_target: Local<usize>,
    mut query: Query<(Entity, &mut Text), With<ScoreValueText>>,
) {
    let target = puzzle.get_score();

    let Ok((entity, mut text)) = query.get_single_mut() else {
        return;
    };

    if target != *last_target {
        // Jumping down means a new run started; don't count backwards.
        if target < *last_target {
            *displayed = target as f32;
        } else {
            commands.entity(entity).insert(PopAnim::small());
        }
        *last_target = target;
    }

    // Count up toward the real score.
    let target_f32 = target as f32;
    if (*displayed - target_f32).abs() > 0.01 {
        let step = SCORE_COUNT_SPEED * time.delta_seconds();
        *displayed = if *displayed < target_f32 {
            (*displayed + step.max(0.2)).min(target_f32)
        } else {
            target_f32
        };
    }

    text.sections[0].value = format!("{}", displayed.round() as usize);
}

pub fn update_streak_text(
    mut commands: Commands,
    game_history: Res<GameHistory>,
    mut last_streak: Local<usize>,
    mut query: Query<(Entity, &mut Text), With<StreakValueText>>,
) {
    let streak = game_history.current_streak();

    let Ok((entity, mut text)) = query.get_single_mut() else {
        return;
    };

    if streak != *last_streak {
        if streak > *last_streak {
            commands.entity(entity).insert(PopAnim::small());
        }
        *last_streak = streak;
    }

    text.sections[0].value = format!("x{}", streak);
    // A live streak is lit; a broken one goes grey. The player should be able
    // to see, without reading, that they just lost something.
    text.sections[0].style.color = if streak == 0 {
        theme::MUTED
    } else {
        theme::SUCCESS
    };
}

pub fn update_timer_text(
    puzzle: Res<ColorPuzzle>,
    game_timer: Res<GameTimer>,
    time: Res<Time>,
    mut query: Query<&mut Text, With<TimerValueText>>,
) {
    let Ok(mut text) = query.get_single_mut() else {
        return;
    };

    if !puzzle.game_mode.is_timed() {
        text.sections[0].value = "--".to_string();
        text.sections[0].style.color = theme::MUTED;
        return;
    }

    let remaining = game_timer.timer.remaining_secs();
    text.sections[0].value = format!("{:02.0}", remaining);

    text.sections[0].style.color = if remaining <= CRITICAL_SECONDS {
        // Pulse: urgency the player feels before they finish reading the number.
        let pulse = (time.elapsed_seconds() * 12.0).sin() * 0.5 + 0.5;
        Color::rgb(
            theme::DANGER.r(),
            theme::DANGER.g() * (0.4 + 0.6 * pulse),
            theme::DANGER.b() * (0.4 + 0.6 * pulse),
        )
    } else if remaining <= WARNING_SECONDS {
        theme::ACCENT
    } else {
        theme::ON_SURFACE
    };
}

pub fn update_level_progress(
    puzzle: Res<ColorPuzzle>,
    mut fill_query: Query<&mut Style, With<LevelProgressFill>>,
    mut level_query: Query<&mut Text, With<LevelValueText>>,
) {
    if let Ok(mut style) = fill_query.get_single_mut() {
        style.size.width = Val::Percent(puzzle.progress_to_next_level() * 100.0);
    }

    if let Ok(mut text) = level_query.get_single_mut() {
        text.sections[0].value = match puzzle.points_to_next_level() {
            // Naming the remaining distance is what turns a bar into a goal.
            Some(points) => format!("NIVEL {}   FALTAM {}", puzzle.level(), points),
            None => format!("NIVEL {}   MAXIMO", puzzle.level()),
        };
    }
}
