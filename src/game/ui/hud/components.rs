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

/// The filling part of the level progress bar.
#[derive(Component)]
pub struct LevelProgressFill;

/// Pause / open-history button.
#[derive(Component)]
pub struct HistoryButtom;

#[derive(Component)]
pub struct HistoryBackButtom;
