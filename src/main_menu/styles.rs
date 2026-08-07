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

/// One mode per card. Tall enough to hold a name, a description and the best
/// score without crowding, and to be an easy target on a phone.
pub const MODE_CARD_STYLE: Style = Style {
    flex_direction: FlexDirection::Column,
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    size: Size::new(Val::Percent(88.0), Val::Px(92.0)),
    max_size: Size::new(Val::Px(420.0), Val::Auto),
    margin: UiRect::all(Val::Px(theme::SPACE_XS)),
    padding: UiRect::all(Val::Px(theme::SPACE_SM)),
    ..Style::DEFAULT
};

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
