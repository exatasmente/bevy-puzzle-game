use bevy::prelude::*;

mod systems;
use systems::*;


pub struct WasmPlugin;

impl Plugin for WasmPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, transition_to_state);
          
    }
}