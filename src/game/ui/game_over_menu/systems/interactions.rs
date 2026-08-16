use bevy::prelude::*;

use crate::events::TransitionToStateEvent;
use crate::game::puzzle::components::{GameHistory, NewGameEvent};
use crate::game::ui::game_over_menu::components::*;
use crate::game::ui::game_over_menu::styles::*;
use crate::AppState;

/// Starts another run in the same mode, without a detour through the menu.
///
/// `NewGameEvent` and its handler already existed but nothing ever sent the
/// event, so there was no way to replay a mode: the only paths off this screen
/// were the main menu and the history list. The cost of starting again is the
/// single biggest lever on whether a player takes another turn.
pub fn interact_with_play_again_button(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<PlayAgainButton>),
    >,
    game_history: Res<GameHistory>,
    mut new_game_event_writer: MessageWriter<NewGameEvent>,
) {
    for (interaction, mut color) in button_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = BUTTON_PRIMARY_PRESSED.into();
                new_game_event_writer.write(NewGameEvent {
                    game_mode: game_history.game_mode,
                });
            }
            Interaction::Hovered => *color = BUTTON_PRIMARY_HOVERED.into(),
            Interaction::None => *color = BUTTON_PRIMARY.into(),
        }
    }
}

pub fn interact_with_history_button(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<GameOverHistoryButton>),
    >,
    mut transition_to_state_event_writer: MessageWriter<TransitionToStateEvent>,
) {
    for (interaction, mut color) in button_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = BUTTON_PRESSED.into();
                transition_to_state_event_writer.write(TransitionToStateEvent {
                    state: AppState::History,
                });
            }
            Interaction::Hovered => *color = BUTTON_HOVERED.into(),
            Interaction::None => *color = BUTTON.into(),
        }
    }
}

pub fn interact_with_main_menu_button(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<MainMenuButton>),
    >,
    mut transition_to_state_event_writer: MessageWriter<TransitionToStateEvent>,
) {
    for (interaction, mut color) in button_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = BUTTON_PRESSED.into();
                transition_to_state_event_writer.write(TransitionToStateEvent {
                    state: AppState::MainMenu,
                });
            }
            Interaction::Hovered => *color = BUTTON_HOVERED.into(),
            Interaction::None => *color = BUTTON.into(),
        }
    }
}

/// The "fim de jogo" card is itself the button: any tap moves on.
pub fn interact_with_game_over_resume_button(
    mut button_query: Query<&Interaction, (Changed<Interaction>, With<GameOverMenu>)>,
    mut transition_to_state_event_writer: MessageWriter<TransitionToStateEvent>,
) {
    for interaction in button_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            transition_to_state_event_writer.write(TransitionToStateEvent {
                state: AppState::GameOverResume,
            });
        }
    }
}
