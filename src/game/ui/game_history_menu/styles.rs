use bevy::prelude::*;

use crate::theme;

pub use crate::theme::{BUTTON, BUTTON_HOVERED, BUTTON_PRESSED, SCRIM, SURFACE};

pub const HISTORY_MENU_STYLE: Style = Style {
    position_type: PositionType::Absolute,
    display: Display::Flex,
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
    ..Style::DEFAULT
};

pub const HISTORY_MENU_CONTAINER_STYLE: Style = Style {
    display: Display::Flex,
    flex_direction: FlexDirection::Column,
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
    padding: UiRect::all(Val::Px(theme::SPACE_MD)),
    gap: Size::new(Val::Px(theme::SPACE_XS), Val::Px(theme::SPACE_XS)),
    ..Style::DEFAULT
};

/// One past round: swatch, label, outcome.
pub const HISTORY_CARD_STYLE: Style = Style {
    flex_direction: FlexDirection::Row,
    justify_content: JustifyContent::SpaceBetween,
    align_items: AlignItems::Center,
    size: Size::new(Val::Percent(88.0), Val::Px(theme::TOUCH_TARGET)),
    max_size: Size::new(Val::Px(420.0), Val::Auto),
    padding: UiRect::all(Val::Px(theme::SPACE_SM)),
    margin: UiRect::all(Val::Px(theme::SPACE_XS)),
    ..Style::DEFAULT
};

/// The color the round was actually asking for.
pub const SWATCH_STYLE: Style = Style {
    size: Size::new(Val::Px(32.0), Val::Px(32.0)),
    ..Style::DEFAULT
};

pub const BUTTON_STYLE: Style = Style {
    size: Size::new(Val::Percent(88.0), Val::Px(theme::TOUCH_TARGET)),
    max_size: Size::new(Val::Px(420.0), Val::Auto),
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    margin: UiRect::all(Val::Px(theme::SPACE_XS)),
    ..Style::DEFAULT
};

pub const PAGINATION_CONTAINER_STYLE: Style = Style {
    display: Display::Flex,
    flex_direction: FlexDirection::Row,
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    size: Size::new(Val::Percent(88.0), Val::Px(theme::TOUCH_TARGET)),
    gap: Size::new(Val::Px(theme::SPACE_SM), Val::Px(theme::SPACE_SM)),
    margin: UiRect::all(Val::Px(theme::SPACE_XS)),
    ..Style::DEFAULT
};

pub const BUTTON_PAGINATION_STYLE: Style = Style {
    size: Size::new(Val::Px(theme::TOUCH_TARGET), Val::Px(theme::TOUCH_TARGET)),
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    ..Style::DEFAULT
};

pub fn get_title_text_style(asset_server: &Res<AssetServer>) -> TextStyle {
    theme::text_title(asset_server)
}

pub fn get_button_text_style(asset_server: &Res<AssetServer>) -> TextStyle {
    theme::text_button(asset_server)
}

pub fn get_label_text_style(asset_server: &Res<AssetServer>) -> TextStyle {
    theme::text_label(asset_server)
}

pub fn get_pagination_button_text_style(asset_server: &Res<AssetServer>) -> TextStyle {
    theme::text(asset_server, theme::TEXT_MD, theme::ON_SURFACE)
}
