use bevy::app::AppExit;
use bevy::prelude::*;

use super::Pagination;
use crate::game::puzzle::components::RenderLevelHistoryEvent;
use crate::game::puzzle::components::GameTimer;
use crate::game::puzzle::components::GameMode;
use crate::game::puzzle::components::NewGameEvent;
use crate::game::ui::game_over_menu::components::*;
use crate::game::ui::game_over_menu::styles::*;
use crate::AppState;
pub fn interact_with_level_history_option(
    mut button_query: Query<(&Interaction, &LevelHistoryOption),(Changed<Interaction>, With<LevelHistoryOption>)>,
    mut render_level_history_event_writer: EventWriter<RenderLevelHistoryEvent>,
    mut app_state_next_state: ResMut<NextState<AppState>>,
) {

    for (interaction, level_history_option) in button_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                render_level_history_event_writer.send(RenderLevelHistoryEvent {
                    index : level_history_option.index
                });
                app_state_next_state.set(AppState::History);
            }, 
            _ => {}
        }
    }

}

pub fn interact_with_pagination_button(
    mut button_query: Query<(&Interaction, &PaginationOption),(Changed<Interaction>, With<PaginationOption>)>,
    mut app_state_next_state: ResMut<NextState<AppState>>,
    mut pagination: ResMut<Pagination>,
) {

    for (interaction, pagination_button) in button_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                pagination.set_page(pagination_button.index);
                app_state_next_state.set(AppState::GameOver);
            }, 
            _ => {}
        }
    }

}


pub fn interact_with_continue_button(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ContinueButton>),
    >,
    mut app_state_next_state: ResMut<NextState<AppState>>,
    mut new_game_event_writer: EventWriter<NewGameEvent>,
    game_timer : Res<GameTimer>,
) {
    for (interaction, mut color) in button_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                if game_timer.timer.finished() {
                    new_game_event_writer.send(NewGameEvent {
                        game_mode : GameMode::TimeTrial,
                    });
                } else {
                    app_state_next_state.set(AppState::Game);
                }

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