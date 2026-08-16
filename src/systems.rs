use bevy::app::AppExit;
use bevy::prelude::*;
use crate::AppState;


/// The ground's journey through the round's colors.
///
/// Not a fade from one color to another: a *sweep* that stops at every color on
/// the board and ends on the answer's. As it passes a color, every piece
/// wearing that color melts into the ground for a moment; when it lands, the
/// answer melts and does not come back.
///
/// That sweep is the round's second channel of information, and the reason the
/// board can be a regular lattice full of deliberate holes without the round
/// becoming a lottery: the player who watched knows which hole appeared last.
#[derive(Component, Debug, Reflect)]
pub struct BackgroundTranstion {
    /// Where the ground starts, then every color it visits. The last entry is
    /// the answer's color.
    path: Vec<Color>,
    time: f32,
    current_time: f32,
}

impl Default for BackgroundTranstion {
    fn default() -> Self {
        Self {
            path: vec![Color::srgb(0.0, 0.0, 0.0)],
            time: 1.0,
            current_time: 1.0,
        }
    }
}

/// Componentwise lerp between two colors.
pub fn lerp_color(from: Color, to: Color, amount: f32) -> Color {
    let amount = amount.clamp(0.0, 1.0);
    Color::srgb(
        from.to_srgba().red + (to.to_srgba().red - from.to_srgba().red) * amount,
        from.to_srgba().green + (to.to_srgba().green - from.to_srgba().green) * amount,
        from.to_srgba().blue + (to.to_srgba().blue - from.to_srgba().blue) * amount,
    )
}

impl BackgroundTranstion {
    pub fn is_in_transition(&self) -> bool {
        self.current_time < self.time
    }

    /// Starts a sweep from the ground's current color through `stops`, which
    /// must end on the color the ground is to settle at.
    pub fn sweep(&mut self, from: Color, stops: Vec<Color>, seconds: f32) {
        self.path = std::iter::once(from).chain(stops).collect();
        self.time = seconds.max(0.001);
        self.current_time = 0.0;
    }

    /// Parks the ground on one color, for the screens that are not a round.
    pub fn set_solid(&mut self, color: Color) {
        self.path = vec![color];
        self.current_time = self.time;
    }

    pub fn get_current_color(&self) -> Color {
        let Some(first) = self.path.first().copied() else {
            return Color::BLACK;
        };

        let segments = self.path.len().saturating_sub(1);
        if segments == 0 {
            return first;
        }

        // Every stop gets the same slice of the second, so a five-color round
        // and an eight-color one both feel like one sweep rather than one
        // dragging and the other rushing.
        let travelled = (self.current_time / self.time).clamp(0.0, 1.0) * segments as f32;
        let segment = (travelled.floor() as usize).min(segments - 1);

        lerp_color(
            self.path[segment],
            self.path[segment + 1],
            travelled - segment as f32,
        )
    }

    pub fn update(&mut self, time: f32) {
        if self.is_in_transition() {
            self.current_time = (self.current_time + time).min(self.time);
        }
    }
}

pub fn spawn_camera(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    _texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let camera_bundle = Camera2dBundle::default();
    commands.spawn((
        camera_bundle,
        BackgroundTranstion::default(),
        // Shake lives on the camera next to the background transition; both
        // write to this entity, but to different parts of it.
        crate::feedback::ScreenShake::default(),
    ));
}

pub fn transition_to_game_state(
    keyboard_input: Res<Input<KeyCode>>,
    app_state: Res<State<AppState>>,
    mut app_state_next_state: ResMut<NextState<AppState>>,
) {
    if keyboard_input.just_pressed(KeyCode::G) {
        if app_state.get() != AppState::Game {
            app_state_next_state.set(AppState::Game);
            println!("Entered AppState::Game");
        }
    }
}

pub fn transition_to_main_menu_state(
    keyboard_input: Res<Input<KeyCode>>,
    app_state: Res<State<AppState>>,
    mut app_state_next_state: ResMut<NextState<AppState>>,
) {
    if keyboard_input.just_pressed(KeyCode::M) {
        if app_state.get() != AppState::MainMenu {
            app_state_next_state.set(AppState::MainMenu);
            println!("Entered AppState::MainMenu");
        }
    }
}

pub fn transition_to_game_over_state(
    keyboard_input: Res<Input<KeyCode>>,
    app_state: Res<State<AppState>>,
    mut app_state_next_state: ResMut<NextState<AppState>>,
) {
    if keyboard_input.just_pressed(KeyCode::H) {
        if app_state.get() != AppState::GameOver {
            app_state_next_state.set(AppState::GameOver);
            println!("Entered AppState::GameOver");
        }
    }
}


pub fn exit_game(
    keyboard_input: Res<Input<KeyCode>>,
    mut app_exit_event_writer: MessageWriter<AppExit>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        app_exit_event_writer.send(AppExit);
    }
}
