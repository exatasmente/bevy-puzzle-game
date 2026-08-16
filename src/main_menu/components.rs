use crate::game::puzzle::components::{GameMode, PowerUps};
use bevy::prelude::Component;

#[derive(Component)]
pub struct MainMenu;

#[derive(Component)]
pub struct PlayButton {
    pub game_mode: GameMode,
}

/// Resumes the run stored for this mode.
///
/// One mode's card carries either this or [`PlayButton`], never both: a card
/// with a stored run resumes it, and a card without one starts fresh.
#[derive(Component)]
pub struct ContinueRunButton {
    pub game_mode: GameMode,
    pub score: usize,
    /// Lives the stored run had left. Zero in a timed mode, which has none.
    pub lives: usize,
    /// Power-ups the stored run had in hand.
    pub power_ups: PowerUps,
}

/// Opens the goals screen.
#[derive(Component)]
pub struct AchievementsButton;
