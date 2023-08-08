mod game_over_menu;
mod game_history_menu;
mod hud;

use game_history_menu::GameHistoryMenuPlugin;
use game_over_menu::GameOverMenuPlugin;
use hud::HudPlugin;

use bevy::prelude::*;

pub struct GameUIPlugin;

impl Plugin for GameUIPlugin {
    fn build(&self, app: &mut App) {
        app
            // Plugins
            .add_plugin(HudPlugin)
            .add_plugin(GameHistoryMenuPlugin)
            .add_plugin(GameOverMenuPlugin);
    }
}
