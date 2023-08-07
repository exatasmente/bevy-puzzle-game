use bevy::prelude::Component;
use crate::game::puzzle::components::GameMode;

#[derive(Component)]
pub struct MainMenu {}

#[derive(Component)]
pub struct PlayButton {
    pub game_mode: GameMode,
}

#[derive(Component)]
pub struct QuitButton {}
