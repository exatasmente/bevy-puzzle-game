use crate::game::puzzle::components::GameMode;
use bevy::prelude::Component;

#[derive(Component)]
pub struct MainMenu;

#[derive(Component)]
pub struct PlayButton {
    pub game_mode: GameMode,
}
