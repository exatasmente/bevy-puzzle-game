use bevy::prelude::*;
use crate::AppState;
use crate::game::ui::hud::components::HistoryButtom;
use crate::game::ui::hud::components::HistoryBackButtom;

pub fn interact_with_pause_button(
    mut app_state_next_state: ResMut<NextState<AppState>>,
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<HistoryButtom>)>,
) {
    for interaction in interaction_query.iter_mut() {
        match interaction {
            Interaction::Clicked => {
                app_state_next_state.set(AppState::GameOver)
            },
            _ => {}
        }
    }
}


pub fn interact_with_history_back_button(
    mut app_state_next_state: ResMut<NextState<AppState>>,
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<HistoryBackButtom>)>,
) {
    for interaction in interaction_query.iter_mut() {
        match interaction {
            Interaction::Clicked => {
                app_state_next_state.set(AppState::GameOver)
            },
            _ => {}
        }
    }
}
