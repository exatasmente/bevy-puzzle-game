use bevy::prelude::*;

mod systems;
use systems::*;


pub struct WasmPlugin;

impl Plugin for WasmPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(transition_to_state);
          
    }
}