use bevy::prelude::*;

use crate::events::TransitionToStateEvent;
use crate::game::puzzle::components::{ColorPuzzle, PowerUps, UsePowerUpEvent};
use crate::game::ui::hud::components::{HistoryBackButtom, HistoryButtom, PowerUpButton};
use crate::game::ui::hud::systems::updates::usable;
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

/// Spends a power-up.
///
/// The effect itself is not applied here: this sends `UsePowerUpEvent` and the
/// puzzle systems act on it, which keeps the board-touching code in the module
/// that owns the board. A press on a button with nothing to spend is ignored
/// rather than played as a failed action — the button is already dimmed, and a
/// buzz for touching a disabled control teaches nothing.
pub fn interact_with_power_up_buttons(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor, &PowerUpButton),
        (Changed<Interaction>, With<PowerUpButton>),
    >,
    power_ups: Res<PowerUps>,
    puzzle: Res<ColorPuzzle>,
    mut use_power_up: MessageWriter<UsePowerUpEvent>,
) {
    for (interaction, mut background_color, button) in button_query.iter_mut() {
        let live = usable(&power_ups, &puzzle, button.kind);

        match *interaction {
            Interaction::Pressed => {
                if live {
                    use_power_up.write(UsePowerUpEvent { kind: button.kind });
                    *background_color = BUTTON_PRESSED.into();
                }
            }
            Interaction::Hovered if live => *background_color = BUTTON_HOVERED.into(),
            _ => {}
        }
    }
}
