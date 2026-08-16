use std::time::Duration;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy::camera::ClearColorConfig;
use crate::events::InteractionAnimationEvent;
use crate::feedback::BannerEvent;
use crate::theme;
use super::components::*;
use crate::systems::BackgroundTranstion;
use crate::wfc::Tile;

#[derive(Component)]
pub struct LastClick;

/// Everything a pick announces, in one parameter.
///
/// Grouped because Bevy 0.10 stops at sixteen system parameters and
/// `player_interaction` had reached seventeen. The bundle is also the honest
/// shape of the thing: these four events are always sent together, as one
/// answer to one tap.
#[derive(SystemParam)]
pub struct PickEvents<'w> {
    start_level: MessageWriter<'w, StartLevelEvent>,
    last_interaction: MessageWriter<'w, LastInteractionEvent>,
    animation: MessageWriter<'w, InteractionAnimationEvent>,
    banner: MessageWriter<'w, BannerEvent>,
}

pub fn player_interaction(
    mut commands: Commands,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    event_click  : Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    mut object_query: Query<(&Transform, &PuzzleColor, &mut Shape), With<PuzzleColor>>,
    ui_interaction_query: Query<&Interaction>,
    mut puzzle: ResMut<ColorPuzzle>,
    mut game_timer: ResMut<GameTimer>,
    mut pending_level_start: ResMut<PendingLevelStart>,
    memory_phase: Res<MemoryPhase>,
    round_intro: Res<RoundIntro>,
    mut events: PickEvents,
    last_click_query: Query<Entity, With<LastClick>>,
) {

    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_q.single() else {
        return;
    };

    // Tapping a HUD button used to also register as a puzzle pick, which now
    // means pausing the game would break the player's streak. Ignore world
    // input whenever the pointer is over any UI element.
    let pointer_over_ui = ui_interaction_query
        .iter()
        .any(|interaction| !matches!(interaction, Interaction::None));
    if pointer_over_ui {
        return;
    }

    // The board is being held so the player can see what they missed.
    if pending_level_start.is_holding() {
        return;
    }

    // In Memory, the colors are still on screen. Accepting a pick now would
    // turn the mode back into an ordinary round.
    if memory_phase.is_previewing() {
        return;
    }

    // The board being dealt is not yet the board on screen.
    if round_intro.is_locked() {
        return;
    }

    // Note what is *not* here: the background sweep no longer gates input. The
    // sweep is a second or so of the ground walking the round's colors, and a
    // player who spots the answer melt away should be able to say so at once
    // rather than wait it out.
    if event_click.just_released(MouseButton::Left) || touches.any_just_pressed() {
        // Everything here is top-left, and nothing is flipped.
        // `viewport_to_ndc` flips y itself before handing the point to the
        // projection, so `viewport_to_world_2d` wants an origin at the *top*
        // left — and both `cursor_position` and `Touches` report winit's
        // top-left positions straight through. So both go in unchanged.
        //
        // This is the opposite of what it was under 0.10, where the cursor
        // arrived bottom-left and only the touch branch had to be converted.
        // Leaving either flip in place mirrors every pick vertically, and
        // compiles without complaint — which is why this is its own commit.
        let screen_position = if let Some(touch) = touches.first_pressed_position() {
            touch
        } else if let Some(cursor) = window.cursor_position() {
            cursor
        } else {
            return;
        };

        let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, screen_position) else {
            return;
        };

        for last_click in last_click_query.iter() {
            commands.entity(last_click).despawn();
        }

        let mut scored = false;
        let mut colors = Vec::new();
        let mut correct_position = None;
        let mut correct_corners = Vec::new();

        for (_, puzzle_color, _) in object_query.iter_mut() {
            colors.push(puzzle_color.as_level_color());

            if puzzle_color.is_correct_color {
                correct_position = Some(Vec2::new(puzzle_color.x, puzzle_color.y));
                correct_corners = puzzle_color.corners.clone();
            }

            if puzzle_color.contains(world_position) && puzzle_color.is_correct_color {
                scored = true;
            }
        }

        let mut bonus_seconds = 0.0;
        let mut leveled_up = false;
        let mut gained_life = false;

        if scored {
            if puzzle.game_mode == GameMode::TimeTrial {
                bonus_seconds = puzzle.get_seconds_added_per_success();
            }

            leveled_up = puzzle.increase_score(&mut game_timer);

            if leveled_up && puzzle.level_grants_life() {
                gained_life = puzzle.gain_life();
            }
        }

        events.animation.write(InteractionAnimationEvent {
            position: world_position,
            scored,
            bonus_seconds,
            // Only meaningful on a miss, where it drives the answer reveal.
            correct_position,
            correct_corners,
        });

        if leveled_up {
            // A regained life rides along on the level-up banner rather than
            // getting one of its own: `handle_banner_events` keeps only the
            // newest banner on screen, so two announcements in the same frame
            // means one of them is never read.
            let text = if gained_life {
                format!("NIVEL {}  +1 VIDA", puzzle.level())
            } else {
                format!("NIVEL {}", puzzle.level())
            };

            events.banner.write(BannerEvent::large(text, theme::ACCENT));
        }

        events.last_interaction.write(LastInteractionEvent::new(
            world_position,
            puzzle.get_correct_color_index(),
            colors,
            scored,
        ));

        if scored {
            // Keep the momentum: a correct pick moves straight on.
            events.start_level.write(StartLevelEvent);
        } else {
            // Missing is no longer free. A mode with a clock pays in seconds; a
            // mode without one pays a life, which is also the only thing that
            // can end an untimed run.
            //
            // The clock is charged by winding `elapsed` forward rather than by
            // shortening the duration, so `TimeTrial`'s bonus seconds — which
            // extend the duration — keep meaning what they meant. Capping at
            // the duration is what turns "the penalty was more time than you
            // had" into an ordinary expiry: `tick_game_timer` is what notices,
            // and it is paused for the length of the hold, so the run ends
            // after the answer has been shown rather than over the top of it.
            let penalty = puzzle.game_mode.miss_penalty_seconds();
            if penalty > 0.0 {
                let duration = game_timer.timer.duration().as_secs_f32();
                let spent = (game_timer.timer.elapsed_secs() + penalty).min(duration);
                game_timer
                    .timer
                    .set_elapsed(Duration::from_secs_f32(spent));
            }

            // Returns whether that was the last one; the run is ended by
            // `advance_pending_level`, once the hold has played out.
            puzzle.lose_life();

            // Hold the board so the reveal has something to point at.
            pending_level_start.hold(puzzle.game_mode.hold_seconds());

            // A blank board has nothing to learn from. Put the colors back for
            // the length of the hold, so a missed Memory round still shows the
            // player what they were supposed to have remembered.
            if memory_phase.is_hidden() {
                for (_, puzzle_color, mut shape) in object_query.iter_mut() {
                    shape.fill = Some(Fill::color(puzzle_color.color));
                }
            }
        }
    }

}

/// The shape of a board piece, from its outline.
fn piece_shape(corners: &[Vec2]) -> shapes::Polygon {
    shapes::Polygon {
        points: corners.to_vec(),
        closed: true,
    }
}

/// Side of a square piece, from its outline.
fn cell_size(corners: &[Vec2]) -> f32 {
    corners
        .iter()
        .map(|corner| corner.x.abs().max(corner.y.abs()) * 2.0)
        .fold(0.0_f32, f32::max)
}

/// A square piece, for `Mosaic`: its grid cells are pieces like any other, they
/// are just all the same shape. Corners are relative to the cell's centre, like
/// every other piece.
fn square_corners(size: f32) -> Vec<Vec2> {
    let half = size / 2.0;
    vec![
        Vec2::new(-half, -half),
        Vec2::new(half, -half),
        Vec2::new(half, half),
        Vec2::new(-half, half),
    ]
}

/// Draws a mosaic piece as children of its cell.
///
/// One node per arm plus a hub at the centre, all in the round's color. Kept as
/// children so the cell entity stays exactly what the rest of the game expects:
/// one `PuzzleColor` per cell, positioned at its bottom-left corner, which is
/// what the hit test and the answer reveal are written against.
fn spawn_tile_arms(parent: &mut ChildSpawnerCommands, tile: Tile, size: f32, color: Color) {
    let edges = tile.edges();

    // A piece with no arms is a blank plate. Drawing its hub anyway would put a
    // mark on every empty cell, which reads as a piece and gives the player a
    // pattern that is not there.
    if !edges.iter().any(|edge| *edge) {
        return;
    }

    let arm_width = size * 0.32;
    let half_arm = arm_width / 2.0;
    let half_cell = size / 2.0;

    // Pieces are spawned at their centre, so everything here is drawn around
    // the origin.
    let hub = shapes::Rectangle {
        extents: Vec2::splat(arm_width),
        origin: shapes::RectangleOrigin::Center,
        radii: None,
    };

    parent.spawn((
        ShapeBuilder::with(&hub).fill(Fill::color(color)).build(),
        Transform::from_xyz(0.0, 0.0, 0.01),
    ));

    // Each arm runs from inside the hub out to the edge it points at, so the
    // two overlap and the joint has no seam. `wfc` indexes edges clockwise from
    // the top, and the board's y axis grows upward.
    let reach = half_cell + half_arm;
    let offset = (half_cell - half_arm) / 2.0;
    let arms = [
        (Vec2::new(arm_width, reach), Vec2::new(0.0, offset)),
        (Vec2::new(reach, arm_width), Vec2::new(offset, 0.0)),
        (Vec2::new(arm_width, reach), Vec2::new(0.0, -offset)),
        (Vec2::new(reach, arm_width), Vec2::new(-offset, 0.0)),
    ];

    for (edge, (extents, position)) in edges.iter().zip(arms) {
        if !edge {
            continue;
        }

        let arm = shapes::Rectangle {
            extents,
            origin: shapes::RectangleOrigin::Center,
            radii: None,
        };

        parent.spawn((
            ShapeBuilder::with(&arm).fill(Fill::color(color)).build(),
            Transform::from_xyz(position.x, position.y, 0.01),
        ));
    }
}

/// Blanks a `Memory` board when its preview runs out.
///
/// Repainting `Fill` leaves the entities — and so the hit test and the answer
/// reveal — untouched: the squares are still exactly where they were, they just
/// stop telling the player which is which.
pub fn hide_memory_board(
    time: Res<Time>,
    puzzle: Res<ColorPuzzle>,
    mut memory_phase: ResMut<MemoryPhase>,
    mut square_query: Query<&mut Shape, With<PuzzleColor>>,
) {
    if !memory_phase.tick(time.delta()) {
        return;
    }

    let hidden = puzzle.hidden_color();
    for mut shape in square_query.iter_mut() {
        shape.fill = Some(Fill::color(hidden));
    }
}

/// Advances the short lock that covers a board being replaced.
pub fn tick_round_intro(time: Res<Time>, mut round_intro: ResMut<RoundIntro>) {
    round_intro.tick(time.delta());
}

/// Starts the next round once a post-miss hold expires — or ends the run, when
/// that miss was the last life.
///
/// The end waits for the hold rather than firing from the pick itself, and for
/// the same reason the hold exists at all: it is what shows the player the
/// answer they missed, and swapping to the summary over the top of it would
/// take away the one thing a miss teaches. Deciding it here rather than in a
/// system of its own also avoids dealing a board nobody sees — a state change
/// does not apply until the next frame, so a `StartLevelEvent` sent alongside
/// it would generate a whole round and then throw it away.
pub fn advance_pending_level(
    time: Res<Time>,
    puzzle: Res<ColorPuzzle>,
    game_timer: Res<GameTimer>,
    mut game_history: ResMut<GameHistory>,
    mut pending_level_start: ResMut<PendingLevelStart>,
    mut start_level_event_writer: MessageWriter<StartLevelEvent>,
    mut app_state_next_state: ResMut<NextState<crate::AppState>>,
) {
    if !pending_level_start.tick(time.delta()) {
        return;
    }

    if puzzle.is_out_of_lives() {
        game_history.set_game_mode(puzzle.game_mode);
        game_history.set_total_time(game_timer.timer.elapsed_secs());
        app_state_next_state.set(crate::AppState::GameOverResume);
        return;
    }

    start_level_event_writer.write(StartLevelEvent);
}


pub fn render_game_history(
    mut commands: Commands,
    game_history: Res<GameHistory>,
    mut render_game_history_events: MessageReader<RenderLevelHistoryEvent>,
    mut object_query: Query<Entity, With<PuzzleColor>>,
    mut last_click_query: Query<Entity, With<LastClick>>,
    mut camera_query: Query<(&mut Camera, &mut BackgroundTranstion), With<Camera2d>>,
) {

    let render_event = render_game_history_events.read().next();

    if render_event.is_none() {
        return;
    }

    let event = render_event.unwrap();

    for entity in object_query.iter_mut() {
        commands.entity(entity).despawn();
    }

    for entity in last_click_query.iter_mut() {
        commands.entity(entity).despawn();
    }


    let level_history = game_history.get_level_history(event.index);
    let Ok((mut camera, mut background_transition)) = camera_query.single_mut() else {
        return;
    };

    // A replay is read, not played, so it sits on the plain app background
    // rather than reproducing the round's sweep.
    background_transition.set_solid(theme::BACKGROUND);
    camera.clear_color = ClearColorConfig::Custom(theme::BACKGROUND);

    let mut z = 0.0;
    level_history.for_each_color(|index, color| {
        let fill = Fill::color(color.color);
        let is_correct_color = color.is_correct_color;
        // Each square remembers its own size, so a round played on a different
        // window size still replays as the board the player actually saw.
        let shape = piece_shape(&color.corners);

        let plate = if color.tile.is_some() {
            Fill::color(theme::SURFACE)
        } else {
            fill
        };

        commands
            .spawn((
                ShapeBuilder::with(&shape).fill(plate).build(),
                Transform::from_xyz(color.x, color.y, z),
                PuzzleColor { index, is_correct_color:  color.is_correct_color, x : color.x , y:  color.y, color: color.color.clone(), corners: color.corners.clone(), tile: color.tile },
            ))
            .with_children(|parent| {
                if let Some(tile) = color.tile {
                    spawn_tile_arms(parent, tile, cell_size(&color.corners), color.color);
                }
            });

        if is_correct_color {
            // Marks the answer on the replay, drawn as the piece shrunk toward
            // its own centre so it follows whatever shape the piece was.
            let inner_shape = piece_shape(
                &color
                    .corners
                    .iter()
                    .map(|corner| *corner * 0.72)
                    .collect::<Vec<_>>(),
            );
            commands .spawn((
                ShapeBuilder::with(&inner_shape)
                    .fill(Fill::color(Color::WHITE))
                    .build(),
                Transform::from_xyz(color.x + 10.0, color.y + 10.0, z + 0.01),
                LastClick,
            ));
        }
        z += 0.1;
    });

    let shape_clicked_position =  shapes::Rectangle {
        extents: Vec2::new(30.0, 30.0),
        origin: shapes::RectangleOrigin::Center,
        radii: None,
    };

    commands .spawn((
        ShapeBuilder::with(&shape_clicked_position)
            .fill(Fill::color(theme::DANGER))
            .build(),
        Transform::from_xyz(
            level_history.clicked_position.x,
            level_history.clicked_position.y,
            1.0,
        ),
        LastClick,
    ));

}

pub fn store_last_interaction_state(
    mut last_interaction_events: MessageReader<LastInteractionEvent>,
    mut game_history: ResMut<GameHistory>,
) {
    let level_history =last_interaction_events.read().next();

    if level_history.is_none() {
        return;
    }

    let event = level_history.unwrap();

    // No banner here any more. The streak still counts, and the HUD still shows
    // it, but a caption thrown across the board interrupts exactly the thing the
    // player is doing — reading a field of near-identical colors.
    game_history.add_level(event.level_history());
}


impl Default for GameTimer {
    fn default() -> Self {
        let mut timer = GameTimer {
            timer: Timer::from_seconds(1.0, TimerMode::Once),
        };

        timer.timer.pause();

        timer
    }
}



pub fn background_transition(
    mut camera_query: Query<(&mut Camera, &mut BackgroundTranstion), With<Camera2d>>,
    time : Res<Time>,
) {

    let Ok((mut camera, mut background_transition)) = camera_query.single_mut() else {
        return;
    };

    if background_transition.is_in_transition() {
        camera.clear_color = ClearColorConfig::Custom(background_transition.get_current_color());
        background_transition.update(time.delta_secs());
    }
}

/// Advances the run clock and ends the run when it expires.
///
/// This used to also spawn and update the on-screen timer text. That text now
/// lives in the HUD, where it can be updated in place and animated; this system
/// is only responsible for time itself.
pub fn tick_game_timer(
    mut game_timer: ResMut<GameTimer>,
    mut game_history: ResMut<GameHistory>,
    puzzle: Res<ColorPuzzle>,
    pending_level_start: Res<PendingLevelStart>,
    mut app_state_next_state: ResMut<NextState<crate::AppState>>,
    time : Res<Time>,
) {
    if !puzzle.game_mode.is_timed() {
        return;
    }

    // The beat that lets a miss teach something is not also a penalty: the
    // clock stops while the answer is being shown.
    if pending_level_start.is_holding() {
        return;
    }

    game_timer.timer.tick(time.delta());

    if game_timer.timer.is_finished() {
        game_history.set_game_mode(puzzle.game_mode);
        game_history.set_total_time(game_timer.timer.duration().as_secs_f32());
        app_state_next_state.set(crate::AppState::GameOverResume);
    }
}

pub fn spawn_objects(
    mut commands: Commands,
    mut object_query: Query<Entity, With<PuzzleColor>>,
    mut puzzle: ResMut<ColorPuzzle>,
    mut camera_query: Query<(&mut Camera, &mut BackgroundTranstion), With<Camera2d>>,
    mut last_click_query: Query<Entity, With<LastClick>>,
    mut memory_phase: ResMut<MemoryPhase>,
    mut round_intro: ResMut<RoundIntro>,
    mut start_level_events: MessageReader<StartLevelEvent>,
) {

    if start_level_events.read().next().is_none() {
        return;
    }

    round_intro.arm();

    // Recursive: a mosaic cell owns the nodes its piece is drawn from.
    for entity in object_query.iter_mut() {
        commands.entity(entity).despawn();
    }

    for entity in last_click_query.iter_mut() {
        commands.entity(entity).despawn();
    }

    let previous_background = puzzle.background_color();
    puzzle.generate_colors();

    let Ok((mut camera, mut background_transition)) = camera_query.single_mut() else {
        return;
    };

    // The ground sweeps every color on the board and lands on the answer's.
    // Each group melts into the ground as the sweep passes its color; the
    // answer melts last and stays gone. That is the information the player
    // needs to tell it from the cells that were empty all along.
    background_transition.sweep(
        previous_background,
        puzzle.sweep(),
        puzzle.transition_seconds,
    );
    camera.clear_color = ClearColorConfig::Custom(previous_background);

    if puzzle.game_mode.hides_colors() {
        // The sweep is part of showing the board, so the preview starts after
        // it. Counting the sweep as preview would make a late level's 0.7s
        // preview almost entirely ramp.
        memory_phase.begin(puzzle.preview_seconds() + puzzle.transition_seconds);
    } else {
        memory_phase.clear();
    }

    // Collect first: laying the board out needs the cell count, and writing the
    // resulting cell size back to the puzzle needs the borrow released.
    let mut cells = Vec::new();
    puzzle.for_each_cell(|index, color, is_correct_color, tile| {
        cells.push((index, color, is_correct_color, tile));
    });

    // Mosaic is laid out on the grid its pattern was generated for; every other
    // mode gets the irregular cut, which is what keeps the invisible piece from
    // being a hole at a predictable address.
    let grid = puzzle.mosaic_grid();
    let slots: Vec<(Vec2, Vec<Vec2>)> = match &grid {
        // A mosaic cell is a piece like any other; they are simply all the same
        // square, because that mode's question is how pieces meet.
        Some(grid) => (0..cells.len())
            .map(|index| {
                let corner = grid.cell_position(index);
                (
                    corner + Vec2::splat(grid.cell_size / 2.0),
                    square_corners(grid.cell_size),
                )
            })
            .collect(),
        None => puzzle
            .slots()
            .iter()
            .map(|piece| (piece.centre, piece.corners.clone()))
            .collect(),
    };

    // A rough board-wide size, for anything that needs one before a piece is
    // in hand.
    puzzle.shape_size = grid
        .as_ref()
        .map(|grid| grid.cell_size)
        .unwrap_or(puzzle.shape_size);

    let mut z = 0.0;
    for (index, color, is_correct_color, tile) in cells {
        let Some((centre, corners)) = slots.get(index).cloned() else {
            continue;
        };

        let shape = piece_shape(&corners);
        let arm_size = cell_size(&corners);

        // In a mosaic the cell is a plate the piece is drawn on, so every plate
        // is the same neutral color; in the color modes the piece *is* the
        // color.
        let plate = if tile.is_some() { theme::SURFACE } else { color };

        commands
            .spawn((
                ShapeBuilder::with(&shape).fill(Fill::color(plate)).build(),
                Transform::from_xyz(centre.x, centre.y, z),
                PuzzleColor {
                    index,
                    is_correct_color,
                    x: centre.x,
                    y: centre.y,
                    color,
                    corners,
                    tile,
                },
                PuzzleColorGame {},
            ))
            .with_children(|parent| {
                if let Some(tile) = tile {
                    spawn_tile_arms(parent, tile, arm_size, color);
                }
            });

        z += 0.1;
    }
}

pub fn start_puzzle_level(
    mut start_level_event_writer: MessageWriter<StartLevelEvent>,
    mut puzzle: ResMut<ColorPuzzle>,
    mut game_timer: ResMut<GameTimer>,
    mut game_history: ResMut<GameHistory>,
    mut pending_level_start: ResMut<PendingLevelStart>,
    mut memory_phase: ResMut<MemoryPhase>,
    mut round_intro: ResMut<RoundIntro>,
    window_query: Query<&Window, With<Window>>
) {
    // A hold left over from a miss in a previous run would swallow the first
    // pick of this one.
    pending_level_start.clear();
    memory_phase.clear();
    round_intro.clear();

    let Ok(window) = window_query.single() else {
        return;
    };
    puzzle.set_window_size(window.width(), window.height());

    // Infinite runs never reach the timer-expiry path, so without this the
    // game-over screen would report whichever mode was played last.
    game_history.set_game_mode(puzzle.game_mode);

    if game_timer.timer.duration().as_secs_f32() != puzzle.start_seconds {
        game_timer.timer = puzzle.setup_timer();
    }



    if game_timer.timer.is_finished() {
        game_timer.timer = puzzle.setup_timer();
    }

    if game_timer.timer.is_paused() {
        game_timer.timer.unpause();
    }

    start_level_event_writer.write(StartLevelEvent);

}

pub fn handle_new_game_event(
    mut new_game_event_reader: MessageReader<NewGameEvent>,
    mut puzzle: ResMut<ColorPuzzle>,
    mut game_timer: ResMut<GameTimer>,
    mut game_history: ResMut<GameHistory>,
    mut app_state_next_state: ResMut<NextState<crate::AppState>>,
    window_query: Query<&Window, With<Window>>
) {
    let events = new_game_event_reader.read().next();

    if events.is_none() {
        return;
    }

    let event = events.unwrap();

    let Ok(window) = window_query.single() else {
        return;
    };
    puzzle.setup(&event.game_mode);
    puzzle.set_window_size(window.width(), window.height());

    puzzle.reset();

    if game_timer.timer.duration().as_secs_f32() != puzzle.start_seconds {
        game_timer.timer = puzzle.setup_timer();
    } else if game_timer.timer.is_finished() {
        game_timer.timer = puzzle.setup_timer();
    }

    if game_timer.timer.is_paused() {
        game_timer.timer.unpause();
    }

    game_history.reset();
    game_history.set_game_mode(event.game_mode);

    // Entering Game runs `start_puzzle_level`, which sends `StartLevelEvent`
    // itself. Sending one here too would queue a second board generation and
    // the player would see the round regenerate twice.
    app_state_next_state.set(crate::AppState::Game);
}

#[derive(Component)]
pub struct PuzzleColorGame;

pub fn despaw_objects(
    mut commands: Commands,
    mut object_query: Query<Entity, With<PuzzleColorGame>>,
    mut puzzle: ResMut<ColorPuzzle>,
    mut pending_level_start: ResMut<PendingLevelStart>,
    mut memory_phase: ResMut<MemoryPhase>,
    mut round_intro: ResMut<RoundIntro>,
) {

    pending_level_start.clear();
    memory_phase.clear();
    round_intro.clear();
    round_intro.clear();

    for entity in object_query.iter_mut() {
        commands.entity(entity).despawn();
    }

    puzzle.generate_colors();
}
