use crate::AppState;

pub struct GameOver {
    pub score: u32,
}


pub struct TransitionToStateEvent {
    pub state: AppState,
}