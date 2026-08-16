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
    update_level_progress, update_lives_pips, update_score_text, update_streak_text,
    update_timer_text,
};
use crate::AppState;
use bevy::prelude::*;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app
            // OnEnter Systems
            .add_systems(OnEnter(AppState::Game), spawn_hud)
            .add_systems(OnEnter(AppState::LevelHistory), spawn_back_button)
            // Systems
            .add_systems(
                Update,
                interact_with_history_back_button.run_if(in_state(AppState::LevelHistory)),
            )
            .add_systems(
                Update,
                (
                    interact_with_pause_button,
                    update_score_text,
                    update_streak_text,
                    update_timer_text,
                    update_lives_pips,
                    update_level_progress,
                )
                    .run_if(in_state(AppState::Game)),
            )
            // OnExit Systems
            .add_systems(OnExit(AppState::Game), despawn_hud)
            .add_systems(OnExit(AppState::LevelHistory), despawn_back_button);
    }
}
