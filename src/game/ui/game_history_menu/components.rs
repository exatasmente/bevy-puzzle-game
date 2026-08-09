use bevy::prelude::Component;

#[derive(Component)]
pub struct LevelHistoryOption {
    pub index: usize,
}

#[derive(Component)]
pub struct PaginationOption {
    pub index: usize,
}

/// Returns to the run in progress.
#[derive(Component)]
pub struct ContinueButton;

/// Ends the run and goes to the summary.
#[derive(Component)]
pub struct EndRunButton;

#[derive(Component)]
pub struct GameHistoryMenu;

#[derive(Component)]
pub struct PaginationContainer;

/// Silences the game, or lets it speak again.
#[derive(Component)]
pub struct SoundToggleButton;
