use crate::game::puzzle::components::PowerUp;
use bevy::prelude::Component;

/// Root of the in-game HUD. Everything under it is despawned together.
#[derive(Component)]
pub struct HudRoot;

/// Root of the standalone back button shown while reviewing a past level.
#[derive(Component)]
pub struct BackButtonRoot;

// `Default` so `spawn_stat` can build these generically.
#[derive(Component, Default)]
pub struct ScoreValueText;

#[derive(Component, Default)]
pub struct StreakValueText;

#[derive(Component, Default)]
pub struct TimerValueText;

#[derive(Component)]
pub struct LevelValueText;

/// Container for the life markers. Only spawned in a mode that has lives.
#[derive(Component)]
pub struct LivesRow;

/// One life marker. `index` is its place in the row counted from the left, so
/// the update system can light the first `n` of them without reordering
/// anything — a spent life keeps its slot, which is how the player can still
/// see how many they started with.
#[derive(Component)]
pub struct LivesPip {
    pub index: usize,
}

/// The filling part of the level progress bar.
#[derive(Component)]
pub struct LevelProgressFill;

/// Pause / open-history button.
#[derive(Component)]
pub struct HistoryButtom;

#[derive(Component)]
pub struct HistoryBackButtom;

/// A power-up the player can spend, one button per kind.
///
/// The count lives on the label rather than in a separate node, so a button
/// that is out reads as "0" instead of vanishing and moving its neighbour under
/// the thumb that was reaching for it.
#[derive(Component)]
pub struct PowerUpButton {
    pub kind: PowerUp,
}

/// The label inside a power-up button, updated with the count.
#[derive(Component)]
pub struct PowerUpButtonLabel {
    pub kind: PowerUp,
}

/// Container for the power-up buttons.
#[derive(Component)]
pub struct PowerUpRow;
