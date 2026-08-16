use bevy::prelude::*;

use crate::game::puzzle::components::{GameMode, PowerUp, PowerUps};
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

/// Where a stored run had got to.
#[derive(Debug, Clone, Copy)]
pub struct RunProgress {
    pub game_mode: GameMode,
    pub score: usize,
    /// Lives left. Meaningless in a timed mode, which stores zero.
    pub lives: usize,
    /// Power-ups still in hand.
    pub power_ups: PowerUps,
}

/// The runs in progress, one per mode, kept across reloads.
///
/// The board is not stored, because it is generated fresh every round anyway —
/// there is no position to restore, only a place in the curve, which the score
/// is what carries. Lives and power-ups are here for the opposite reason: they
/// are the run state the score cannot be derived from, and without them the way
/// to survive a bad round would be to close the tab.
///
/// **One slot per mode, not one slot.** A single slot meant starting any other
/// mode silently threw away the run you were in the middle of, which is a
/// surprising amount of progress to lose for tapping the wrong card. Each mode
/// now keeps its own place, and only a finished run clears one — a run that
/// ended in game over has nothing left to return to.
#[derive(Resource, Debug, Default)]
pub struct SavedRun {
    runs: Vec<RunProgress>,
}

impl SavedRun {
    /// The stored run for one mode, if there is one to come back to.
    pub fn get(&self, game_mode: GameMode) -> Option<RunProgress> {
        self.runs
            .iter()
            .find(|run| run.game_mode == game_mode)
            .copied()
    }

    /// Whether any mode has something to resume. Drives the menu's wording.
    pub fn any(&self) -> bool {
        !self.runs.is_empty()
    }

    /// Records where a run has got to. A score of zero is not worth coming back
    /// to, so it clears that mode's slot instead — otherwise the menu would
    /// offer to resume a run the player never started scoring in.
    pub fn store(&mut self, game_mode: GameMode, score: usize, lives: usize, power_ups: PowerUps) {
        if score == 0 {
            self.clear(game_mode);
            return;
        }

        // A run with no lives left is already lost. There is a beat between the
        // last life going and the summary screen clearing this, and a tab
        // closed inside it should not come back as a run that cannot be played.
        if game_mode.starting_lives().is_some() && lives == 0 {
            self.clear(game_mode);
            return;
        }

        let progress = RunProgress {
            game_mode,
            score,
            lives,
            power_ups,
        };

        match self.runs.iter_mut().find(|run| run.game_mode == game_mode) {
            Some(existing) => *existing = progress,
            None => self.runs.push(progress),
        }

        self.persist();
    }

    /// Drops one mode's run. Called when that run ends, not when another starts.
    pub fn clear(&mut self, game_mode: GameMode) {
        self.runs.retain(|run| run.game_mode != game_mode);
        self.persist();
    }

    fn persist(&self) {
        storage::save(RUN_KEY, &self.serialize());
    }

    /// `mode=score:lives:life=n,cut=n` entries separated by `;`.
    ///
    /// Hand-rolled for the same reason `BestScores` is: three integers and an
    /// enum do not justify pulling serde into the wasm bundle.
    fn serialize(&self) -> String {
        self.runs
            .iter()
            .map(|run| {
                let power_ups = PowerUp::iter()
                    .map(|kind| format!("{}={}", kind.storage_key(), run.power_ups.count(kind)))
                    .collect::<Vec<_>>()
                    .join(",");

                format!(
                    "{}={}:{}:{}",
                    run.game_mode.storage_key(),
                    run.score,
                    run.lives,
                    power_ups
                )
            })
            .collect::<Vec<_>>()
            .join(";")
    }

    pub fn load() -> Self {
        let Some(raw) = storage::load(RUN_KEY) else {
            return Self::default();
        };

        // Splitting on `;` is what makes this backwards compatible for free: a
        // save written when there was only one slot is simply a one-entry list,
        // and the optional tails below cover the fields it predates.
        let runs = raw
            .split(';')
            .filter_map(|entry| Self::parse_entry(entry.trim()))
            .collect();

        Self { runs }
    }

    fn parse_entry(entry: &str) -> Option<RunProgress> {
        let (key, value) = entry.split_once('=')?;

        // `score`, `score:lives`, or `score:lives:life=n,cut=n` — each tail was
        // added later, and a save missing one is read as the friendlier of the
        // two readings rather than discarded.
        let mut parts = value.trim().split(':');
        let score = parts.next()?.trim().parse::<usize>().ok()?;
        let lives = parts.next();
        let power_ups = parts.next();

        if score == 0 {
            return None;
        }

        // An unknown mode key means the save came from a build that had a mode
        // this one does not. Dropping it beats resuming into the wrong game.
        let game_mode = GameMode::iter().find(|mode| mode.storage_key() == key.trim())?;

        let full = game_mode.starting_lives().unwrap_or(0);
        let lives = lives
            .and_then(|lives| lives.trim().parse::<usize>().ok())
            .unwrap_or(full)
            .min(full);

        // A stored run with no lives left was already lost; see `store`.
        if full > 0 && lives == 0 {
            return None;
        }

        let mut counts = PowerUps::default();
        if let Some(raw) = power_ups {
            for field in raw.split(',') {
                let Some((kind, count)) = field.split_once('=') else {
                    continue;
                };
                let Ok(count) = count.trim().parse::<usize>() else {
                    continue;
                };
                if let Some(kind) = PowerUp::iter().find(|k| k.storage_key() == kind.trim()) {
                    for _ in 0..count {
                        counts.grant(kind);
                    }
                }
            }
        }

        Some(RunProgress {
            game_mode,
            score,
            lives,
            power_ups: counts,
        })
    }
}
