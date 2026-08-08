use bevy::prelude::*;

use crate::theme;

/// Side of the colored chip that identifies a mode.
pub const MODE_CHIP_SIZE: f32 = 44.0;

pub const MAIN_MENU_STYLE: Style = Style {
    flex_direction: FlexDirection::Column,
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
    // No horizontal padding: Taffy sizes the content box, so a node at 100%
    // plus padding is wider than its parent and everything inside it slides
    // off the screen edge. The children carry their own width instead.
    ..Style::DEFAULT
};

pub const TITLE_STYLE: Style = Style {
    flex_direction: FlexDirection::Column,
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    size: Size::new(Val::Percent(100.0), Val::Px(120.0)),
    margin: UiRect {
        left: Val::Px(0.0),
        right: Val::Px(0.0),
        top: Val::Px(0.0),
        bottom: Val::Px(theme::SPACE_MD),
    },
    ..Style::DEFAULT
};

/// The padded row inside a mode card: chip, then the text column.
pub fn mode_card_inner_style() -> Style {
    Style {
        min_size: Size::new(Val::Percent(100.0), Val::Px(76.0)),
        ..theme::outlined_inner_style()
    }
}

/// The text column beside the chip.
pub fn mode_card_text_style(width: f32) -> Style {
    Style {
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        size: Size::new(Val::Px(width), Val::Auto),
        margin: UiRect::left(Val::Px(theme::SPACE_SM)),
        ..Style::DEFAULT
    }
}

/// Width left for a mode card's text once the border, the padding and the chip
/// have taken their share.
pub fn mode_card_text_width(width: f32) -> f32 {
    (width - theme::HAIRLINE * 2.0 - theme::SPACE_SM * 3.0 - MODE_CHIP_SIZE).max(80.0)
}

/// A mode card's border, in the mode's own color.
///
/// Dim at rest so four cards do not compete with each other; the interaction
/// system brightens it on hover and press.
pub fn card_border(accent: Color) -> Color {
    Color::rgba(accent.r(), accent.g(), accent.b(), 0.45)
}

pub fn card_border_hovered(accent: Color) -> Color {
    Color::rgba(accent.r(), accent.g(), accent.b(), 0.75)
}

pub fn card_border_pressed(accent: Color) -> Color {
    accent
}

pub fn get_mode_name_text_style(asset_server: &Res<AssetServer>) -> TextStyle {
    theme::text(asset_server, theme::TEXT_MD, theme::ON_SURFACE)
}

pub fn get_mode_description_text_style(asset_server: &Res<AssetServer>) -> TextStyle {
    theme::text(asset_server, theme::TEXT_XS, theme::MUTED)
}

pub fn get_best_score_text_style(asset_server: &Res<AssetServer>) -> TextStyle {
    theme::text(asset_server, theme::TEXT_XS, theme::ACCENT)
}
