use crate::AppState;

pub struct GameOver {
    pub score: u32,
}


pub struct TransitionToStateEvent {
    pub state: AppState,
}

pub struct InteractionAnimationEvent {
    pub x: f32,
    pub y: f32,
}