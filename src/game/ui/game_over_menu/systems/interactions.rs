use bevy::prelude::*;
use crate::ui::game_over_menu::components::*;
use crate::ui::game_over_menu::styles::*;
use crate::events::TransitionToStateEvent;

use crate::AppState;

pub fn interact_with_history_button(
    mut button_query: Query<&Interaction,(Changed<Interaction>, With<GameOverHistoryButton>)>,
    mut transition_to_state_event_writer: EventWriter<TransitionToStateEvent>,
) {


    for interaction in button_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                transition_to_state_event_writer.send(TransitionToStateEvent {
                    state: AppState::History,
                });
            }, 
            _ => {}
        }
    }

}


pub fn interact_with_main_menu_button(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<MainMenuButton>),
    >,
    mut transition_to_state_event_writer: EventWriter<TransitionToStateEvent>,
) {
    for (interaction, mut color) in button_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                transition_to_state_event_writer.send(TransitionToStateEvent {
                    state: AppState::MainMenu,
                });
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

pub fn interact_with_game_over_resume_button(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<GameOverMenu>),
    >,
    mut transition_to_state_event_writer: EventWriter<TransitionToStateEvent>,
) {
    for (interaction, mut color) in button_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                transition_to_state_event_writer.send(TransitionToStateEvent {
                    state: AppState::GameOverResume,
                });
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}