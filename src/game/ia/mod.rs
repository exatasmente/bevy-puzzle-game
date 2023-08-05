use bevy::prelude::*;

pub mod follow;
pub mod wander;

use crate::AppState;

use follow::*;
pub struct IaPlugin;

impl Plugin for IaPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(follow_player_system.run_if(in_state(AppState::Game)));
    }
}