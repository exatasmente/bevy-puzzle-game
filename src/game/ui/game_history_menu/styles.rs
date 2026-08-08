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
    padding: UiRect::all(Val::Px(theme::SPACE_SM)),
    gap: Size::new(Val::Px(theme::SPACE_XS), Val::Px(theme::SPACE_XS)),
    ..Style::DEFAULT
};

/// One past round: swatch, label, outcome.
pub fn history_card_style(width: f32) -> Style {
    Style {
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        size: Size::new(Val::Px(width), Val::Auto),
        min_size: Size::new(Val::Px(width), Val::Px(theme::TOUCH_TARGET)),
        padding: UiRect::all(Val::Px(theme::SPACE_SM)),
        margin: UiRect::all(Val::Px(theme::SPACE_XS)),
        ..Style::DEFAULT
    }
}

/// Width left for the round's label once the swatch and the outcome mark have
/// taken their share of the card.
pub fn history_card_label_width(width: f32) -> f32 {
    (width - theme::SPACE_SM * 2.0 - SWATCH_SIZE - 40.0).max(60.0)
}

pub const SWATCH_SIZE: f32 = 32.0;

/// The color the round was actually asking for.
pub const SWATCH_STYLE: Style = Style {
    // `min_size` too: a flex row with a long label would otherwise shrink the
    // swatch to nothing, and the swatch is the whole point of the row.
    size: Size::new(Val::Px(SWATCH_SIZE), Val::Px(SWATCH_SIZE)),
    min_size: Size::new(Val::Px(SWATCH_SIZE), Val::Px(SWATCH_SIZE)),
    ..Style::DEFAULT
};

pub fn button_style(width: f32) -> Style {
    theme::button_style(width)
}

pub fn pagination_container_style(width: f32) -> Style {
    Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        size: Size::new(Val::Px(width), Val::Px(theme::TOUCH_TARGET)),
        gap: Size::new(Val::Px(theme::SPACE_SM), Val::Px(theme::SPACE_SM)),
        margin: UiRect::all(Val::Px(theme::SPACE_XS)),
        ..Style::DEFAULT
    }
}

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
