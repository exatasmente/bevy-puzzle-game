use bevy::app::AppExit;
use bevy::prelude::*;

use crate::game::puzzle::components::GameHistory;
use crate::game::puzzle::components::RenderLevelHistoryEvent;
use crate::game::ui::game_over_menu::components::*;
use crate::game::ui::game_over_menu::styles::*;
use crate::AppState;

pub fn interact_with_restart_button(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<RestartButton>),
    >,
    mut app_state_next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, mut color) in button_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                app_state_next_state.set(AppState::Game);
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

pub fn interact_with_quit_button(
    mut app_exit_event_writer: EventWriter<AppExit>,
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<QuitButton>),
    >,
) {
    for (interaction, mut color) in button_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                app_exit_event_writer.send(AppExit);
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
