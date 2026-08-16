mod components;
mod styles;
mod systems;

use bevy::prelude::*;

use crate::AppState;
use systems::interactions::*;
use systems::layout::*;

pub struct AchievementsMenuPlugin;

impl Plugin for AchievementsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Achievements), spawn_achievements_menu)
            .add_systems(
                Update,
                interact_with_back_button.run_if(in_state(AppState::Achievements)),
            )
            // Tears down live `Button` entities, so it runs after `Update`.
            .add_systems(
                PostUpdate,
                relayout_achievements_menu.run_if(in_state(AppState::Achievements)),
            )
            .add_systems(OnExit(AppState::Achievements), despawn_achievements_menu);
    }
}
