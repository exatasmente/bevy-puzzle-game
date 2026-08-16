//! Goals the player can reach, and the record of which ones they have.
//!
//! Every goal is a *threshold on something the game already counts* — a streak,
//! a level, a personal best, the set of modes played. Nothing here asks the
//! player to do anything they would not otherwise be doing, which is the point:
//! a goal that needs its own behaviour is a second game bolted to the side of
//! this one, and this one is about looking carefully at colours.
//!
//! They unlock during play rather than at the end of a run, so the
//! announcement lands while the player is still in the thing they did.

use bevy::prelude::*;

use crate::feedback::BannerEvent;
use crate::game::puzzle::components::{ColorPuzzle, GameHistory, GameMode};
use crate::game::score::resources::BestScores;
use crate::storage;

const STORAGE_KEY: &str = "color_puzzle.achievements";
const MODES_KEY: &str = "color_puzzle.modes_played";

/// One goal.
///
/// A flat list rather than tiers-with-levels: the tiers are already legible
/// from the numbers, and a nested model would need a screen to explain itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Achievement {
    FirstPoint,
    Streak5,
    Streak10,
    Streak25,
    Level5,
    Level10,
    Level25,
    Score100,
    Score500,
    AllModes,
    RecordEveryMode,
}

impl Achievement {
    pub fn iter() -> impl Iterator<Item = Achievement> {
        [
            Achievement::FirstPoint,
            Achievement::Streak5,
            Achievement::Streak10,
            Achievement::Streak25,
            Achievement::Level5,
            Achievement::Level10,
            Achievement::Level25,
            Achievement::Score100,
            Achievement::Score500,
            Achievement::AllModes,
            Achievement::RecordEveryMode,
        ]
        .into_iter()
    }

    /// Stable key for storage. Never change these without migrating.
    pub fn storage_key(&self) -> &'static str {
        match self {
            Achievement::FirstPoint => "first_point",
            Achievement::Streak5 => "streak_5",
            Achievement::Streak10 => "streak_10",
            Achievement::Streak25 => "streak_25",
            Achievement::Level5 => "level_5",
            Achievement::Level10 => "level_10",
            Achievement::Level25 => "level_25",
            Achievement::Score100 => "score_100",
            Achievement::Score500 => "score_500",
            Achievement::AllModes => "all_modes",
            Achievement::RecordEveryMode => "record_every_mode",
        }
    }

    /// ASCII only, like every other user-facing string here: the display font
    /// draws accents as blanks.
    pub fn title(&self) -> &'static str {
        match self {
            Achievement::FirstPoint => "PRIMEIRO ACERTO",
            Achievement::Streak5 => "SEQUENCIA DE 5",
            Achievement::Streak10 => "SEQUENCIA DE 10",
            Achievement::Streak25 => "SEQUENCIA DE 25",
            Achievement::Level5 => "NIVEL 5",
            Achievement::Level10 => "NIVEL 10",
            Achievement::Level25 => "NIVEL 25",
            Achievement::Score100 => "100 PONTOS",
            Achievement::Score500 => "500 PONTOS",
            Achievement::AllModes => "TODOS OS MODOS",
            Achievement::RecordEveryMode => "RECORDE EM TODOS",
        }
    }

    /// What it takes, said plainly. The list is also the tutorial for what the
    /// game rewards.
    pub fn description(&self) -> &'static str {
        match self {
            Achievement::FirstPoint => "Acerte uma vez.",
            Achievement::Streak5 => "5 acertos seguidos.",
            Achievement::Streak10 => "10 acertos seguidos.",
            Achievement::Streak25 => "25 acertos seguidos.",
            Achievement::Level5 => "Chegue ao nivel 5.",
            Achievement::Level10 => "Chegue ao nivel 10.",
            Achievement::Level25 => "Chegue ao nivel 25.",
            Achievement::Score100 => "100 pontos numa partida.",
            Achievement::Score500 => "500 pontos numa partida.",
            Achievement::AllModes => "Jogue os cinco modos.",
            Achievement::RecordEveryMode => "Pontue em todos os modos.",
        }
    }

    /// The colour it wears once unlocked.
    pub fn accent(&self) -> Color {
        match self {
            Achievement::FirstPoint | Achievement::Score100 | Achievement::Score500 => {
                crate::theme::PRIMARY
            }
            Achievement::Streak5 | Achievement::Streak10 | Achievement::Streak25 => {
                crate::theme::SUCCESS
            }
            Achievement::Level5 | Achievement::Level10 | Achievement::Level25 => {
                crate::theme::LIME
            }
            Achievement::AllModes | Achievement::RecordEveryMode => crate::theme::ACCENT,
        }
    }
}

/// Which goals are reached, and which modes have been played at all.
///
/// The modes are tracked here rather than derived from `BestScores`, because a
/// mode can be played without scoring in it, and "jogue os cinco modos" should
/// mean playing them.
#[derive(Resource, Debug, Default)]
pub struct Achievements {
    unlocked: Vec<Achievement>,
    modes_played: Vec<GameMode>,
}

impl Achievements {
    pub fn has(&self, achievement: Achievement) -> bool {
        self.unlocked.contains(&achievement)
    }

    pub fn unlocked_count(&self) -> usize {
        self.unlocked.len()
    }

    pub fn total() -> usize {
        Achievement::iter().count()
    }

    /// Records a goal. False when it was already held, so the caller only
    /// announces the ones that are new.
    fn unlock(&mut self, achievement: Achievement) -> bool {
        if self.has(achievement) {
            return false;
        }

        self.unlocked.push(achievement);
        self.persist();
        true
    }

    pub fn note_mode_played(&mut self, mode: GameMode) {
        if self.modes_played.contains(&mode) {
            return;
        }

        self.modes_played.push(mode);
        self.persist();
    }

    fn modes_played_count(&self) -> usize {
        self.modes_played.len()
    }

    fn persist(&self) {
        storage::save(
            STORAGE_KEY,
            &self
                .unlocked
                .iter()
                .map(|a| a.storage_key())
                .collect::<Vec<_>>()
                .join(","),
        );
        storage::save(
            MODES_KEY,
            &self
                .modes_played
                .iter()
                .map(|m| m.storage_key())
                .collect::<Vec<_>>()
                .join(","),
        );
    }

    pub fn load() -> Self {
        // Unknown keys are skipped rather than fatal, so a build that drops a
        // goal does not wipe the rest of the record.
        let unlocked = storage::load(STORAGE_KEY)
            .map(|raw| {
                raw.split(',')
                    .filter_map(|key| {
                        Achievement::iter().find(|a| a.storage_key() == key.trim())
                    })
                    .collect()
            })
            .unwrap_or_default();

        let modes_played = storage::load(MODES_KEY)
            .map(|raw| {
                raw.split(',')
                    .filter_map(|key| GameMode::iter().find(|m| m.storage_key() == key.trim()))
                    .collect()
            })
            .unwrap_or_default();

        Self {
            unlocked,
            modes_played,
        }
    }
}

/// Loads the record at startup, populating the already-initialised resource so
/// no system can observe a frame where it does not exist.
pub fn load_achievements(mut achievements: ResMut<Achievements>) {
    *achievements = Achievements::load();
}

/// Notes the mode as played when a round starts.
pub fn note_mode_played(puzzle: Res<ColorPuzzle>, mut achievements: ResMut<Achievements>) {
    achievements.note_mode_played(puzzle.game_mode);
}

/// Checks every goal against the run in progress.
///
/// Runs each frame and does nothing on almost all of them: the checks are
/// integer comparisons against values already in memory, and the alternative —
/// hooking each one to the event that could move it — spreads the same rules
/// across four systems that then have to agree with each other.
pub fn check_achievements(
    puzzle: Res<ColorPuzzle>,
    game_history: Res<GameHistory>,
    best_scores: Res<BestScores>,
    mut achievements: ResMut<Achievements>,
    mut banner: MessageWriter<BannerEvent>,
) {
    let score = puzzle.get_score();
    let level = puzzle.level();
    let streak = game_history.current_streak();

    let reached = [
        (Achievement::FirstPoint, score >= 1),
        (Achievement::Streak5, streak >= 5),
        (Achievement::Streak10, streak >= 10),
        (Achievement::Streak25, streak >= 25),
        (Achievement::Level5, level >= 5),
        (Achievement::Level10, level >= 10),
        (Achievement::Level25, level >= 25),
        (Achievement::Score100, score >= 100),
        (Achievement::Score500, score >= 500),
        (
            Achievement::AllModes,
            achievements.modes_played_count() >= GameMode::iter().count(),
        ),
        (
            Achievement::RecordEveryMode,
            GameMode::iter().all(|mode| best_scores.get(mode) > 0),
        ),
    ];

    // Only the first new one per frame gets announced. Several can come true
    // together — level 5 and a streak of 5 often land on the same pick — and
    // `handle_banner_events` keeps only the newest anyway, so sending three
    // would show one and silently drop two.
    for (achievement, condition) in reached {
        if condition && achievements.unlock(achievement) {
            banner.write(BannerEvent::achievement(achievement.title()));
            return;
        }
    }
}
