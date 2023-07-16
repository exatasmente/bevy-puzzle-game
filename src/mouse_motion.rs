use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
pub struct MouseMotionPlugin;
use crate::tilemap::TILE_SIZE;

#[derive(Component)]
struct MouseDrag {
    is_dragging: bool,
    current_drag_entity_id: Option<usize>,
}

impl MouseDrag {
    fn new() -> Self {
        Self {
            is_dragging: false,
            current_drag_entity_id: None,
        }
    }
}


impl bevy::app::Plugin for MouseMotionPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_event::<MouseMotion>()
            .add_startup_system(setup_mouse_motion)
            .add_system(update_mouse_motion.run_if(in_state(crate::AppState::Game)));
    }
}

#[derive(Component)]
pub struct Draggable;

fn setup_mouse_motion(mut commands: Commands) {
    commands.spawn(MouseDrag::new());
}

fn update_mouse_motion(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    event_click  : Res<Input<MouseButton>>,
    ctrl : Res<Input<KeyCode>>,
    mut mouse_drag_query : Query<&mut MouseDrag>,
    mut  query : Query<(&mut Transform, Entity), With<Draggable>>
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();
    let mut mouse_drag = mouse_drag_query.single_mut();
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    {
        if  ctrl.just_released(KeyCode::LControl) && mouse_drag.is_dragging {
            mouse_drag.is_dragging = false;
            mouse_drag.current_drag_entity_id = None;
        } else if ctrl.pressed(KeyCode::LControl) && event_click.pressed(MouseButton::Left) && !mouse_drag.is_dragging {
            mouse_drag.is_dragging = true;
        }
        
        let just_pressed = event_click.just_pressed(MouseButton::Left);
        
        if !mouse_drag.is_dragging && !just_pressed {
            return;
        }
        
    
        for (mut transform, entity) in query.iter_mut() {
            
            if mouse_drag.is_dragging {
                if mouse_drag.current_drag_entity_id.is_none() {
                    if !mouse_hover(transform.translation, world_position) {
                        continue;
                    }
                    mouse_drag.current_drag_entity_id = Some(entity.index() as usize);
                } else if mouse_drag.current_drag_entity_id.unwrap() != entity.index() as usize {
                    continue;
                }

                transform.translation.x += world_position.x - transform.translation.x;
                transform.translation.y += world_position.y - transform.translation.y;
                break;
            }
            
        }
    }
}
fn mouse_hover(translation: Vec3, delta: Vec2) -> bool {
    let x = translation.x;
    let y = translation.y;
    let delta_x = delta.x;
    let delta_y = delta.y;
    
    let bounding_box = (Vec2::new((x - (TILE_SIZE/2.0)) - 1.0, (y - (TILE_SIZE/2.0)))- 1.0, Vec2::new((x + (TILE_SIZE/2.0)) - 1.0, (y + (TILE_SIZE/2.0)- 1.0)));

    
    if (bounding_box.0.x..bounding_box.1.x)
    .contains(&delta_x)
    && (bounding_box.0.y..bounding_box.1.y)
      .contains(&delta_y)
    {
        println!("HOVER: {:?} {:?}", bounding_box, delta);
        return true;
    }

    
    
    false
}

