use bevy::prelude::*;

use crate::theme;

pub use crate::theme::{
    BUTTON, BUTTON_HOVERED, BUTTON_PRESSED, BUTTON_PRIMARY, BUTTON_PRIMARY_HOVERED,
    BUTTON_PRIMARY_PRESSED, SCRIM, SURFACE,
};

pub fn game_over_menu_style() -> Node {
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

pub fn game_over_menu_container_style() -> Node {
    Node {
    display: Display::Flex,
    flex_direction: FlexDirection::Column,
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    width: Val::Percent(100.0),
    height: Val::Percent(100.0),
    // Vertical only, and SPACE_MD rather than SPACE_LG: the summary is a tall
    // stack — score, record, stats, three buttons — and the larger padding was
    // enough to push the last button off a short screen.
    padding: UiRect {
        left: Val::Px(0.0),
        right: Val::Px(0.0),
        top: Val::Px(theme::SPACE_MD),
        bottom: Val::Px(theme::SPACE_MD),
    },
    column_gap: Val::Px(theme::SPACE_SM),
    row_gap: Val::Px(theme::SPACE_SM),
    ..Node::DEFAULT
}
}

/// Row holding one stat label and its value, pushed to opposite edges so the
/// numbers line up in a column the eye can scan.
pub fn stat_row_style(width: f32) -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        width: Val::Px(width),
        height: Val::Auto,
        min_width: Val::Px(width),
        min_height: Val::Px(30.0),
        ..Node::DEFAULT
    }
}

pub fn button_style(width: f32) -> Node {
    theme::button_style(width)
}

/// Primary action: taller than the rest so the thumb finds it first.
pub fn primary_button_style(width: f32) -> Node {
    Node {
        min_width: Val::Px(width),
        min_height: Val::Px(60.0),
        margin: UiRect::all(Val::Px(theme::SPACE_SM)),
        ..theme::button_style(width)
    }
}

pub fn get_title_text_style(asset_server: &Res<AssetServer>) -> theme::TextStyle {
    theme::text_title(asset_server)
}

pub fn get_resume_text_style(asset_server: &Res<AssetServer>) -> theme::TextStyle {
    theme::text_body(asset_server)
}

pub fn get_label_text_style(asset_server: &Res<AssetServer>) -> theme::TextStyle {
    theme::text_label(asset_server)
}

pub fn get_button_text_style(asset_server: &Res<AssetServer>) -> theme::TextStyle {
    theme::text_button(asset_server)
}
