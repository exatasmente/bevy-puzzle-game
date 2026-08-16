use bevy::prelude::*;

use crate::theme;

pub use crate::theme::{BUTTON, BUTTON_HOVERED, BUTTON_PRESSED, SCRIM, SURFACE};

pub fn history_menu_style() -> Node {
    Node {
    position_type: PositionType::Absolute,
    display: Display::Flex,
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    width: Val::Percent(100.0),
    height: Val::Percent(100.0),
    ..Node::DEFAULT
}
}

pub fn history_menu_container_style() -> Node {
    Node {
    display: Display::Flex,
    flex_direction: FlexDirection::Column,
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    width: Val::Percent(100.0),
    height: Val::Percent(100.0),
    // Vertical only — see main_menu_style().
    padding: UiRect {
        left: Val::Px(0.0),
        right: Val::Px(0.0),
        top: Val::Px(theme::SPACE_SM),
        bottom: Val::Px(theme::SPACE_SM),
    },
    column_gap: Val::Px(theme::SPACE_XS),
    row_gap: Val::Px(theme::SPACE_XS),
    ..Node::DEFAULT
}
}

/// One past round: swatch, label, outcome.
pub fn history_card_style(width: f32) -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        width: Val::Px(width),
        height: Val::Auto,
        min_width: Val::Px(width),
        min_height: Val::Px(theme::TOUCH_TARGET),
        padding: UiRect::all(Val::Px(theme::SPACE_SM)),
        margin: UiRect::all(Val::Px(theme::SPACE_XS)),
        ..Node::DEFAULT
    }
}

/// Width left for the round's label once the swatch and the outcome mark have
/// taken their share of the card.
pub fn history_card_label_width(width: f32) -> f32 {
    (width - theme::SPACE_SM * 2.0 - SWATCH_SIZE - 40.0).max(60.0)
}

pub const SWATCH_SIZE: f32 = 32.0;

pub fn button_style(width: f32) -> Node {
    theme::button_style(width)
}

pub fn pagination_container_style(width: f32) -> Node {
    Node {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        width: Val::Px(width),
        height: Val::Px(theme::TOUCH_TARGET),
        column_gap: Val::Px(theme::SPACE_SM),
        row_gap: Val::Px(theme::SPACE_SM),
        margin: UiRect::all(Val::Px(theme::SPACE_XS)),
        ..Node::DEFAULT
    }
}

pub fn button_pagination_style() -> Node {
    Node {
    width: Val::Px(theme::TOUCH_TARGET),
    height: Val::Px(theme::TOUCH_TARGET),
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    ..Node::DEFAULT
}
}

pub fn get_title_text_style(asset_server: &Res<AssetServer>) -> theme::TextStyle {
    theme::text_title(asset_server)
}

pub fn get_button_text_style(asset_server: &Res<AssetServer>) -> theme::TextStyle {
    theme::text_button(asset_server)
}

pub fn get_label_text_style(asset_server: &Res<AssetServer>) -> theme::TextStyle {
    theme::text_label(asset_server)
}

pub fn get_pagination_button_text_style(asset_server: &Res<AssetServer>) -> theme::TextStyle {
    theme::text(asset_server, theme::TEXT_MD, theme::ON_SURFACE)
}
