mod components;
mod styles;
mod systems;

use systems::interactions::*;
use systems::layout::*;

use crate::AppState;
use crate::game::SimulationState;

use bevy::prelude::*;

pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // OnEnter Systems
            .add_system(spawn_pause_menu.in_schedule(OnEnter(AppState::Paused)))
            // Systems
            .add_systems(
                (
                    interact_with_resume_button,
                    interact_with_main_menu_button,
                    interact_with_quit_button,
                )
                    .in_set(OnUpdate(AppState::Paused)),
            )
            // OnExit Systems
            .add_system(despawn_pause_menu.in_schedule(OnExit(AppState::Paused)));
    }
}
