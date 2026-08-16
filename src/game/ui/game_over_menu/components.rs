use bevy::prelude::Component;

#[derive(Component)]
pub struct GameOverMenu;

#[derive(Component)]
pub struct MainMenuButton;

#[derive(Component)]
pub struct GameOverHistoryButton;

/// The primary action on the end-of-run screen.
#[derive(Component)]
pub struct PlayAgainButton;

/// Publishes the run's numbers for the page to turn into a shareable image.
#[derive(Component)]
pub struct ShareScoreButton;
