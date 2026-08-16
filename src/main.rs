use bevy::{prelude::*, window::PresentMode};
use bevy_prototype_lyon::prelude::ShapePlugin;
mod main_menu;
use main_menu::*;

mod systems;
use systems::*;

mod game;
use game::*;

mod events;
use events::*;

mod pagination;
use pagination::*;

mod wasm;
use wasm::*;

mod feedback;
use feedback::*;

mod interaction_animation;
use interaction_animation::*;

mod audio;
mod board;
mod layout;
mod mosaic_pattern;
mod oklab;
mod wfc;
mod storage;
mod theme;

pub const PIXELS_PER_METER: f32 = 492.3;
pub const RESOLUTION: f32 = 16.0 / 9.0;

fn main() {
    App::new()
        .init_state::<AppState>()
        .add_message::<TransitionToStateEvent>()
        .add_message::<InteractionAnimationEvent>()
        .add_message::<layout::RelayoutEvent>()
        .init_resource::<layout::LayoutWidth>()
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        // My Plugins
        .add_plugins((
            MainMenuPlugin,
            GamePlugin,
            WasmPlugin,
            FeedbackPlugin,
            audio::GameAudioPlugin,
            InteractionAnimationPlugin,
        ))

        // Startup Systems
        .add_systems(Startup, spawn_camera)
        // Systems
        .add_systems(Update, (
            transition_to_game_state,
            transition_to_main_menu_state,
            transition_to_game_over_state,
            exit_game,
            layout::track_window_width,
        ))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode : PresentMode::AutoNoVsync,
                title: "PUZZLE".into(),
                canvas : Some("#canvas".into()),
                prevent_default_event_handling: false,
                fit_canvas_to_parent: true,
                resize_constraints : WindowResizeConstraints {
                    min_width : 320.,
                    min_height : 480.,
                    max_width : 1080.,
                    max_height : 3046.,
                },
                ..default()
            }),
            ..default()
        }))
        .add_plugins(ShapePlugin)
        .run();
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    Game,
    Paused,
    LevelHistory,
    History,
    GameOverResume,
    GameOver,
}
