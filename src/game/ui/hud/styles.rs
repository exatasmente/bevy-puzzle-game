use bevy::prelude::*;

use crate::theme;

pub use crate::theme::{BUTTON, BUTTON_HOVERED, BUTTON_PRESSED};

/// Full-screen, non-interactive layer holding the HUD. It carries no
/// `Interaction`, so it never intercepts a tap meant for the board.
pub const HUD_ROOT_STYLE: Style = Style {
    position_type: PositionType::Absolute,
    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
    flex_direction: FlexDirection::Column,
    justify_content: JustifyContent::FlexStart,
    align_items: AlignItems::Center,
    ..Style::DEFAULT
};

pub const TOP_BAR_STYLE: Style = Style {
    size: Size::new(Val::Percent(100.0), Val::Px(72.0)),
    flex_direction: FlexDirection::Row,
    justify_content: JustifyContent::SpaceBetween,
    align_items: AlignItems::Center,
    // Narrower than the rest of the app on purpose: at 320px this row carries
    // three stats and a 48px button, and SPACE_MD on both sides was enough to
    // push "00:00" out of its column.
    padding: UiRect::all(Val::Px(theme::SPACE_SM)),
    ..Style::DEFAULT
};

/// A label stacked over a value ("PONTOS" / "12").
///
/// The three stats share whatever the pause button leaves, evenly: fixed widths
/// meant the widest value ("00:00") decided whether the row fit.
pub const STAT_STYLE: Style = Style {
    flex_direction: FlexDirection::Column,
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    flex_grow: 1.0,
    flex_basis: Val::Px(0.0),
    min_size: Size::new(Val::Px(56.0), Val::Auto),
    ..Style::DEFAULT
};

pub const PROGRESS_TRACK_STYLE: Style = Style {
    size: Size::new(Val::Percent(90.0), Val::Px(6.0)),
    ..Style::DEFAULT
};

pub const PROGRESS_FILL_STYLE: Style = Style {
    size: Size::new(Val::Percent(0.0), Val::Percent(100.0)),
    ..Style::DEFAULT
};

pub const LEVEL_ROW_STYLE: Style = Style {
    size: Size::new(Val::Percent(90.0), Val::Auto),
    flex_direction: FlexDirection::Row,
    justify_content: JustifyContent::SpaceBetween,
    align_items: AlignItems::Center,
    // Clear of the stat values above it: this font's glyphs draw well past the
    // line box it reports, so nodes that merely abut each other collide.
    margin: UiRect::new(
        Val::Px(theme::SPACE_XS),
        Val::Px(theme::SPACE_XS),
        Val::Px(theme::SPACE_SM),
        Val::Px(theme::SPACE_XS),
    ),
    ..Style::DEFAULT
};

/// Square tap target for the pause control, sized for a thumb.
pub const ICON_BUTTON_STYLE: Style = Style {
    size: Size::new(Val::Px(theme::TOUCH_TARGET), Val::Px(theme::TOUCH_TARGET)),
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    ..Style::DEFAULT
};

/// Bottom-anchored container for the "voltar" button on the review screen.
pub const BACK_BUTTON_ROOT_STYLE: Style = Style {
    position_type: PositionType::Absolute,
    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
    flex_direction: FlexDirection::Column,
    justify_content: JustifyContent::FlexEnd,
    align_items: AlignItems::Center,
    padding: UiRect::all(Val::Px(theme::SPACE_LG)),
    ..Style::DEFAULT
};

pub const BACK_BUTTON_WIDTH: f32 = 200.0;

/// Track color for the progress bar: visible, but clearly the empty part.
pub const PROGRESS_TRACK_COLOR: Color = Color::rgba(1.0, 1.0, 1.0, 0.18);
