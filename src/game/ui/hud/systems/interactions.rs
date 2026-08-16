use bevy::prelude::*;

use crate::events::TransitionToStateEvent;
use crate::game::ui::hud::components::{HistoryBackButtom, HistoryButtom};
use crate::game::ui::hud::styles::{BUTTON, BUTTON_HOVERED, BUTTON_PRESSED};
use crate::AppState;

pub fn interact_with_pause_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<HistoryButtom>),
    >,
    mut transition_to_state_event_writer: MessageWriter<TransitionToStateEvent>,
) {
    for (interaction, mut background_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *background_color = BUTTON_PRESSED.into();
                // Routed through the event rather than setting NextState here,
                // which is what the rest of the UI does and what the web build
                // needs.
                transition_to_state_event_writer.write(TransitionToStateEvent {
                    state: AppState::History,
                });
            }
            Interaction::Hovered => *background_color = BUTTON_HOVERED.into(),
            Interaction::None => *background_color = BUTTON.into(),
        }
    }
}

pub fn interact_with_history_back_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<HistoryBackButtom>),
    >,
    mut transition_to_state_event_writer: MessageWriter<TransitionToStateEvent>,
) {
    for (interaction, mut background_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *background_color = BUTTON_PRESSED.into();
                transition_to_state_event_writer.write(TransitionToStateEvent {
                    state: AppState::History,
                });
            }
            Interaction::Hovered => *background_color = BUTTON_HOVERED.into(),
            Interaction::None => *background_color = BUTTON.into(),
        }
    }
}
