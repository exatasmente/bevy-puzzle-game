use bevy::prelude::*;

use bevy_prototype_lyon::prelude::ShapePlugin;
mod main_menu;
use main_menu::*;

mod systems;
use systems::*;

mod events;
use events::*;

mod game;
use game::*;


pub const PIXELS_PER_METER: f32 = 492.3;
pub const RESOLUTION: f32 = 16.0 / 9.0;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_state::<AppState>()
        // My Plugins
        .add_plugin(MainMenuPlugin)
        .add_plugin(GamePlugin)
        // Startup Systems
        .add_startup_system(spawn_camera)
        // Systems
        .add_system(transition_to_game_state)
        .add_system(transition_to_main_menu_state)
        .add_system(exit_game)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "PUZZLE".into(),
                present_mode: bevy::window::PresentMode::AutoVsync,
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugin(ShapePlugin)
        
        .register_type::<BackgroundTranstion>()
        .run();
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    Game,
    GameOver,
}
