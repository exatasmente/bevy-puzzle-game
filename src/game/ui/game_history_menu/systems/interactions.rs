use bevy::prelude::*;

use crate::events::TransitionToStateEvent;
use crate::game::puzzle::components::ColorPuzzle;
use crate::game::puzzle::components::GameHistory;
use crate::game::puzzle::components::GameTimer;
use crate::game::puzzle::components::RenderLevelHistoryEvent;
use crate::game::ui::game_history_menu::components::*;
use crate::game::ui::game_history_menu::styles::*;
use crate::game::ui::game_history_menu::SpawnPaginationEvent;
use crate::pagination::Pagination;
use crate::AppState;

pub fn interact_with_level_history_option(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor, &LevelHistoryOption),
        (Changed<Interaction>, With<LevelHistoryOption>),
    >,
    mut render_level_history_event_writer: EventWriter<RenderLevelHistoryEvent>,
    mut transition_to_state_event_writer: EventWriter<TransitionToStateEvent>,
) {
    for (interaction, mut color, level_history_option) in button_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = BUTTON_PRESSED.into();
                transition_to_state_event_writer.send(TransitionToStateEvent {
                    state: AppState::LevelHistory,
                });
                render_level_history_event_writer.send(RenderLevelHistoryEvent {
                    index: level_history_option.index,
                });
            }
            Interaction::Hovered => *color = BUTTON_HOVERED.into(),
            Interaction::None => *color = BUTTON.into(),
        }
    }
}

pub fn interact_with_pagination_button(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor, &PaginationOption),
        (Changed<Interaction>, With<PaginationOption>),
    >,
    mut spawn_pagination_event_writer: EventWriter<SpawnPaginationEvent>,
    mut pagination: ResMut<Pagination>,
) {
    for (interaction, mut color, pagination_button) in button_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = BUTTON_PRESSED.into();
                pagination.set_page(pagination_button.index);
                spawn_pagination_event_writer.send(SpawnPaginationEvent);
            }
            Interaction::Hovered => *color = BUTTON_HOVERED.into(),
            Interaction::None => *color = BUTTON.into(),
        }
    }
}

pub fn interact_with_continue_button(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ContinueButton>),
    >,
    mut transition_to_state_event_writer: EventWriter<TransitionToStateEvent>,
    puzzle: Res<ColorPuzzle>,
    game_timer: Res<GameTimer>,
) {
    for (interaction, mut color) in button_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = BUTTON_PRESSED.into();

                // An Infinite run has no clock to expire, so it always resumes.
                // Testing the timer alone used to send it to the game-over
                // screen, because a zero-length timer reads as finished.
                let run_is_over = puzzle.game_mode.is_timed() && game_timer.timer.finished();

                transition_to_state_event_writer.send(TransitionToStateEvent {
                    state: if run_is_over {
                        AppState::GameOver
                    } else {
                        AppState::Game
                    },
                });
            }
            Interaction::Hovered => *color = BUTTON_HOVERED.into(),
            Interaction::None => *color = BUTTON.into(),
        }
    }
}

/// Ends the run deliberately and goes to the summary.
pub fn interact_with_end_run_button(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<EndRunButton>),
    >,
    mut transition_to_state_event_writer: EventWriter<TransitionToStateEvent>,
    puzzle: Res<ColorPuzzle>,
    game_timer: Res<GameTimer>,
    mut game_history: ResMut<GameHistory>,
) {
    for (interaction, mut color) in button_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = BUTTON_PRESSED.into();

                // Record what the summary needs, the same way the timer-expiry
                // path does.
                game_history.set_game_mode(puzzle.game_mode);
                game_history.set_total_time(game_timer.timer.elapsed_secs());

                transition_to_state_event_writer.send(TransitionToStateEvent {
                    state: AppState::GameOverResume,
                });
            }
            Interaction::Hovered => *color = BUTTON_HOVERED.into(),
            Interaction::None => *color = BUTTON.into(),
        }
    }
}
