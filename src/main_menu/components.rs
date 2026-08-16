use crate::game::puzzle::components::GameMode;
use bevy::prelude::Component;

#[derive(Component)]
pub struct MainMenu;

#[derive(Component)]
pub struct PlayButton {
    pub game_mode: GameMode,
}

/// Resumes the run stored from a previous session.
#[derive(Component)]
pub struct ContinueRunButton {
    pub game_mode: GameMode,
    pub score: usize,
    /// Lives the stored run had left. Zero in a timed mode, which has none.
    pub lives: usize,
}
