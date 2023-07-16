use bevy::prelude::*;
pub struct GameUiPlugin;

#[derive(Component)]
pub struct StartGameButton {
    pub is_hovered: bool,
    pub is_pressed: bool,
    pub is_released: bool,
    
}

impl bevy::app::Plugin for GameUiPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_startup_system(draw_ui)
            .add_system(handle_mouse_button);
    }
}

fn handle_mouse_button(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut button_query: Query<(&Transform, &mut StartGameButton)>,
    mouse_button_input: Res<Input<MouseButton>>,
) {

    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    {
   
        for (transform, mut button) in button_query.iter_mut() {
            button.is_pressed = false;
            button.is_released = false;
            if button.is_hovered && mouse_button_input.just_pressed(MouseButton::Left) {
                button.is_pressed = true;
            }
            if button.is_hovered && mouse_button_input.just_released(MouseButton::Left) {
                button.is_released = true;
            }
        }
    }
}

fn draw_ui(mut command : Commands, asset_server : Res<AssetServer>, mut materials : ResMut<Assets<ColorMaterial>>) {
    command.spawn(TextBundle {
        text: Text::from_section(
            "Hello Bevy!",
            TextStyle {
                font: asset_server.load("digital7mono.ttf"),
                font_size: 60.0,
                color: Color::WHITE,
            },
        ),
        ..Default::default()
    }).insert(StartGameButton {
        is_hovered: false,
        is_pressed: false,
        is_released: false,
    });
}
