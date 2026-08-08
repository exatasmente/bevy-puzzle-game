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

/// The dark panel the stats sit on.
///
/// The HUD used to float directly over the board, which meant its legibility
/// depended on whatever color the round had produced. A panel of its own fixes
/// the contrast, and it is what the mock-up shows.
pub const HUD_PANEL_STYLE: Style = Style {
    size: Size::new(Val::Percent(100.0), Val::Auto),
    flex_direction: FlexDirection::Column,
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    // Vertical only — horizontal padding on a 100%-wide node overflows it.
    // The rows inside inset themselves with a percentage width instead.
    padding: UiRect {
        left: Val::Px(0.0),
        right: Val::Px(0.0),
        top: Val::Px(theme::SPACE_SM),
        bottom: Val::Px(theme::SPACE_SM),
    },
    ..Style::DEFAULT
};

/// How much of the panel width its rows use, leaving an even gutter.
pub const HUD_ROW_WIDTH: f32 = 94.0;

/// Translucent so the round's color still washes through the panel.
pub const HUD_PANEL_COLOR: Color = Color::rgba(0.055, 0.047, 0.086, 0.82);

pub const TOP_BAR_STYLE: Style = Style {
    size: Size::new(Val::Percent(HUD_ROW_WIDTH), Val::Px(56.0)),
    flex_direction: FlexDirection::Row,
    justify_content: JustifyContent::SpaceBetween,
    align_items: AlignItems::Center,
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

/// Hairline between two stats, as in the mock-up.
pub const STAT_DIVIDER_STYLE: Style = Style {
    size: Size::new(Val::Px(1.0), Val::Px(28.0)),
    min_size: Size::new(Val::Px(1.0), Val::Px(28.0)),
    ..Style::DEFAULT
};

pub const PROGRESS_TRACK_STYLE: Style = Style {
    size: Size::new(Val::Percent(HUD_ROW_WIDTH), Val::Px(6.0)),
    ..Style::DEFAULT
};

pub const PROGRESS_FILL_STYLE: Style = Style {
    size: Size::new(Val::Percent(0.0), Val::Percent(100.0)),
    ..Style::DEFAULT
};

pub const LEVEL_ROW_STYLE: Style = Style {
    size: Size::new(Val::Percent(HUD_ROW_WIDTH), Val::Auto),
    flex_direction: FlexDirection::Row,
    justify_content: JustifyContent::FlexStart,
    align_items: AlignItems::Center,
    // Clear of the stat values above and the progress bar below: this font's
    // glyphs draw well past the line box it reports, so nodes that merely abut
    // each other collide.
    margin: UiRect {
        left: Val::Px(0.0),
        right: Val::Px(0.0),
        top: Val::Px(theme::SPACE_SM),
        bottom: Val::Px(theme::SPACE_SM),
    },
    ..Style::DEFAULT
};

/// Square tap target for the pause control, sized for a thumb.
pub const ICON_BUTTON_STYLE: Style = Style {
    size: Size::new(Val::Px(theme::TOUCH_TARGET), Val::Px(theme::TOUCH_TARGET)),
    min_size: Size::new(Val::Px(theme::TOUCH_TARGET), Val::Px(theme::TOUCH_TARGET)),
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
pub const PROGRESS_TRACK_COLOR: Color = Color::rgba(1.0, 1.0, 1.0, 0.14);
