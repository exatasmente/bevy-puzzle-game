use bevy::prelude::*;

pub const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
pub const TRANSPARENT_BUTTON: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);

pub const BUTTON_STYLE: Style = Style {
    size: Size::new(Val::Px(150.), Val::Px(60.0)),
    margin: UiRect::all(Val::Px(10.0)),
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    ..Style::DEFAULT
};

pub fn get_text_style(asset_server: &Res<AssetServer>) -> TextStyle {
    TextStyle {
        font: asset_server.load("digital7mono.ttf"),
        font_size: 12.0,
        color: Color::rgb(1.0, 1.0, 1.0),
    }
}
