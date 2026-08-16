use bevy::prelude::*;

use crate::events::TransitionToStateEvent;
use crate::game::puzzle::components::{level_for_score, GameHistory, NewGameEvent};
use crate::game::score::resources::LastRunOutcome;
use crate::storage;
use crate::game::ui::game_over_menu::components::*;
use crate::game::ui::game_over_menu::styles::*;
use crate::AppState;

/// Starts another run in the same mode, without a detour through the menu.
///
/// `NewGameEvent` and its handler already existed but nothing ever sent the
/// event, so there was no way to replay a mode: the only paths off this screen
/// were the main menu and the history list. The cost of starting again is the
/// single biggest lever on whether a player takes another turn.
pub fn interact_with_play_again_button(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<PlayAgainButton>),
    >,
    game_history: Res<GameHistory>,
    mut new_game_event_writer: MessageWriter<NewGameEvent>,
) {
    for (interaction, mut color) in button_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = BUTTON_PRIMARY_PRESSED.into();
                new_game_event_writer.write(NewGameEvent {
                    game_mode: game_history.game_mode,
                });
            }
            Interaction::Hovered => *color = BUTTON_PRIMARY_HOVERED.into(),
            Interaction::None => *color = BUTTON_PRIMARY.into(),
        }
    }
}

pub fn interact_with_history_button(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<GameOverHistoryButton>),
    >,
    mut transition_to_state_event_writer: MessageWriter<TransitionToStateEvent>,
) {
    for (interaction, mut color) in button_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = BUTTON_PRESSED.into();
                transition_to_state_event_writer.write(TransitionToStateEvent {
                    state: AppState::History,
                });
            }
            Interaction::Hovered => *color = BUTTON_HOVERED.into(),
            Interaction::None => *color = BUTTON.into(),
        }
    }
}

pub fn interact_with_main_menu_button(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<MainMenuButton>),
    >,
    mut transition_to_state_event_writer: MessageWriter<TransitionToStateEvent>,
) {
    for (interaction, mut color) in button_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = BUTTON_PRESSED.into();
                transition_to_state_event_writer.write(TransitionToStateEvent {
                    state: AppState::MainMenu,
                });
            }
            Interaction::Hovered => *color = BUTTON_HOVERED.into(),
            Interaction::None => *color = BUTTON.into(),
        }
    }
}

/// The "fim de jogo" card is itself the button: any tap moves on.
pub fn interact_with_game_over_resume_button(
    mut button_query: Query<&Interaction, (Changed<Interaction>, With<GameOverMenu>)>,
    mut transition_to_state_event_writer: MessageWriter<TransitionToStateEvent>,
) {
    for interaction in button_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            transition_to_state_event_writer.write(TransitionToStateEvent {
                state: AppState::GameOverResume,
            });
        }
    }
}

/// Hands the run's numbers to the page, which draws the image and shares it.
///
/// Rust does not build the picture. Drawing a card in-engine would mean reading
/// a texture back off the GPU and handing the bytes to JS, and `navigator.share`
/// has to be called from inside the gesture anyway — which is the page's world,
/// not this one. So the summary goes out through `storage::save`, the same
/// escape hatch used to observe the game from outside it, and the script in
/// `index.html` picks it up.
///
/// A counter rides along on the key so two shares of the same score still read
/// as two separate requests. Without it the second press writes an identical
/// value, the storage event never fires, and the button appears dead.
pub fn interact_with_share_button(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ShareScoreButton>),
    >,
    outcome: Res<LastRunOutcome>,
    game_history: Res<GameHistory>,
    mut requests: Local<usize>,
) {
    for (interaction, mut background_color) in button_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *background_color = BUTTON_PRESSED.into();
                *requests += 1;

                // `key=value` pairs, the same shape every other stored value
                // here uses.
                let payload = format!(
                    "n={};mode={};score={};best={};record={};level={};streak={}",
                    *requests,
                    game_history.game_mode.as_str(),
                    outcome.score,
                    outcome.best,
                    usize::from(outcome.is_record),
                    level_for_score(outcome.score),
                    game_history.max_streak,
                );

                storage::save("color_puzzle.share_request", &payload);
            }
            Interaction::Hovered => *background_color = BUTTON_HOVERED.into(),
            Interaction::None => *background_color = BUTTON.into(),
        }
    }
}
