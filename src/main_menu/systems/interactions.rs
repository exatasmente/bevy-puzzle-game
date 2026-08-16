use bevy::prelude::*;

use crate::events::TransitionToStateEvent;
use crate::game::puzzle::components::{ColorPuzzle, PowerUps};
use crate::game::puzzle::components::GameHistory;
use crate::main_menu::components::*;
use crate::main_menu::styles::{card_border, card_border_hovered, card_border_pressed};
use crate::game::score::resources::SavedRun;
use crate::pagination::Pagination;
use crate::AppState;

pub fn interact_with_play_button(
    // Iterated, not `get_single_mut`: the menu has one of these per mode, so a
    // single-result query silently did nothing whenever more than one changed
    // in the same frame.
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor, &PlayButton),
        (Changed<Interaction>, With<PlayButton>),
    >,
    mut transition_to_state_event_writer: MessageWriter<TransitionToStateEvent>,
    mut puzzle: ResMut<ColorPuzzle>,
    mut game_history: ResMut<GameHistory>,
    mut pagination: ResMut<Pagination>,
    mut power_ups: ResMut<PowerUps>,
) {
    for (interaction, mut background_color, play_button) in button_query.iter_mut() {
        // The card's border carries the mode's own color, so the feedback for
        // touching it has to be built from that color rather than from the
        // shared grey button ramp.
        let accent = play_button.game_mode.accent();

        match *interaction {
            Interaction::Pressed => {
                *background_color = card_border_pressed(accent).into();
                puzzle.setup(&play_button.game_mode);
                // A fresh run starts empty-handed: power-ups are earned inside
                // one run and do not carry between them.
                power_ups.clear();
                game_history.reset();
                game_history.set_game_mode(play_button.game_mode);
                pagination.reset();
                transition_to_state_event_writer.write(TransitionToStateEvent {
                    state: AppState::Game,
                });
            }
            Interaction::Hovered => *background_color = card_border_hovered(accent).into(),
            Interaction::None => *background_color = card_border(accent).into(),
        }
    }
}

/// Picks a stored run back up.
///
/// The board is not restored, only the score — which is where the level, the
/// piece count and the color distance all come from. There is no position to
/// come back to in a game that deals a new board every round; what the player
/// is returning to is their place in the curve.
pub fn interact_with_continue_run_button(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor, &ContinueRunButton),
        (Changed<Interaction>, With<ContinueRunButton>),
    >,
    mut transition_to_state_event_writer: MessageWriter<TransitionToStateEvent>,
    mut puzzle: ResMut<ColorPuzzle>,
    mut game_history: ResMut<GameHistory>,
    mut pagination: ResMut<Pagination>,
    mut saved_run: ResMut<SavedRun>,
    mut power_ups: ResMut<PowerUps>,
) {
    for (interaction, mut background_color, button) in button_query.iter_mut() {
        let accent = button.game_mode.accent();

        match *interaction {
            Interaction::Pressed => {
                *background_color = card_border_pressed(accent).into();

                puzzle.setup(&button.game_mode);
                puzzle.restore_score(button.score);
                // After `setup`, which seeds a full complement: the run is
                // picked up where it was left, lives included.
                puzzle.restore_lives(button.lives);
                *power_ups = button.power_ups;

                game_history.reset();
                game_history.set_game_mode(button.game_mode);
                game_history.restore(button.score);
                pagination.reset();

                // Keep the stored run pointing at the same place until the
                // resumed run moves it. Dropping it here would lose the run if
                // the player bounced straight back to the menu.
                saved_run.store(
                    button.game_mode,
                    button.score,
                    button.lives,
                    button.power_ups,
                );

                transition_to_state_event_writer.write(TransitionToStateEvent {
                    state: AppState::Game,
                });
            }
            Interaction::Hovered => *background_color = card_border_hovered(accent).into(),
            Interaction::None => *background_color = card_border(accent).into(),
        }
    }
}

/// Opens the goals screen.
pub fn interact_with_achievements_button(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<AchievementsButton>),
    >,
    mut transition_to_state_event_writer: MessageWriter<TransitionToStateEvent>,
) {
    for (interaction, mut background_color) in button_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *background_color = crate::theme::SURFACE.into();
                transition_to_state_event_writer.write(TransitionToStateEvent {
                    state: AppState::Achievements,
                });
            }
            Interaction::Hovered => {
                *background_color = crate::theme::BUTTON_HOVERED.into()
            }
            Interaction::None => *background_color = crate::theme::SURFACE_RAISED.into(),
        }
    }
}
