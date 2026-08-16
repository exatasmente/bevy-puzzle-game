use bevy::prelude::*;

use crate::theme;

pub use crate::theme::{BUTTON, BUTTON_HOVERED, BUTTON_PRESSED};

/// Full-screen, non-interactive layer holding the HUD. It carries no
/// `Interaction`, so it never intercepts a tap meant for the board.
pub fn hud_root_style() -> Node {
    Node {
    position_type: PositionType::Absolute,
    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
    flex_direction: FlexDirection::Column,
    justify_content: JustifyContent::FlexStart,
    align_items: AlignItems::Center,
    ..Node::DEFAULT
}
}

/// The dark panel the stats sit on.
///
/// The HUD used to float directly over the board, which meant its legibility
/// depended on whatever color the round had produced. A panel of its own fixes
/// the contrast, and it is what the mock-up shows.
pub fn hud_panel_style() -> Node {
    Node {
    width: Val::Percent(100.0),
        height: Val::Auto,
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
    ..Node::DEFAULT
}
}

/// How much of the panel width its rows use, leaving an even gutter.
pub const HUD_ROW_WIDTH: f32 = 94.0;

/// Translucent so the round's color still washes through the panel.
pub const HUD_PANEL_COLOR: Color = Color::srgba(0.055, 0.047, 0.086, 0.82);

pub fn top_bar_style() -> Node {
    Node {
    size: Size::new(Val::Percent(HUD_ROW_WIDTH), Val::Px(56.0)),
    flex_direction: FlexDirection::Row,
    justify_content: JustifyContent::SpaceBetween,
    align_items: AlignItems::Center,
    ..Node::DEFAULT
}
}

/// A label stacked over a value ("PONTOS" / "12").
///
/// The three stats share whatever the pause button leaves, evenly: fixed widths
/// meant the widest value ("00:00") decided whether the row fit.
pub fn stat_style() -> Node {
    Node {
    flex_direction: FlexDirection::Column,
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    flex_grow: 1.0,
    flex_basis: Val::Px(0.0),
    min_width: Val::Px(56.0),
        min_height: Val::Auto,
    ..Node::DEFAULT
}
}

/// Hairline between two stats, as in the mock-up.
pub fn stat_divider_style() -> Node {
    Node {
    size: Size::new(Val::Px(1.0), Val::Px(28.0)),
    min_size: Size::new(Val::Px(1.0), Val::Px(28.0)),
    ..Node::DEFAULT
}
}

pub fn progress_track_style() -> Node {
    Node {
    size: Size::new(Val::Percent(HUD_ROW_WIDTH), Val::Px(6.0)),
    ..Node::DEFAULT
}
}

pub fn progress_fill_style() -> Node {
    Node {
    size: Size::new(Val::Percent(0.0), Val::Percent(100.0)),
    ..Node::DEFAULT
}
}

/// The level on the left, the lives on the right.
///
/// `SpaceBetween` rather than `FlexStart` because the row now has two ends. It
/// reads the same as before in a timed mode, where the lives are not spawned
/// and a single child stays where it was.
pub fn level_row_style() -> Node {
    Node {
    width: Val::Percent(HUD_ROW_WIDTH),
        height: Val::Auto,
    flex_direction: FlexDirection::Row,
    justify_content: JustifyContent::SpaceBetween,
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
    ..Node::DEFAULT
}
}

/// The lives, at the right end of the level row.
///
/// Here and not in the top bar: at 320px that row already holds three stats and
/// a 48px button, which is why "SEQ" is abbreviated. The level row has the
/// space, and the markers are not a number that needs a label.
pub fn lives_row_style() -> Node {
    Node {
    flex_direction: FlexDirection::Row,
    align_items: AlignItems::Center,
    ..Node::DEFAULT
}
}

/// One life marker: a small square, because this font has no heart glyph and
/// Bevy 0.10's UI cannot round a corner.
pub fn lives_pip_style() -> Node {
    Node {
    size: Size::new(Val::Px(LIVES_PIP_SIZE), Val::Px(LIVES_PIP_SIZE)),
    min_size: Size::new(Val::Px(LIVES_PIP_SIZE), Val::Px(LIVES_PIP_SIZE)),
    margin: UiRect {
        left: Val::Px(theme::SPACE_XS),
        right: Val::Px(0.0),
        top: Val::Px(0.0),
        bottom: Val::Px(0.0),
    },
    ..Node::DEFAULT
}
}

pub const LIVES_PIP_SIZE: f32 = 12.0;

/// A life already spent. It stays in the row rather than disappearing, so the
/// gap is legible as something lost instead of the row simply being shorter.
pub const LIVES_PIP_SPENT_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.14);

/// Square tap target for the pause control, sized for a thumb.
pub fn icon_button_style() -> Node {
    Node {
    size: Size::new(Val::Px(theme::TOUCH_TARGET), Val::Px(theme::TOUCH_TARGET)),
    min_size: Size::new(Val::Px(theme::TOUCH_TARGET), Val::Px(theme::TOUCH_TARGET)),
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    ..Node::DEFAULT
}
}

/// Bottom-anchored container for the "voltar" button on the review screen.
pub fn back_button_root_style() -> Node {
    Node {
    position_type: PositionType::Absolute,
    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
    flex_direction: FlexDirection::Column,
    justify_content: JustifyContent::FlexEnd,
    align_items: AlignItems::Center,
    padding: UiRect::all(Val::Px(theme::SPACE_LG)),
    ..Node::DEFAULT
}
}

pub const BACK_BUTTON_WIDTH: f32 = 200.0;

/// Track color for the progress bar: visible, but clearly the empty part.
pub const PROGRESS_TRACK_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.14);
