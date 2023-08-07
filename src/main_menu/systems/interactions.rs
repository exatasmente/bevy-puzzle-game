use bevy::app::AppExit;
use bevy::prelude::*;

use crate::game::puzzle::components::ColorPuzzle;
use crate::main_menu::components::*;
use crate::main_menu::styles::{HOVERED_BUTTON_COLOR, NORMAL_BUTTON_COLOR, PRESSED_BUTTON_COLOR};
use crate::AppState;

pub fn interact_with_play_button(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor, &PlayButton),
        (Changed<Interaction>, With<PlayButton>),
    >,
    mut app_state_next_state: ResMut<NextState<AppState>>,
    mut puzzle : ResMut<ColorPuzzle>,
) {
    if let Ok((interaction, mut background_color, play_button)) = button_query.get_single_mut() {
        match *interaction {
            Interaction::Clicked => {
                *background_color = PRESSED_BUTTON_COLOR.into();
                puzzle.setup(&play_button.game_mode);
                app_state_next_state.set(AppState::Game);
            }
            Interaction::Hovered => {
                *background_color = HOVERED_BUTTON_COLOR.into();
            }
            Interaction::None => {
                *background_color = NORMAL_BUTTON_COLOR.into();
            }
        }
    }
}

pub fn interact_with_quit_button(
    mut app_exit_event_writer: EventWriter<AppExit>,
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<QuitButton>),
    >,
) {
    if let Ok((interaction, mut background_color)) = button_query.get_single_mut() {
        match *interaction {
            Interaction::Clicked => {
                *background_color = PRESSED_BUTTON_COLOR.into();
                app_exit_event_writer.send(AppExit);
            }
            Interaction::Hovered => {
                *background_color = HOVERED_BUTTON_COLOR.into();
            }
            Interaction::None => {
                *background_color = NORMAL_BUTTON_COLOR.into();
            }
        }
    }
}
