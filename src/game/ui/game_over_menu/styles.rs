use bevy::prelude::*;

use crate::theme;

pub use crate::theme::{
    BUTTON, BUTTON_HOVERED, BUTTON_PRESSED, BUTTON_PRIMARY, BUTTON_PRIMARY_HOVERED,
    BUTTON_PRIMARY_PRESSED, SCRIM, SURFACE,
};

pub const GAME_OVER_MENU_STYLE: Style = Style {
    position_type: PositionType::Absolute,
    display: Display::Flex,
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
    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
    padding: UiRect::all(Val::Px(theme::SPACE_LG)),
    gap: Size::new(Val::Px(theme::SPACE_SM), Val::Px(theme::SPACE_SM)),
    ..Style::DEFAULT
};

/// Row holding one stat label and its value, pushed to opposite edges so the
/// numbers line up in a column the eye can scan.
pub const STAT_ROW_STYLE: Style = Style {
    flex_direction: FlexDirection::Row,
    justify_content: JustifyContent::SpaceBetween,
    align_items: AlignItems::Center,
    size: Size::new(Val::Percent(80.0), Val::Px(30.0)),
    ..Style::DEFAULT
};

pub const BUTTON_STYLE: Style = Style {
    size: Size::new(Val::Percent(80.0), Val::Px(theme::TOUCH_TARGET)),
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    margin: UiRect::all(Val::Px(theme::SPACE_XS)),
    ..Style::DEFAULT
};

/// Primary action: taller than the rest so the thumb finds it first.
pub const PRIMARY_BUTTON_STYLE: Style = Style {
    size: Size::new(Val::Percent(80.0), Val::Px(60.0)),
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    margin: UiRect::all(Val::Px(theme::SPACE_SM)),
    ..Style::DEFAULT
};

pub fn get_title_text_style(asset_server: &Res<AssetServer>) -> TextStyle {
    theme::text_title(asset_server)
}

pub fn get_resume_text_style(asset_server: &Res<AssetServer>) -> TextStyle {
    theme::text_body(asset_server)
}

pub fn get_label_text_style(asset_server: &Res<AssetServer>) -> TextStyle {
    theme::text_label(asset_server)
}

pub fn get_button_text_style(asset_server: &Res<AssetServer>) -> TextStyle {
    theme::text_button(asset_server)
}
