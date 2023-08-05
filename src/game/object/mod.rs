pub mod components;
mod systems;
use systems::*;
use bevy::prelude::*;


pub struct ObjectPlugin;

impl Plugin for ObjectPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(object_movement.run_if(in_state(super::AppState::Game)));
    }
}
