use bevy::prelude::*;

use crate::theme;

pub use crate::theme::{BUTTON, BUTTON_HOVERED, BUTTON_PRESSED};

pub const MAIN_MENU_STYLE: Style = Style {
    flex_direction: FlexDirection::Column,
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
    padding: UiRect::all(Val::Px(theme::SPACE_MD)),
    gap: Size::new(Val::Px(theme::SPACE_SM), Val::Px(theme::SPACE_SM)),
    ..Style::DEFAULT
};

pub const TITLE_STYLE: Style = Style {
    flex_direction: FlexDirection::Column,
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    size: Size::new(Val::Percent(100.0), Val::Px(120.0)),
    ..Style::DEFAULT
};

/// One mode per card.
///
/// Sized in pixels from the window rather than as a percentage: the card holds
/// a description long enough to need fitting, and `theme::wrapped_text` fits a
/// label to a pixel width. Height is `Auto` over a floor, so the card can grow
/// with its contents instead of clipping them.
pub fn mode_card_style(width: f32) -> Style {
    Style {
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        size: Size::new(Val::Px(width), Val::Auto),
        min_size: Size::new(Val::Px(width), Val::Px(92.0)),
        margin: UiRect::all(Val::Px(theme::SPACE_XS)),
        padding: UiRect::all(Val::Px(theme::SPACE_SM)),
        ..Style::DEFAULT
    }
}

/// Width available to text inside a mode card.
pub fn mode_card_text_width(width: f32) -> f32 {
    width - theme::SPACE_SM * 2.0
}

pub fn get_title_text_style(asset_server: &Res<AssetServer>) -> TextStyle {
    theme::text(asset_server, theme::TEXT_XL, theme::ON_SURFACE)
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
