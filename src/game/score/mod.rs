use bevy::prelude::*;

pub mod resources;
mod systems;

use crate::AppState;

use resources::*;
use systems::*;

/// Owns the player's personal bests.
///
/// This module existed in the tree but was never declared in `game/mod.rs`, so
/// none of it compiled. It is now the home of the comparison target that gives
/// a finished run its meaning.
pub struct ScorePlugin;

/// Marks the point at which `LastRunOutcome` is valid for this run.
///
/// The game-over screen is built on the same state transition, so it orders
/// itself after this set rather than racing it and rendering the previous run's
/// numbers.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordOutcomeSet;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastRunOutcome>()
            .init_resource::<BestScores>()
            .init_resource::<SavedRun>()
            .add_systems(Startup, load_best_scores)
            .add_systems(Update, remember_run.run_if(in_state(AppState::Game)))
            .add_systems(
                OnEnter(AppState::GameOverResume),
                record_run_outcome.in_set(RecordOutcomeSet),
            );
    }
}
