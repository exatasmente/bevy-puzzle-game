mod components;
mod styles;
mod systems;

use crate::game::ui::hud::systems::interactions::{
    interact_with_history_back_button, interact_with_pause_button,
};
use crate::game::ui::hud::systems::layout::{
    despawn_back_button, despawn_hud, spawn_back_button, spawn_hud,
};
use crate::game::ui::hud::systems::updates::{
    update_level_progress, update_score_text, update_streak_text, update_timer_text,
};
use crate::AppState;
use bevy::prelude::*;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app
            // OnEnter Systems
            .add_system(spawn_hud.in_schedule(OnEnter(AppState::Game)))
            .add_system(spawn_back_button.in_schedule(OnEnter(AppState::LevelHistory)))
            // Systems
            .add_system(interact_with_history_back_button.run_if(in_state(AppState::LevelHistory)))
            .add_systems(
                (
                    interact_with_pause_button,
                    update_score_text,
                    update_streak_text,
                    update_timer_text,
                    update_level_progress,
                )
                    .in_set(OnUpdate(AppState::Game)),
            )
            // OnExit Systems
            .add_system(despawn_hud.in_schedule(OnExit(AppState::Game)))
            .add_system(despawn_back_button.in_schedule(OnExit(AppState::LevelHistory)));
    }
}
