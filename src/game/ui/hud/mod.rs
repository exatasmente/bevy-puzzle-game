mod components;
mod styles;
mod systems;

use crate::game::ui::hud::systems::interactions::{interact_with_pause_button, interact_with_history_back_button};
use crate::game::ui::hud::systems::layout::{spawn_hud, despawn_hud,despawn_back_button, spawn_back_button};
use crate::AppState;
use bevy::prelude::*;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app
            // OnEnter Systems
            .add_system(spawn_hud.in_schedule(OnEnter(AppState::Game)))
            .add_system(spawn_back_button.in_schedule(OnEnter(AppState::History)))
            // Systems
            .add_system(interact_with_history_back_button.run_if(in_state(AppState::History)))
            .add_system(interact_with_pause_button.run_if(in_state(AppState::Game)))
            // OnExit Systems
            .add_system(despawn_hud.in_schedule(OnExit(AppState::Game)))
            .add_system(despawn_back_button.in_schedule(OnExit(AppState::History)));
    }
}
