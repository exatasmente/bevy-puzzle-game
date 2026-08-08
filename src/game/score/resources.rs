use bevy::prelude::*;

use crate::game::puzzle::components::GameMode;
use crate::storage;

const STORAGE_KEY: &str = "color_puzzle.best_scores";
const RUN_KEY: &str = "color_puzzle.saved_run";

/// Best score per mode, persisted where the platform allows it.
///
/// Kept per mode on purpose: a personal best is only motivating if it is a
/// like-for-like comparison. One shared number would mean a 30-second TimeTrial
/// run competing against an unlimited Infinite one, and the player would learn
/// to ignore it.
#[derive(Resource, Debug, Default)]
pub struct BestScores {
    infinite: usize,
    against_the_clock: usize,
    time_trial: usize,
    memory: usize,
    mosaic: usize,
}

impl BestScores {
    pub fn get(&self, mode: GameMode) -> usize {
        match mode {
            GameMode::Infinite => self.infinite,
            GameMode::AgainstTheClock => self.against_the_clock,
            GameMode::TimeTrial => self.time_trial,
            GameMode::Memory => self.memory,
            GameMode::Mosaic => self.mosaic,
        }
    }

    fn set(&mut self, mode: GameMode, value: usize) {
        match mode {
            GameMode::Infinite => self.infinite = value,
            GameMode::AgainstTheClock => self.against_the_clock = value,
            GameMode::TimeTrial => self.time_trial = value,
            GameMode::Memory => self.memory = value,
            GameMode::Mosaic => self.mosaic = value,
        }
    }

    /// Records a finished run. Returns true when it beat the stored best.
    ///
    /// A first run counts as a record only if it scored at all — celebrating a
    /// zero would spend the celebration on nothing.
    pub fn submit(&mut self, mode: GameMode, score: usize) -> bool {
        if score > self.get(mode) {
            self.set(mode, score);
            self.persist();
            return score > 0;
        }

        false
    }

    fn persist(&self) {
        storage::save(STORAGE_KEY, &self.serialize());
    }

    pub fn load() -> Self {
        storage::load(STORAGE_KEY)
            .map(|raw| Self::deserialize(&raw))
            .unwrap_or_default()
    }

    /// `mode=score` pairs separated by `;`. Hand-rolled to avoid pulling serde
    /// in for three integers.
    fn serialize(&self) -> String {
        GameMode::iter()
            .map(|mode| format!("{}={}", mode.storage_key(), self.get(mode)))
            .collect::<Vec<_>>()
            .join(";")
    }

    fn deserialize(raw: &str) -> Self {
        let mut scores = Self::default();

        for entry in raw.split(';') {
            let Some((key, value)) = entry.split_once('=') else {
                continue;
            };
            let Ok(value) = value.trim().parse::<usize>() else {
                continue;
            };

            // Unknown keys are ignored rather than fatal, so a future mode can
            // be added without wiping what is already stored.
            if let Some(mode) = GameMode::iter().find(|mode| mode.storage_key() == key.trim()) {
                scores.set(mode, value);
            }
        }

        scores
    }
}

/// Result of the run that just ended, handed to the game-over screen.
#[derive(Resource, Debug, Default)]
pub struct LastRunOutcome {
    pub score: usize,
    pub best: usize,
    pub is_record: bool,
}

/// A run in progress, kept across reloads.
///
/// Only the mode and the score are stored, because the score is what the level
/// is derived from and the board is generated fresh every round anyway — there
/// is no position to restore, only a place in the curve. Storing the whole
/// `GameHistory` would persist a list of past rounds nobody comes back for.
#[derive(Resource, Debug, Default)]
pub struct SavedRun {
    run: Option<(GameMode, usize)>,
}

impl SavedRun {
    pub fn get(&self) -> Option<(GameMode, usize)> {
        self.run
    }

    /// Records where the run has got to. A score of zero is not worth coming
    /// back to, so it clears instead — otherwise the menu would offer to resume
    /// a run the player never started scoring in.
    pub fn store(&mut self, mode: GameMode, score: usize) {
        if score == 0 {
            self.clear();
            return;
        }

        self.run = Some((mode, score));
        storage::save(RUN_KEY, &format!("{}={}", mode.storage_key(), score));
    }

    pub fn clear(&mut self) {
        self.run = None;
        storage::save(RUN_KEY, "");
    }

    pub fn load() -> Self {
        let Some(raw) = storage::load(RUN_KEY) else {
            return Self::default();
        };

        let Some((key, value)) = raw.split_once('=') else {
            return Self::default();
        };

        let Ok(score) = value.trim().parse::<usize>() else {
            return Self::default();
        };

        // An unknown mode key means the save came from a build that had a mode
        // this one does not. Dropping it beats resuming into the wrong game.
        let run = GameMode::iter()
            .find(|mode| mode.storage_key() == key.trim())
            .filter(|_| score > 0)
            .map(|mode| (mode, score));

        Self { run }
    }
}
