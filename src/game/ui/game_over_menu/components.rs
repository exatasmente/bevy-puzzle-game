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
