use bevy::app::AppExit;
use bevy::prelude::*;


use crate::events::*;
use crate::AppState;


#[derive(Component, Debug, Reflect)]
pub struct BackgroundTranstion {
    start_color: Color,
    end_color: Color,
    time: f32,
    current_time: f32,
}

impl Default for BackgroundTranstion {
    fn default() -> Self {
        Self {
            start_color: Color::rgb(0.0, 0.0, 0.0),
            end_color: Color::rgb(0.0, 0.0, 0.0),
            time: 3.0,
            current_time: 0.0,
        }
    }
}

impl BackgroundTranstion {
    pub fn is_in_transition(&self) -> bool {
        self.current_time < self.time
    }

    fn get_color_diff(&mut self, a : f32, b : f32) -> f32 {
        if a > b {
            a - b
        } else {
            b - a
        }
    }

    pub fn get_current_color(&mut self) -> Color {
        let mut color = self.start_color.clone();
        let red_diff = self.get_color_diff(self.start_color.r(), self.end_color.r());
        let green_diff = self.get_color_diff(self.start_color.g(), self.end_color.g());
        let blue_diff = self.get_color_diff(self.start_color.b(), self.end_color.b());
        
        let red_change = if self.start_color.r() > self.end_color.r() {
            self.start_color.r() - ((red_diff / self.time) * self.current_time)
        } else {
            self.start_color.r() + ((red_diff / self.time)  * self.current_time)
        };

        let green_change = if self.start_color.g() > self.end_color.g() {
            self.start_color.g() - ((green_diff / self.time) * self.current_time)
        } else {
            self.start_color.g() + ((green_diff / self.time ) * self.current_time)
        };

        let blue_change = if self.start_color.b() > self.end_color.b() {
            self.start_color.b() - ((blue_diff / self.time) * self.current_time)
        } else {
            self.start_color.b() + ((blue_diff / self.time) * self.current_time)
        };


        color.set_r(red_change);
        color.set_g(green_change);
        color.set_b(blue_change);

        color
    }

    pub fn start_transition(&mut self, start_color: Color, end_color: Color, time: f32) {
        self.start_color = start_color;
        self.end_color = end_color;
        self.time = time;
        self.current_time = 0.0;
    }

    pub fn update(&mut self, time: f32) {
        if self.is_in_transition() {
            self.current_time = if self.current_time + time > self.time {
                self.time
            } else {
                self.current_time + time
            };
        }
    }

    pub fn reset(&mut self) {
        self.start_color = Color::rgb(0.0, 0.0, 0.0);
        self.end_color = Color::rgb(0.0, 0.0, 0.0);
        self.current_time = 0.0;
    }

    pub fn set_start_color(&mut self, color: Color) {
        self.start_color = color;
    }

    pub fn set_end_color(&mut self, color: Color) {
        self.end_color = color;
    }

    pub fn set_time(&mut self, time: f32) {
        self.time = time;
    }

    pub fn set_current_time(&mut self, current_time: f32) {
        self.current_time = current_time;
    }
}

pub fn spawn_camera(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    _texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let camera_bundle = Camera2dBundle::default();
    commands.spawn((camera_bundle, BackgroundTranstion::default() ));
}

pub fn transition_to_game_state(
    keyboard_input: Res<Input<KeyCode>>,
    app_state: Res<State<AppState>>,
    mut app_state_next_state: ResMut<NextState<AppState>>,
) {
    if keyboard_input.just_pressed(KeyCode::G) {
        if app_state.0 != AppState::Game {
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
        if app_state.0 != AppState::MainMenu {
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
        if app_state.0 != AppState::GameOver {
            app_state_next_state.set(AppState::GameOver);
            println!("Entered AppState::GameOver");
        }
    }
}

pub fn handle_game_over(
    mut game_over_event_reader: EventReader<GameOver>,
    mut app_state_next_state: ResMut<NextState<AppState>>,
) {
    for event in game_over_event_reader.iter() {
        println!("Your final score is: {}", event.score.to_string());
        app_state_next_state.set(AppState::GameOver);
        println!("Entered AppState::GameOver");
    }
}

pub fn exit_game(
    keyboard_input: Res<Input<KeyCode>>,
    mut app_exit_event_writer: EventWriter<AppExit>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        app_exit_event_writer.send(AppExit);
    }
}
