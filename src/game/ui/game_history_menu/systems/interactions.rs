use bevy::prelude::*;

use crate::Pagination;
use crate::game::puzzle::components::RenderLevelHistoryEvent;
use crate::game::puzzle::components::GameTimer;
use crate::game::ui::game_history_menu::components::*;
use crate::game::ui::game_history_menu::styles::*;
use crate::game::ui::game_history_menu::SpawnPaginationEvent;
use crate::AppState;
use crate::events::TransitionToStateEvent;

pub fn interact_with_level_history_option(
    mut button_query: Query<(&Interaction, &LevelHistoryOption),(Changed<Interaction>, With<LevelHistoryOption>)>,
    mut render_level_history_event_writer: EventWriter<RenderLevelHistoryEvent>,
    mut transition_to_state_event_writer: EventWriter<TransitionToStateEvent>,
) {

    for (interaction, level_history_option) in button_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                transition_to_state_event_writer.send(TransitionToStateEvent {
                    state: AppState::LevelHistory,
                });

                render_level_history_event_writer.send(RenderLevelHistoryEvent {
                    index : level_history_option.index
                });
                
                
                
            }, 
            _ => {}
        }
    }

}

pub fn interact_with_pagination_button(
    mut button_query: Query<(&Interaction, &PaginationOption),(Changed<Interaction>, With<PaginationOption>)>,
    mut spawn_pagination_event_writer: EventWriter<SpawnPaginationEvent>,
    mut pagination: ResMut<Pagination>,
) {


    for (interaction, pagination_button) in button_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                println!("Pagination Button Clicked, {}", pagination_button.index);
                pagination.set_page(pagination_button.index);
                spawn_pagination_event_writer.send(SpawnPaginationEvent);
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
    mut transition_to_state_event_writer: EventWriter<TransitionToStateEvent>,
    game_timer : Res<GameTimer>,
) {
    for (interaction, mut color) in button_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                if game_timer.timer.finished() {
                    transition_to_state_event_writer.send(TransitionToStateEvent {
                        state: AppState::GameOver,
                    });
                } else {
                    transition_to_state_event_writer.send(TransitionToStateEvent {
                        state: AppState::Game,
                    });
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