mod game_over_menu;
mod pause_menu;

use game_over_menu::GameOverMenuPlugin;
use pause_menu::PauseMenuPlugin;

use bevy::prelude::*;

pub struct GameUIPlugin;

impl Plugin for GameUIPlugin {
    fn build(&self, app: &mut App) {
        app
            // Plugins
            .add_plugin(PauseMenuPlugin)
            .add_plugin(GameOverMenuPlugin);
    }
}
