use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy::core_pipeline::clear_color::ClearColorConfig;
use crate::events::InteractionAnimationEvent;
use crate::feedback::BannerEvent;
use crate::theme;
use super::components::*;
use crate::systems::BackgroundTranstion;
use crate::wfc::Tile;

#[derive(Component)]
pub struct LastClick;

pub fn player_interaction(
    mut commands: Commands,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform, &BackgroundTranstion)>,
    event_click  : Res<Input<MouseButton>>,
    touches: Res<Touches>,
    mut object_query: Query<(&Transform, &PuzzleColor, &mut Fill), With<PuzzleColor>>,
    ui_interaction_query: Query<&Interaction>,
    mut puzzle: ResMut<ColorPuzzle>,
    mut game_timer: ResMut<GameTimer>,
    mut pending_level_start: ResMut<PendingLevelStart>,
    memory_phase: Res<MemoryPhase>,
    mut start_level_event_writer: EventWriter<StartLevelEvent>,
    mut last_interraction_event_writer: EventWriter<LastInteractionEvent>,
    mut interaction_animation_event_writer: EventWriter<InteractionAnimationEvent>,
    mut banner_event_writer: EventWriter<BannerEvent>,
    last_click_query: Query<Entity, With<LastClick>>,
) {

    let window = windows.single();
    let (camera, camera_transform, background_transtion) = camera_q.single();

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

    if !background_transtion.is_in_transition() && (event_click.just_released(MouseButton::Left) || touches.any_just_pressed()) {
        // Bevy 0.10's `viewport_to_world_2d` maps its argument straight to NDC
        // without flipping y, so it wants a position whose origin is the
        // *bottom* left — which is what `cursor_position` returns. Touches are
        // the odd one out: `bevy_winit` passes them through in winit's
        // top-left convention, so they need converting first.
        //
        // The touch branch used to convert after the fact instead, by negating
        // the resulting world y. That happens to agree with this only while the
        // camera sits exactly at the origin; converting the input is what
        // actually means "the point the finger is on".
        let screen_position = if let Some(touch) = touches.first_pressed_position() {
            Vec2::new(touch.x, window.height() - touch.y)
        } else if let Some(cursor) = window.cursor_position() {
            cursor
        } else {
            return;
        };

        let Some(world_position) = camera.viewport_to_world_2d(camera_transform, screen_position) else {
            return;
        };

        for last_click in last_click_query.iter() {
            commands.entity(last_click).despawn_recursive();
        }

        let mut scored = false;
        let mut colors = Vec::new();
        let mut correct_position = None;
        let mut correct_size = puzzle.shape_size;

        for (transform, puzzle_color, _) in object_query.iter_mut() {
            colors.push(puzzle_color.as_level_color());

            if puzzle_color.is_correct_color {
                correct_position = Some(Vec2::new(puzzle_color.x, puzzle_color.y));
                correct_size = puzzle_color.size;
            }

            if mouse_hover(transform.translation, world_position, puzzle_color.size) && puzzle_color.is_correct_color {
                scored = true;
            }
        }

        let mut bonus_seconds = 0.0;
        let mut leveled_up = false;

        if scored {
            if puzzle.game_mode == GameMode::TimeTrial {
                bonus_seconds = puzzle.get_seconds_added_per_success();
            }

            leveled_up = puzzle.increase_score(&mut game_timer);
        }

        interaction_animation_event_writer.send(InteractionAnimationEvent {
            position: world_position,
            scored,
            bonus_seconds,
            // Only meaningful on a miss, where it drives the answer reveal.
            correct_position,
            shape_size: correct_size,
        });

        if leveled_up {
            banner_event_writer.send(BannerEvent::large(
                format!("NIVEL {}", puzzle.level()),
                theme::ACCENT,
            ));
        }

        last_interraction_event_writer.send(LastInteractionEvent::new(
            world_position,
            puzzle.get_correct_color_index(),
            colors,
            scored,
        ));

        if scored {
            // Keep the momentum: a correct pick moves straight on.
            start_level_event_writer.send(StartLevelEvent);
        } else {
            // Hold the board so the reveal has something to point at.
            pending_level_start.hold();

            // A blank board has nothing to learn from. Put the colors back for
            // the length of the hold, so a missed Memory round still shows the
            // player what they were supposed to have remembered.
            if memory_phase.is_hidden() {
                for (_, puzzle_color, mut fill) in object_query.iter_mut() {
                    *fill = Fill::color(puzzle_color.color);
                }
            }
        }
    }

}

/// Draws a mosaic piece as children of its cell.
///
/// One node per arm plus a hub at the centre, all in the round's color. Kept as
/// children so the cell entity stays exactly what the rest of the game expects:
/// one `PuzzleColor` per cell, positioned at its bottom-left corner, which is
/// what the hit test and the answer reveal are written against.
fn spawn_tile_arms(parent: &mut ChildBuilder, tile: Tile, size: f32, color: Color) {
    let edges = tile.edges();

    // A piece with no arms is a blank plate. Drawing its hub anyway would put a
    // mark on every empty cell, which reads as a piece and gives the player a
    // pattern that is not there.
    if !edges.iter().any(|edge| *edge) {
        return;
    }

    let arm_width = size * 0.32;
    let centre = size / 2.0;
    let half_arm = arm_width / 2.0;

    // The hub joins the arms, so a corner reads as one bent pipe rather than
    // two rectangles that happen to meet.
    let hub = shapes::Rectangle {
        extents: Vec2::new(arm_width, arm_width),
        origin: shapes::RectangleOrigin::Center,
    };

    parent.spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(&hub),
            transform: Transform::from_xyz(centre, centre, 0.01),
            ..default()
        },
        Fill::color(color),
    ));

    // Each arm runs from inside the hub out to the edge it points at, so the
    // two overlap and the joint has no seam. `wfc` indexes edges clockwise from
    // the top, and the board's y axis grows upward.
    let reach = centre + half_arm;
    let offset = (centre - half_arm) / 2.0;
    let arms = [
        (Vec2::new(arm_width, reach), Vec2::new(centre, centre + offset)),
        (Vec2::new(reach, arm_width), Vec2::new(centre + offset, centre)),
        (Vec2::new(arm_width, reach), Vec2::new(centre, centre - offset)),
        (Vec2::new(reach, arm_width), Vec2::new(centre - offset, centre)),
    ];

    for (edge, (extents, position)) in edges.iter().zip(arms) {
        if !edge {
            continue;
        }

        let arm = shapes::Rectangle {
            extents,
            origin: shapes::RectangleOrigin::Center,
        };

        parent.spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&arm),
                transform: Transform::from_xyz(position.x, position.y, 0.01),
                ..default()
            },
            Fill::color(color),
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
    mut square_query: Query<&mut Fill, With<PuzzleColor>>,
) {
    if !memory_phase.tick(time.delta()) {
        return;
    }

    let hidden = puzzle.hidden_color();
    for mut fill in square_query.iter_mut() {
        *fill = Fill::color(hidden);
    }
}

/// Starts the next round once a post-miss hold expires.
pub fn advance_pending_level(
    time: Res<Time>,
    mut pending_level_start: ResMut<PendingLevelStart>,
    mut start_level_event_writer: EventWriter<StartLevelEvent>,
) {
    if pending_level_start.tick(time.delta()) {
        start_level_event_writer.send(StartLevelEvent);
    }
}


pub fn render_game_history(
    mut commands: Commands,
    game_history: Res<GameHistory>,
    mut render_game_history_events: EventReader<RenderLevelHistoryEvent>,
    mut object_query: Query<Entity, With<PuzzleColor>>,
    mut last_click_query: Query<Entity, With<LastClick>>,
    mut camera_query: Query<(&mut Camera2d, &mut BackgroundTranstion), With<Camera>>,
) {

    let render_event = render_game_history_events.iter().next();

    if render_event.is_none() {
        return;
    }

    let event = render_event.unwrap();

    for entity in object_query.iter_mut() {
        commands.entity(entity).despawn_recursive();
    }

    for entity in last_click_query.iter_mut() {
        commands.entity(entity).despawn_recursive();
    }


    let level_history = game_history.get_level_history(event.index);
    let (mut camera, mut background_transition) = camera_query.single_mut();

    // A replay is read, not played, so it sits on the plain app background
    // rather than reproducing the round's tint.
    background_transition.reset();
    background_transition.set_start_color(theme::BACKGROUND);
    background_transition.set_end_color(theme::BACKGROUND);

    camera.clear_color = ClearColorConfig::Custom(theme::BACKGROUND);

    let mut z = 0.0;
    level_history.for_each_color(|index, color| {
        let fill = Fill::color(color.color);
        let is_correct_color = color.is_correct_color;
        // Each square remembers its own size, so a round played on a different
        // window size still replays as the board the player actually saw.
        let size = color.size;

        let shape = shapes::Rectangle {
            extents: Vec2::new(size, size),
            origin: shapes::RectangleOrigin::BottomLeft,
        };

        let plate = if color.tile.is_some() {
            Fill::color(theme::SURFACE)
        } else {
            fill
        };

        commands
            .spawn((
                ShapeBundle {
                    path: GeometryBuilder::build_as(&shape),
                    transform: Transform::from_xyz(
                        color.x,
                        color.y,
                        z
                    ),
                    ..default()
                },
                plate,
                PuzzleColor { index, is_correct_color:  color.is_correct_color, x : color.x , y:  color.y, color: color.color.clone(), size, tile: color.tile },
            ))
            .with_children(|parent| {
                if let Some(tile) = color.tile {
                    spawn_tile_arms(parent, tile, size, color.color);
                }
            });

        if is_correct_color {
            let inner_shape =  shapes::Rectangle {
                extents: Vec2::new(size - 20.0, size - 20.0),
                origin: shapes::RectangleOrigin::BottomLeft,
            };
            commands .spawn((
                ShapeBundle {
                    path: GeometryBuilder::build_as(&inner_shape),
                    transform: Transform::from_xyz(
                        color.x + 10.0,
                        color.y + 10.0,
                        z + 0.01
                    ),
                    ..default()
                },
                Fill::color(Color::WHITE),
                LastClick,
            ));
        }
        z += 0.1;
    });

    let shape_clicked_position =  shapes::Rectangle {
        extents: Vec2::new(30.0, 30.0),
        origin: shapes::RectangleOrigin::Center ,
    };

    commands .spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(&shape_clicked_position),
            transform: Transform::from_xyz(
                level_history.clicked_position.x,
                level_history.clicked_position.y,
                1.0
            ),
            ..default()
        },
        Fill::color(theme::DANGER),
        LastClick,
    ));

}

pub fn store_last_interaction_state(
    mut last_interaction_events: EventReader<LastInteractionEvent>,
    mut game_history: ResMut<GameHistory>,
    mut banner_event_writer: EventWriter<BannerEvent>,
) {
    let level_history =last_interaction_events.iter().next();

    if level_history.is_none() {
        return;
    }

    let event = level_history.unwrap();

    game_history.add_level(event.level_history());

    // Streaks are the cheapest motivation in the game: the player builds
    // something they then don't want to lose. Call out the milestones so the
    // thing they stand to lose is salient while they still have it.
    if let Some(label) = streak_milestone_label(game_history.current_streak()) {
        banner_event_writer.send(BannerEvent::new(label, theme::SUCCESS));
    }
}

/// Whether a pick landed inside a square.
///
/// `translation` is the square's bottom-left corner: the shapes are built with
/// `RectangleOrigin::BottomLeft`.
///
/// This used to test a 30x30 box growing up and to the right of the pick rather
/// than the pick itself, which let a tap in the gap below-left of the target
/// score as a hit. Harmless when squares were scattered; on a grid it would
/// hand out points for tapping the wrong square.
fn mouse_hover(translation: Vec3, point: Vec2, shape_size : f32) -> bool {
    point.x >= translation.x
        && point.x <= translation.x + shape_size
        && point.y >= translation.y
        && point.y <= translation.y + shape_size
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
    mut camera_query: Query<(&mut Camera2d, &mut BackgroundTranstion), With<Camera>>,
    time : Res<Time>,
) {

    let (mut camera, mut background_transition) = camera_query.single_mut();

    if background_transition.is_in_transition() {
        camera.clear_color = ClearColorConfig::Custom(background_transition.get_current_color());
        background_transition.update(time.delta_seconds());
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
    mut app_state_next_state: ResMut<NextState<crate::AppState>>,
    time : Res<Time>,
) {
    if !puzzle.game_mode.is_timed() {
        return;
    }

    game_timer.timer.tick(time.delta());

    if game_timer.timer.finished() {
        game_history.set_game_mode(puzzle.game_mode);
        game_history.set_total_time(game_timer.timer.duration().as_secs_f32());
        app_state_next_state.set(crate::AppState::GameOverResume);
    }
}

pub fn spawn_objects(
    mut commands: Commands,
    mut object_query: Query<Entity, With<PuzzleColor>>,
    mut puzzle: ResMut<ColorPuzzle>,
    mut camera_query: Query<(&mut Camera2d, &mut BackgroundTranstion), With<Camera>>,
    mut last_click_query: Query<Entity, With<LastClick>>,
    mut memory_phase: ResMut<MemoryPhase>,
    mut start_level_events: EventReader<StartLevelEvent>,
) {

    if start_level_events.iter().next().is_none() {
        return;
    }

    // Recursive: a mosaic cell owns the nodes its piece is drawn from.
    for entity in object_query.iter_mut() {
        commands.entity(entity).despawn_recursive();
    }

    for entity in last_click_query.iter_mut() {
        commands.entity(entity).despawn_recursive();
    }

    let previous_background = puzzle.background_color();
    puzzle.generate_colors();

    let (mut camera, mut background_transition) = camera_query.single_mut();

    // The background is the app's dark ground with a hint of this round's hue,
    // never a square's color. Painting it with the target used to make the
    // correct square invisible, which turned the game into "find the gap".
    background_transition.reset();
    background_transition.set_end_color(puzzle.background_color());
    background_transition.set_start_color(previous_background);
    background_transition.set_time(puzzle.transition_seconds);
    camera.clear_color = ClearColorConfig::Custom(previous_background);

    if puzzle.game_mode.hides_colors() {
        memory_phase.begin(puzzle.preview_seconds());
    } else {
        memory_phase.clear();
    }

    // Collect first: laying the board out needs the cell count, and writing the
    // resulting cell size back to the puzzle needs the borrow released.
    let mut cells = Vec::new();
    puzzle.for_each_cell(|index, color, is_correct_color, tile| {
        cells.push((index, color, is_correct_color, tile));
    });

    let grid = puzzle.round_grid();

    // Everything downstream — the answer reveal, the replay, the hit test —
    // measures squares by this.
    puzzle.shape_size = grid.cell_size;

    let shape = shapes::Rectangle {
        extents: Vec2::new(grid.cell_size, grid.cell_size),
        origin: shapes::RectangleOrigin::BottomLeft,
    };

    let mut z = 0.0;
    for (index, color, is_correct_color, tile) in cells {
        let position = grid.cell_position(index);

        // In a mosaic the cell is a plate the piece is drawn on, so every plate
        // is the same neutral color; in the color modes the cell *is* the
        // color.
        let plate = if tile.is_some() { theme::SURFACE } else { color };

        commands
            .spawn((
                ShapeBundle {
                    path: GeometryBuilder::build_as(&shape),
                    transform: Transform::from_xyz(position.x, position.y, z),
                    ..default()
                },
                Fill::color(plate),
                PuzzleColor {
                    index,
                    is_correct_color,
                    x: position.x,
                    y: position.y,
                    color,
                    size: grid.cell_size,
                    tile,
                },
                PuzzleColorGame {},
            ))
            .with_children(|parent| {
                if let Some(tile) = tile {
                    spawn_tile_arms(parent, tile, grid.cell_size, color);
                }
            });

        z += 0.1;
    }
}

pub fn start_puzzle_level(
    mut start_level_event_writer: EventWriter<StartLevelEvent>,
    mut puzzle: ResMut<ColorPuzzle>,
    mut game_timer: ResMut<GameTimer>,
    mut game_history: ResMut<GameHistory>,
    mut pending_level_start: ResMut<PendingLevelStart>,
    mut memory_phase: ResMut<MemoryPhase>,
    window_query: Query<&Window, With<Window>>
) {
    // A hold left over from a miss in a previous run would swallow the first
    // pick of this one.
    pending_level_start.clear();
    memory_phase.clear();

    let window = window_query.single();
    puzzle.set_window_size(window.width(), window.height());

    // Infinite runs never reach the timer-expiry path, so without this the
    // game-over screen would report whichever mode was played last.
    game_history.set_game_mode(puzzle.game_mode);

    if game_timer.timer.duration().as_secs_f32() != puzzle.start_seconds {
        game_timer.timer = puzzle.setup_timer();
    }



    if game_timer.timer.finished() {
        game_timer.timer = puzzle.setup_timer();
    }

    if game_timer.timer.paused() {
        game_timer.timer.unpause();
    }

    start_level_event_writer.send(StartLevelEvent);

}

pub fn handle_new_game_event(
    mut new_game_event_reader: EventReader<NewGameEvent>,
    mut puzzle: ResMut<ColorPuzzle>,
    mut game_timer: ResMut<GameTimer>,
    mut game_history: ResMut<GameHistory>,
    mut app_state_next_state: ResMut<NextState<crate::AppState>>,
    window_query: Query<&Window, With<Window>>
) {
    let events = new_game_event_reader.iter().next();

    if events.is_none() {
        return;
    }

    let event = events.unwrap();

    let window = window_query.single();
    puzzle.setup(&event.game_mode);
    puzzle.set_window_size(window.width(), window.height());

    puzzle.reset();

    if game_timer.timer.duration().as_secs_f32() != puzzle.start_seconds {
        game_timer.timer = puzzle.setup_timer();
    } else if game_timer.timer.finished() {
        game_timer.timer = puzzle.setup_timer();
    }

    if game_timer.timer.paused() {
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
) {

    pending_level_start.clear();
    memory_phase.clear();

    for entity in object_query.iter_mut() {
        commands.entity(entity).despawn_recursive();
    }

    puzzle.generate_colors();
}
