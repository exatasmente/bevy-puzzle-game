use bevy::prelude::*;

pub const BACKGROUND_COLOR: Color = Color::rgba(0.25, 0.25, 0.25, 0.5);

pub const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
pub const TRANSPARENT_BUTTON: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);
pub const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

pub const GAME_OVER_MENU_STYLE: Style = Style {
    position_type: PositionType::Absolute, // Needed to display separately from HUD.
    display: Display::Flex,                // Hidden by Default
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,    
    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
    ..Style::DEFAULT
};

pub const GAME_OVER_MENU_CONTAINER_STYLE: Style = Style {
    display: Display::Flex,
    flex_direction: FlexDirection::Column,
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    size: Size::new(Val::Percent(100.), Val::Percent(100.)),
    gap: Size::new(Val::Px(8.0), Val::Px(8.0)),
    ..Style::DEFAULT
};


pub const BUTTON_HISTORY_STYLE: Style = Style {
    size: Size::new(Val::Percent(50.), Val::Px(80.0)),
    margin: UiRect::all(Val::Px(10.0)),
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    ..Style::DEFAULT
};

pub const PAGINATION_CONTAINER_STYLE: Style = Style {
    display: Display::Flex,
    flex_direction: FlexDirection::Row,
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    size: Size::new(Val::Percent(100.), Val::Px(100.)),
    gap: Size::new(Val::Px(8.0), Val::Px(8.0)),
    margin: UiRect::all(Val::Px(10.0)),
    ..Style::DEFAULT
};

pub const BUTTON_PAGINATION_STYLE: Style = Style {
    size: Size::new(Val::Px(60.0), Val::Px(60.0)),
    display: Display::Flex,
    flex_direction: FlexDirection::Column,
    margin: UiRect::all(Val::Px(4.0)),
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    ..Style::DEFAULT
};

pub const PAGINATION_TEXT_STYLE: Style = Style {
    display: Display::Flex,
    flex_direction : FlexDirection::Row,
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    ..Style::DEFAULT
};

pub fn get_button_text_style(asset_server: &Res<AssetServer>,) -> TextStyle {
    TextStyle {
        font: asset_server.load("digital7mono.ttf"),
        font_size: 12.0,
        color: Color::rgb(1.0, 1.0, 1.0),
    }
}


pub fn get_pagination_button_text_style(asset_server: &Res<AssetServer>,) -> TextStyle {
    TextStyle {
        font: asset_server.load("digital7mono.ttf"),
        font_size: 20.0,
        color: Color::rgb(1.0, 1.0, 1.0),
    }
}
