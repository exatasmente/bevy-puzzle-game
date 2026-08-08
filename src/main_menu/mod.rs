mod components;
mod styles;
mod systems;

use systems::interactions::*;
use systems::layout::*;

use bevy::prelude::*;

use crate::AppState;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // OnEnter State Systems
            .add_system(spawn_main_menu.in_schedule(OnEnter(AppState::MainMenu)))
            .add_system(reset_background.in_schedule(OnEnter(AppState::MainMenu)))
            // Systems
            .add_systems(
                (interact_with_play_button, interact_with_continue_run_button)
                    .in_set(OnUpdate(AppState::MainMenu)),
            )
            // Rebuilding tears down live `Button` entities, so it runs after
            // `Update` — see the note on `relayout_main_menu`.
            .add_system(
                relayout_main_menu
                    .in_base_set(CoreSet::PostUpdate)
                    .run_if(in_state(AppState::MainMenu)),
            )
            // OnExit State Systems
            .add_system(despawn_main_menu.in_schedule(OnExit(AppState::MainMenu)));
    }
}
