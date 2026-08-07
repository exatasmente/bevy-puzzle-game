use crate::AppState;
use bevy::prelude::Vec2;

pub struct TransitionToStateEvent {
    pub state: AppState,
}

/// Fired once per pick, carrying everything the feedback layer needs to make the
/// outcome unmistakable.
///
/// The previous version carried only a position, so a correct pick and a wrong
/// pick produced identical animations — the player had to infer the result from
/// the score counter. Feedback that ambiguous cannot reinforce anything.
pub struct InteractionAnimationEvent {
    pub position: Vec2,
    pub scored: bool,
    /// Seconds granted by this pick (TimeTrial). Zero when nothing was granted.
    pub bonus_seconds: f32,
    /// Where the correct square was, so a miss can reveal the answer.
    pub correct_position: Option<Vec2>,
    /// Size of the squares, so the reveal outline matches the board.
    pub shape_size: f32,
}
