//! Keeps the HUD in step with the run, and makes each change felt.
//!
//! Every system here follows the same shape: read the authoritative value from
//! a resource, compare it to what is currently displayed, and when it differs,
//! animate the difference rather than replacing it silently.

use bevy::prelude::*;

use crate::feedback::PopAnim;
use crate::game::puzzle::components::{ColorPuzzle, GameHistory, GameTimer};
use crate::game::ui::hud::components::*;
use crate::game::ui::hud::styles::LIVES_PIP_SPENT_COLOR;
use crate::theme;

/// Seconds left at which the timer starts warning.
const WARNING_SECONDS: f32 = 10.0;
/// Seconds left at which it starts pulsing.
const CRITICAL_SECONDS: f32 = 5.0;

/// How fast the displayed score chases the real one, in points per second.
/// Fast enough to never lag behind play, slow enough that the eye catches the
/// movement and reads it as a gain.
const SCORE_COUNT_SPEED: f32 = 14.0;

/// However far the counter has to travel, it arrives within this long.
///
/// A fixed rate is right for the +1 it was written for and wrong for anything
/// larger. Resuming a saved run sets the score to wherever the player left off,
/// and at fourteen a second a run continued at 1307 spent a minute and a half
/// displaying a number that was not the score — while the level and the points
/// to the next level, which do not animate, showed the truth beside it.
const MAX_COUNT_SECONDS: f32 = 0.4;

pub fn update_score_text(
    mut commands: Commands,
    puzzle: Res<ColorPuzzle>,
    time: Res<Time>,
    mut displayed: Local<f32>,
    mut last_target: Local<usize>,
    mut query: Query<(Entity, &mut Text), With<ScoreValueText>>,
) {
    let target = puzzle.get_score();

    let Ok((entity, mut text)) = query.single_mut() else {
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
        let remaining = target_f32 - *displayed;
        let rate = SCORE_COUNT_SPEED.max(remaining / MAX_COUNT_SECONDS);
        let step = rate * time.delta_secs();
        *displayed = if *displayed < target_f32 {
            (*displayed + step.max(0.2)).min(target_f32)
        } else {
            target_f32
        };
    }

    text.0 = format!("{}", displayed.round() as usize);
}

pub fn update_streak_text(
    mut commands: Commands,
    game_history: Res<GameHistory>,
    mut last_streak: Local<usize>,
    mut query: Query<(Entity, &mut Text, &mut TextColor), With<StreakValueText>>,
) {
    let streak = game_history.current_streak();

    let Ok((entity, mut text, mut text_color)) = query.single_mut() else {
        return;
    };

    if streak != *last_streak {
        if streak > *last_streak {
            commands.entity(entity).insert(PopAnim::small());
        }
        *last_streak = streak;
    }

    text.0 = format!("x{}", streak);
    // A live streak is lit; a broken one goes grey. The player should be able
    // to see, without reading, that they just lost something.
    text_color.0 = if streak == 0 {
        theme::MUTED
    } else {
        theme::SUCCESS
    };
}

pub fn update_timer_text(
    puzzle: Res<ColorPuzzle>,
    game_timer: Res<GameTimer>,
    time: Res<Time>,
    mut query: Query<(&mut Text, &mut TextColor), With<TimerValueText>>,
) {
    let Ok((mut text, mut text_color)) = query.single_mut() else {
        return;
    };

    if !puzzle.game_mode.is_timed() {
        text.0 = "--".to_string();
        text_color.0 = theme::MUTED;
        return;
    }

    let remaining = game_timer.timer.remaining_secs();
    text.0 = format!("{:02.0}", remaining);

    text_color.0 = if remaining <= CRITICAL_SECONDS {
        // Pulse: urgency the player feels before they finish reading the number.
        let pulse = (time.elapsed_secs() * 12.0).sin() * 0.5 + 0.5;
        Color::srgb(
            theme::DANGER.to_srgba().red,
            theme::DANGER.to_srgba().green * (0.4 + 0.6 * pulse),
            theme::DANGER.to_srgba().blue * (0.4 + 0.6 * pulse),
        )
    } else if remaining <= WARNING_SECONDS {
        theme::ACCENT
    } else {
        theme::INFO
    };
}

/// Lights the first `lives` markers and dims the rest.
///
/// Deliberately not a banner. A miss already holds the board so the answer can
/// be shown, and a caption thrown across the middle of the screen would cover
/// exactly the thing the hold exists to reveal — the same reason the streak
/// messages were taken out. The last life announces itself by pulsing in the
/// HUD instead, where it is out of the board's way.
pub fn update_lives_pips(
    mut commands: Commands,
    puzzle: Res<ColorPuzzle>,
    time: Res<Time>,
    mut last_lives: Local<Option<usize>>,
    mut query: Query<(Entity, &LivesPip, &mut BackgroundColor)>,
) {
    let lives = puzzle.lives();

    // The marker that just went out is the one at the new count, counting from
    // zero. Punching it is what makes the loss register as an event rather than
    // as a square that was always that colour.
    let just_lost = match *last_lives {
        Some(previous) if lives < previous => Some(lives),
        _ => None,
    };

    if *last_lives != Some(lives) {
        *last_lives = Some(lives);
    }

    let critical = lives == 1;

    for (entity, pip, mut color) in query.iter_mut() {
        if pip.index >= lives {
            *color = LIVES_PIP_SPENT_COLOR.into();

            if just_lost == Some(pip.index) {
                commands.entity(entity).insert(PopAnim::small());
            }

            continue;
        }

        *color = if critical {
            let pulse = (time.elapsed_secs() * 12.0).sin() * 0.5 + 0.5;
            Color::srgb(
                theme::DANGER.to_srgba().red,
                theme::DANGER.to_srgba().green * (0.4 + 0.6 * pulse),
                theme::DANGER.to_srgba().blue * (0.4 + 0.6 * pulse),
            )
        } else {
            theme::DANGER
        }
        .into();
    }
}

pub fn update_level_progress(
    puzzle: Res<ColorPuzzle>,
    mut fill_query: Query<&mut Node, With<LevelProgressFill>>,
    mut level_query: Query<&mut Text, With<LevelValueText>>,
) {
    if let Ok(mut style) = fill_query.single_mut() {
        style.width = Val::Percent(puzzle.progress_to_next_level() * 100.0);
    }

    if let Ok(mut text) = level_query.single_mut() {
        // Naming the remaining distance is what turns a bar into a goal. There
        // is no last level any more, so there is no "MAXIMO" case to fall to.
        text.0 = format!(
            "NIVEL {}   FALTAM {}",
            puzzle.level(),
            puzzle.points_to_next_level()
        );
    }
}
