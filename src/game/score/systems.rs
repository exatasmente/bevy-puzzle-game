use bevy::prelude::*;

use super::resources::*;
use crate::game::puzzle::components::GameHistory;

/// Populates the already-initialised resource rather than inserting it, so no
/// system can observe a frame where `BestScores` does not exist yet.
pub fn load_best_scores(mut best_scores: ResMut<BestScores>) {
    *best_scores = BestScores::load();
}

/// Called once as a run ends. Stores the result and works out whether it was a
/// personal best, so the game-over screen can lead with that.
pub fn record_run_outcome(
    game_history: Res<GameHistory>,
    mut best_scores: ResMut<BestScores>,
    mut outcome: ResMut<LastRunOutcome>,
) {
    let score = game_history.total_score;
    let mode = game_history.game_mode;

    let is_record = best_scores.submit(mode, score);

    outcome.score = score;
    outcome.best = best_scores.get(mode);
    outcome.is_record = is_record;
}
