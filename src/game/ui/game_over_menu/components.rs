use bevy::prelude::Component;

#[derive(Component)]
pub struct GameOverMenu {}

#[derive(Component)]
pub struct FinalScoreText {}

#[derive(Component)]
pub struct RestartButton {}

#[derive(Component)]
pub struct MainMenuButton {}

#[derive(Component)]
pub struct QuitButton {}


#[derive(Component)]
pub struct LevelHistoryOption {
    pub index : usize,
}
