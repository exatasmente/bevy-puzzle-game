use bevy::prelude::*;

use super::resources::*;
use crate::game::puzzle::components::{ColorPuzzle, GameHistory};

/// Populates the already-initialised resource rather than inserting it, so no
/// system can observe a frame where `BestScores` does not exist yet.
pub fn load_best_scores(mut best_scores: ResMut<BestScores>, mut saved_run: ResMut<SavedRun>) {
    *best_scores = BestScores::load();
    *saved_run = SavedRun::load();
}

/// Keeps the stored run in step with the one being played.
///
/// Written when the run's state changes rather than when the player leaves,
/// because the way a browser game ends is usually a closed tab: there is no
/// exit to hook. Lives are watched alongside the score, since a miss moves one
/// and not the other — keyed on the score alone, a run resumed after a bad
/// round would come back with the lives it had before it.
pub fn remember_run(
    puzzle: Res<ColorPuzzle>,
    mut saved_run: ResMut<SavedRun>,
    mut last: Local<Option<(usize, usize)>>,
) {
    let progress = (puzzle.get_score(), puzzle.lives());

    if *last == Some(progress) {
        return;
    }

    *last = Some(progress);
    saved_run.store(puzzle.game_mode, progress.0, progress.1);
}

/// Called once as a run ends. Stores the result and works out whether it was a
/// personal best, so the game-over screen can lead with that.
pub fn record_run_outcome(
    game_history: Res<GameHistory>,
    mut best_scores: ResMut<BestScores>,
    mut outcome: ResMut<LastRunOutcome>,
    mut saved_run: ResMut<SavedRun>,
) {
    // The run is over, so there is nothing left to come back to.
    saved_run.clear();

    let score = game_history.total_score;
    let mode = game_history.game_mode;

    let is_record = best_scores.submit(mode, score);

    outcome.score = score;
    outcome.best = best_scores.get(mode);
    outcome.is_record = is_record;
}
