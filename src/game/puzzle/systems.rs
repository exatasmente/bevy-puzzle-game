use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy::core_pipeline::clear_color::ClearColorConfig;

use super::components::*;
use crate::systems::BackgroundTranstion;

const SQUARE_SIZE: f32 = 200.0;
const N_OF_COLS: usize = 6;

#[derive(Component)]
pub struct LastClick;

pub fn player_interaction(
    mut commands: Commands,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform, &BackgroundTranstion)>,
    event_click  : Res<Input<MouseButton>>,
    touches: Res<Touches>,
    mut object_query: Query<(&Transform,&mut Fill, &PuzzleColor), With<PuzzleColor>>,
    mut puzzle: ResMut<ColorPuzzle>,
    mut start_level_event_writer: EventWriter<StartLevelEvent>,
    last_click_query: Query<Entity, With<LastClick>>,
) {

    let window = windows.single();
    let (camera, camera_transform, background_transtion) = camera_q.single();
    
    if !background_transtion.is_in_transition() && (event_click.just_released(MouseButton::Left) || touches.any_just_pressed()) {
        let is_touch = touches.first_pressed_position().is_some();
        let world_position;

        if is_touch {
            let temp_world_position =  camera.viewport_to_world_2d(camera_transform, touches.first_pressed_position().unwrap()).unwrap();
            world_position = Vec2::new(temp_world_position.x, temp_world_position.y * -1.0);
        } else {
            world_position = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor)).unwrap();
        }

        for last_click in last_click_query.iter() {
            commands.entity(last_click).despawn_recursive();
        }

        let shape =  shapes::Rectangle {
            extents: Vec2::new(
                10.0,
                10.0,
            ),
            origin: shapes::RectangleOrigin::BottomLeft,
        };

        commands
            .spawn(( 
                ShapeBundle {
                    path: GeometryBuilder::build_as(&shape),
                    transform: Transform::from_xyz(world_position.x, world_position.y , 1.0),
                    ..default()
                },
                Fill::color(Color::RED),
                LastClick,        
            )
        );
        

        for (transform,mut fill, puzzle_color) in object_query.iter_mut() {
            if mouse_hover(transform.translation, world_position, puzzle.shape_size) && puzzle_color.is_correct_color {
                puzzle.increase_score();
            } else if puzzle_color.is_correct_color {
                fill.color = Color::WHITE;
                commands
                .spawn(( 
                    ShapeBundle {
                        path: GeometryBuilder::build_as(&shape),
                        transform: Transform::from_xyz(transform.translation.x + puzzle.shape_size /2.0, transform.translation.y  + puzzle.shape_size /2.0 , 1.0),
                        ..default()
                    },
                    Fill::color(Color::BLACK),
                    LastClick,        
                ));
            } else {
                fill.color = Color::BLACK;
                commands
                .spawn(( 
                    ShapeBundle {
                        path: GeometryBuilder::build_as(&shape),
                        transform: Transform::from_xyz(transform.translation.x + puzzle.shape_size /2.0, transform.translation.y  + puzzle.shape_size /2.0 , 1.0),
                        ..default()
                    },
                    Fill::color(Color::WHITE),
                    LastClick,        
                ));
            }
        

        }

        start_level_event_writer.send(StartLevelEvent);    

            
    }
 
}


fn mouse_hover(translation: Vec3, delta: Vec2, shape_size : f32) -> bool {
    let x1 = translation.x;
    let y1 = translation.y;
    let x2 = translation.x + shape_size;
    let y2 = translation.y + shape_size;
    let x3 = delta.x;
    let y3 = delta.y;
    let x4 = x3 + 10.0;
    let y4 = y3 + 10.0;
    println!("x1: {}, y1: {}, x2: {}, y2: {}, x3: {}, y3: {}, x4: {}, y4: {}, inter : {}", x1, y1, x2, y2, x3, y3, x4, y4, cord_is_intersecting(x1, y1, x2, y2, x3, y3, x4, y4));
    cord_is_intersecting(x1, y1, x2, y2, x3, y3, x4, y4)
}


fn random_range(min: f32, max: f32) -> f32 {
    rand::random::<f32>() * (max - min) + min
}


#[derive(Resource, Reflect, Debug)]
pub struct GameTimer {
    pub timer: Timer,
}



pub fn background_transition(
    mut commands: Commands,
    mut camera_query: Query<(&mut Camera2d, &mut BackgroundTranstion), With<Camera>>,
    time : Res<Time>,
    mut game_timer: ResMut<GameTimer>,
) {

    let (mut camera, mut background_transition) = camera_query.single_mut();
    
    if background_transition.is_in_transition() {
        if !game_timer.timer.paused() {
            game_timer.timer.pause()
        }
        
        camera.clear_color = ClearColorConfig::Custom(background_transition.get_current_color());
        background_transition.update(time.delta_seconds());
    } else if game_timer.timer.paused() {
        game_timer.timer.unpause();
    }
}

#[derive(Component)]
pub struct RemainingTime;

pub fn render_remaining_time(
    mut commands: Commands,
    mut query: Query<&mut Text, With<RemainingTime>>,
    asset_server: Res<AssetServer>,
    mut game_timer: ResMut<GameTimer>,
    puzzle: Res<ColorPuzzle>,
    mut app_state_next_state: ResMut<NextState<crate::AppState>>,
    time : Res<Time>,
) {

    game_timer.timer.tick(time.delta());
    
    if query.iter_mut().next().is_none() {
        let font = asset_server.load("digital7mono.ttf");
        let text_style = TextStyle {
            font: font.clone(),
            font_size: 20.0,
            color: Color::BLACK,
        };
        let text_alignment = TextAlignment::Center;
        commands.spawn((
            Text2dBundle {
                text: Text::from_section(format!("Time : {:02.0} ", game_timer.timer.remaining_secs()), text_style.clone())
                    .with_alignment(text_alignment),
                    transform: Transform::from_translation(Vec3::new(0.0, (puzzle.get_height() / 2.0) * -1.0, 2.0)),
                ..default()
            },
            RemainingTime,
        ));

        return;
    }

    let mut text = query.single_mut();

    if game_timer.timer.finished() {
        app_state_next_state.set(crate::AppState::GameOver);
    }

    text.sections[0].value = format!("Time : {:02.0} ", game_timer.timer.remaining_secs());

  
}

fn cord_is_intersecting(
    x1: f32, y1: f32, x2: f32, y2: f32,
    x3: f32, y3: f32, x4: f32, y4: f32,
) -> bool {
    !(x1 > x4 || x3 > x2 || y1 > y4 || y3 > y2)
}

#[derive(Component)]
pub struct ScoreText;

pub fn spawn_objects(
    mut commands: Commands,
    mut object_query: Query<Entity, With<PuzzleColor>>,
    mut score_query: Query<Entity, With<ScoreText>>,
    mut puzzle: ResMut<ColorPuzzle>,
    mut camera_query: Query<(&mut Camera2d, &mut BackgroundTranstion), With<Camera>>,
    mut start_level_events: EventReader<StartLevelEvent>,
    asset_server: Res<AssetServer>,
) {
    
    if start_level_events.iter().next().is_none() {
        return;
    }

    for entity in object_query.iter_mut() {
        commands.entity(entity).despawn();        
    }

    for entity in score_query.iter_mut() {
        commands.entity(entity).despawn();        
    }

    let font = asset_server.load("digital7mono.ttf");
    
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 20.0,
        color: Color::BLACK,
        
    };
    let text_alignment = TextAlignment::Center;
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(format!("Score : {} ", puzzle.get_score().to_string()), text_style.clone())
                .with_alignment(text_alignment),
            transform: Transform::from_translation(Vec3::new(0.0, puzzle.get_height() / 2.0, 2.0)),
            ..default()
        },
        ScoreText,
    ));

    let current_color = puzzle.get_color();
    puzzle.generate_colors();

    let (mut camera, mut background_transition) = camera_query.single_mut();

    background_transition.reset();
    background_transition.set_end_color(puzzle.get_color());
    background_transition.set_start_color(current_color);
    background_transition.set_time(puzzle.transition_seconds);
    camera.clear_color = ClearColorConfig::Custom(puzzle.get_color());

    let mut n_of_rows = 0;

    let mut used_spaces = Vec::new();
    let mut z = 0.0;
    puzzle.for_each_color( |index,color, is_correct_color| {
        
        let shape =  shapes::Rectangle {
            extents: Vec2::new(
                puzzle.shape_size,
                puzzle.shape_size,
            ),
            origin: shapes::RectangleOrigin::BottomLeft,
        };

        if index % N_OF_COLS == 0 {
            n_of_rows += 1;
        }

        let mut x = random_range((puzzle.get_width() / 2.0 ) * -1.0 , puzzle.get_width()  / 2.0 );
        let mut y = random_range((puzzle.get_height()  / 2.0 ) * -1.0 , puzzle.get_height()  / 2.0 );    
        let mut exists = used_spaces.iter().any(|(start_x, start_y, end_x, end_y)| {
            cord_is_intersecting(
                x, y, x + puzzle.shape_size, y + puzzle.shape_size,
                *start_x, *start_y, *end_x, *end_y
            )
        });

        let mut max_tries = 100;
        while exists && max_tries > 0 {
            x = random_range((puzzle.get_width()  / 2.0 ) * -1.0 , puzzle.get_width()  / 2.0 );
            y = random_range((puzzle.get_height()  / 2.0 ) * -1.0 , puzzle.get_height()  / 2.0 );    

            exists = used_spaces.iter().any(|(start_x, start_y, end_x, end_y)| {
                cord_is_intersecting(
                    x, y, x + puzzle.shape_size, y + puzzle.shape_size,
                    *start_x, *start_y, *end_x, *end_y
                )
            });
            max_tries -= 1;
        }

        used_spaces.push((x,y, x + puzzle.shape_size, y + puzzle.shape_size));

        commands
            .spawn(( 
                ShapeBundle {
                    path: GeometryBuilder::build_as(&shape),
                    transform: Transform::from_xyz(x , y , z),
                    ..default()
                },
                Fill::color(color),
                PuzzleColor { index, is_correct_color},
                
            )
        );
        z += 0.1;
    });
    

}

pub fn start_puzzle_level(
    mut commands: Commands,
    mut start_level_event_writer: EventWriter<StartLevelEvent>,
    mut puzzle: ResMut<ColorPuzzle>,
    window_query: Query<&Window, With<Window>>
) {
    commands.insert_resource(GameTimer { timer: puzzle.setup_timer()});
    start_level_event_writer.send(StartLevelEvent);    

    let window = window_query.single();
    puzzle.set_window_size(window.width(), window.height());
    
}


pub fn despaw_objects(
    mut commands: Commands,
    mut object_query: Query<Entity, With<PuzzleColor>>,
    mut puzzle: ResMut<ColorPuzzle>,
) {


    for entity in object_query.iter_mut() {
        commands.entity(entity).despawn();        
    }

    puzzle.generate_colors();
}
