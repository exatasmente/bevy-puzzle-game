use bevy::prelude::*;

use crate::achievements_menu::components::AchievementsBackButton;
use crate::events::TransitionToStateEvent;
use crate::theme;
use crate::AppState;

pub fn interact_with_back_button(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<AchievementsBackButton>),
    >,
    mut transition_to_state_event_writer: MessageWriter<TransitionToStateEvent>,
) {
    for (interaction, mut background_color) in button_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *background_color = theme::BUTTON_PRIMARY_PRESSED.into();
                transition_to_state_event_writer.write(TransitionToStateEvent {
                    state: AppState::MainMenu,
                });
            }
            Interaction::Hovered => *background_color = theme::BUTTON_PRIMARY_HOVERED.into(),
            Interaction::None => *background_color = theme::PRIMARY.into(),
        }
    }
}
