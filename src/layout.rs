//! Keeps screens laid out for the window they are actually in.
//!
//! Menus size their cards and buttons in pixels taken from the window width,
//! because that is the only way to fit a label to a button with this font (see
//! `theme::wrapped_text`). Those pixels are read once, when the screen is
//! spawned — so a window that changes size afterwards leaves every card the
//! wrong width.
//!
//! That is not a corner case. The web build sets `fit_canvas_to_parent`, so the
//! canvas starts at Bevy's default 1280x720 and is resized to the real viewport
//! a frame or two later — after the main menu has been built. On a phone the
//! menu was being laid out 420px wide inside a 390px window, which is why its
//! cards ran off both edges.

use bevy::prelude::*;

/// Sent when the window has changed size enough to be worth rebuilding for.
pub struct RelayoutEvent;

/// The last window width a screen was built against.
#[derive(Resource)]
pub struct LayoutWidth(f32);

impl Default for LayoutWidth {
    fn default() -> Self {
        // Deliberately not a plausible window width, so the first real
        // measurement always counts as a change.
        Self(-1.0)
    }
}

/// A resize smaller than this is ignored: rebuilding a screen mid-drag on every
/// pixel would be wasteful, and a couple of pixels change no layout decision.
const SIGNIFICANT_CHANGE: f32 = 4.0;

pub fn track_window_width(
    windows: Query<&Window>,
    mut layout_width: ResMut<LayoutWidth>,
    mut relayout_event_writer: EventWriter<RelayoutEvent>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };

    if (window.width() - layout_width.0).abs() < SIGNIFICANT_CHANGE {
        return;
    }

    layout_width.0 = window.width();
    relayout_event_writer.send(RelayoutEvent);
}
