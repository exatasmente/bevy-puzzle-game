use bevy::prelude::*;
use crate::events::TransitionToStateEvent;
use crate::AppState;

pub fn transition_to_state(
    mut app_state_next_state: ResMut<NextState<AppState>>,
    mut events: EventReader<TransitionToStateEvent>,
) {
    let event = events.iter().next();

    if event.is_none() {
        return;
    }

    let event = event.unwrap();

    app_state_next_state.set(event.state);    

    
}