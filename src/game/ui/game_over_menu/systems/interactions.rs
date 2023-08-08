use bevy::prelude::*;
use crate::ui::game_over_menu::components::*;
use crate::ui::game_over_menu::styles::*;

use crate::AppState;

pub fn interact_with_history_button(
    mut button_query: Query<&Interaction,(Changed<Interaction>, With<GameOverHistoryButton>)>,
    mut app_state_next_state: ResMut<NextState<AppState>>,
) {


    for interaction in button_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                app_state_next_state.set(AppState::History);
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
    mut app_state_next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, mut color) in button_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                app_state_next_state.set(AppState::MainMenu);
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