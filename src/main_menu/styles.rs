use bevy::prelude::*;

use crate::theme;

/// Side of the colored chip that identifies a mode.
pub const MODE_CHIP_SIZE: f32 = 44.0;

pub fn main_menu_style() -> Node {
    Node {
    flex_direction: FlexDirection::Column,
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
    // No horizontal padding: Taffy sizes the content box, so a node at 100%
    // plus padding is wider than its parent and everything inside it slides
    // off the screen edge. The children carry their own width instead.
    ..Node::DEFAULT
}
}

/// The wordmark block.
///
/// Both this and the cards below are sized from the window height, because the
/// menu has no scrolling and the mode list has grown to five: at the fixed
/// sizes they used to have, the last card fell off the bottom of a 480px
/// screen, which is the shortest window the app allows.
pub fn title_style(window_height: f32) -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        size: Size::new(Val::Percent(100.0), Val::Px((window_height * 0.14).clamp(70.0, 120.0))),
        margin: UiRect {
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            top: Val::Px(0.0),
            bottom: Val::Px(theme::SPACE_SM),
        },
        ..Node::DEFAULT
    }
}

/// Height a mode card gets, given the window and how many modes must fit.
pub fn mode_card_height(window_height: f32, modes: usize) -> f32 {
    let title = (window_height * 0.14).clamp(70.0, 120.0);
    let per_card = (window_height - title - theme::SPACE_LG) / modes.max(1) as f32;

    // Below the touch target the card stops being a comfortable tap; above 84
    // it just wastes space the list needs.
    (per_card - theme::SPACE_SM * 2.0).clamp(theme::TOUCH_TARGET, 84.0)
}

/// The padded row inside a mode card: chip, then the text column.
pub fn mode_card_inner_style(height: f32) -> Node {
    Node {
        min_size: Size::new(Val::Percent(100.0), Val::Px(height)),
        ..theme::outlined_inner_style()
    }
}

/// The text column beside the chip.
pub fn mode_card_text_style(width: f32) -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        width: Val::Px(width),
        height: Val::Auto,
        margin: UiRect::left(Val::Px(theme::SPACE_SM)),
        ..Node::DEFAULT
    }
}

/// Width left for a mode card's text once the border, the padding and the chip
/// have taken their share.
pub fn mode_card_text_width(width: f32) -> f32 {
    (width - theme::HAIRLINE * 2.0 - theme::SPACE_SM * 3.0 - MODE_CHIP_SIZE).max(80.0)
}

/// Side of the colored chip, shrunk if the card had to get short.
pub fn mode_chip_size(card_height: f32) -> f32 {
    MODE_CHIP_SIZE.min(card_height - theme::SPACE_SM * 2.0).max(20.0)
}

/// A mode card's border, in the mode's own color.
///
/// Dim at rest so four cards do not compete with each other; the interaction
/// system brightens it on hover and press.
pub fn card_border(accent: Color) -> Color {
    Color::srgba(accent.to_srgba().red, accent.to_srgba().green, accent.to_srgba().blue, 0.45)
}

pub fn card_border_hovered(accent: Color) -> Color {
    Color::srgba(accent.to_srgba().red, accent.to_srgba().green, accent.to_srgba().blue, 0.75)
}

pub fn card_border_pressed(accent: Color) -> Color {
    accent
}

pub fn get_mode_name_text_style(asset_server: &Res<AssetServer>) -> theme::TextStyle {
    theme::text(asset_server, theme::TEXT_MD, theme::ON_SURFACE)
}

pub fn get_mode_description_text_style(asset_server: &Res<AssetServer>) -> theme::TextStyle {
    theme::text(asset_server, theme::TEXT_XS, theme::MUTED)
}

pub fn get_best_score_text_style(asset_server: &Res<AssetServer>) -> theme::TextStyle {
    theme::text(asset_server, theme::TEXT_XS, theme::ACCENT)
}
