//! Layout for the goals screen. Colours and type come from `theme`.

use bevy::prelude::*;

use crate::theme;

pub fn menu_style() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::FlexStart,
        align_items: AlignItems::Center,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        padding: UiRect::vertical(Val::Px(theme::SPACE_MD)),
        row_gap: Val::Px(theme::SPACE_XS),
        ..Node::DEFAULT
    }
}

/// One goal: a colour chip, then title over requirement.
pub fn row_style(width: f32) -> Node {
    Node {
        width: Val::Px(width),
        min_height: Val::Px(theme::TOUCH_TARGET),
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: Val::Px(theme::SPACE_SM),
        padding: UiRect::all(Val::Px(theme::SPACE_XS)),
        ..Node::DEFAULT
    }
}

pub fn chip_style() -> Node {
    Node {
        width: Val::Px(CHIP),
        height: Val::Px(CHIP),
        min_width: Val::Px(CHIP),
        min_height: Val::Px(CHIP),
        ..Node::DEFAULT
    }
}

pub const CHIP: f32 = 20.0;

pub fn text_column_style() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::FlexStart,
        flex_grow: 1.0,
        ..Node::DEFAULT
    }
}

/// A goal not yet reached. Still listed, and still readable — the list is what
/// tells the player what the game rewards, so hiding the locked ones would hide
/// the only place that is written down.
pub const LOCKED_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.05);
