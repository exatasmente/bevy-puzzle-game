use bevy::prelude::Component;


#[derive(Component)]
pub struct LevelHistoryOption {
    pub index : usize,
}

#[derive(Component)]
pub struct PaginationOption {
    pub index: usize,
}

#[derive(Component)]
pub struct ContinueButton;

#[derive(Component)]
pub struct GameHistoryMenu;