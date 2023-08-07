mod game_over_menu;
mod pause_menu;
mod hud;

use game_over_menu::GameOverMenuPlugin;
use pause_menu::PauseMenuPlugin;
use hud::HudPlugin;

use bevy::prelude::*;

pub struct GameUIPlugin;

impl Plugin for GameUIPlugin {
    fn build(&self, app: &mut App) {
        app
            // Plugins
            .add_plugin(PauseMenuPlugin)
            .add_plugin(HudPlugin)
            .add_plugin(GameOverMenuPlugin);
    }
}
