//! Shared design tokens.
//!
//! Every feature `styles.rs` pulls colors, type and spacing from here so the
//! screens read as one product. Layout `Style` consts stay with their feature.
//!
//! Note on text: `digital7mono.ttf` is a seven-segment display font with a
//! narrow glyph set. Keep user-facing strings ASCII and unaccented — no "ç",
//! no "ó", no check marks — or they render as blanks.

use bevy::prelude::*;

// --- Palette ---------------------------------------------------------------
//
// Neon on near-black, from the mock-up: a very dark violet ground, panels one
// step above it, and saturated accents used sparingly enough that they still
// read as neon rather than as noise.

/// The ground the whole game sits on. Also the board background during a round
/// — see `ColorPuzzle::background_color`, which tints it slightly toward the
/// round's own hue.
pub const BACKGROUND: Color = Color::rgb(0.055, 0.047, 0.086);

/// Scrim behind full-screen menus. Nearly opaque: the board behind it is a
/// puzzle in progress, and a legible menu matters more than a glimpse of it.
pub const SCRIM: Color = Color::rgba(0.043, 0.035, 0.071, 0.94);

/// Panels sitting on the scrim.
pub const SURFACE: Color = Color::rgb(0.098, 0.090, 0.145);
/// One step above `SURFACE`: rows, tiles and secondary buttons on a panel.
pub const SURFACE_RAISED: Color = Color::rgb(0.129, 0.118, 0.188);
/// A square with its color hidden, in `Memory`. Light enough to read as a
/// face-down card against the board background — the player still has to be
/// able to see what they are aiming at.
pub const SURFACE_HIDDEN: Color = Color::rgb(0.243, 0.231, 0.318);

/// Hairline that gives a panel its edge. Bevy 0.10 UI has no border color, so
/// this is used as the background of a wrapper node with a couple of pixels of
/// padding — the inner node paints over all but the edge.
pub const OUTLINE: Color = Color::rgb(0.184, 0.169, 0.271);

pub const ON_SURFACE: Color = Color::rgb(0.93, 0.93, 0.96);
/// Secondary text: present but demoted.
pub const MUTED: Color = Color::rgb(0.54, 0.53, 0.63);

/// The brand color, and the color of the one action a screen wants taken.
pub const PRIMARY: Color = Color::rgb(0.659, 0.333, 0.969);
/// Reward green. Reserved for gains — correct picks, records, streaks.
pub const SUCCESS: Color = Color::rgb(0.290, 0.871, 0.502);
/// Loss red. Reserved for misses, for time running out, and for the one
/// button that ends a run.
pub const DANGER: Color = Color::rgb(0.882, 0.114, 0.282);
/// Celebration gold. Reserved for the rarest moments (records, level ups) so
/// it keeps its meaning.
pub const ACCENT: Color = Color::rgb(0.961, 0.647, 0.141);
pub const LIME: Color = Color::rgb(0.639, 0.776, 0.078);
pub const INFO: Color = Color::rgb(0.231, 0.510, 0.965);
pub const PINK: Color = Color::rgb(0.925, 0.282, 0.600);

// --- Buttons ---------------------------------------------------------------

pub const BUTTON: Color = SURFACE_RAISED;
pub const BUTTON_HOVERED: Color = Color::rgb(0.176, 0.161, 0.259);
pub const BUTTON_PRESSED: Color = Color::rgb(0.224, 0.204, 0.325);

pub const BUTTON_PRIMARY: Color = PRIMARY;
pub const BUTTON_PRIMARY_HOVERED: Color = Color::rgb(0.729, 0.443, 0.980);
pub const BUTTON_PRIMARY_PRESSED: Color = Color::rgb(0.796, 0.553, 0.988);

/// Destructive action: ending the run.
pub const BUTTON_DANGER: Color = DANGER;
pub const BUTTON_DANGER_HOVERED: Color = Color::rgb(0.925, 0.220, 0.373);
pub const BUTTON_DANGER_PRESSED: Color = Color::rgb(0.949, 0.353, 0.478);

// --- Spacing ---------------------------------------------------------------

pub const SPACE_XS: f32 = 4.0;
pub const SPACE_SM: f32 = 8.0;
pub const SPACE_MD: f32 = 16.0;
pub const SPACE_LG: f32 = 24.0;

/// Minimum comfortable tap size on a phone. The game ships to mobile browsers,
/// so no interactive element should be smaller than this in either axis.
pub const TOUCH_TARGET: f32 = 48.0;

// --- Type scale ------------------------------------------------------------
//
// The old screens mixed 10/12/18/20/32px with no relationship between them and
// were unreadable on a phone. These are the only sizes that should appear.

pub const TEXT_XS: f32 = 14.0;
pub const TEXT_SM: f32 = 18.0;
pub const TEXT_MD: f32 = 24.0;
pub const TEXT_LG: f32 = 32.0;
pub const TEXT_XL: f32 = 44.0;
pub const TEXT_DISPLAY: f32 = 64.0;

// --- Layout ----------------------------------------------------------------

/// Widest a menu column is ever allowed to get. Past this, extra width only
/// spreads a short label across a line the eye has to travel.
pub const CONTENT_MAX_WIDTH: f32 = 420.0;

/// Width of a full-width card or button for a given window width.
///
/// Screens size their children in pixels rather than percentages so that
/// [`wrapped_text`] has a real number to fit a label to; a percentage button
/// holding a label of unknown width overflows silently.
pub fn content_width(window_width: f32) -> f32 {
    (window_width - SPACE_MD * 2.0)
        .min(CONTENT_MAX_WIDTH)
        .max(160.0)
}

/// How wide one glyph of `digital7mono.ttf` is, as a fraction of the font size.
/// The font is monospaced, so one number describes it. Measured off rendered
/// output at about 0.96 and rounded up: overestimating costs a little type
/// size, underestimating puts text back outside its button.
pub const GLYPH_ADVANCE_RATIO: f32 = 1.0;

/// Width of `value` rendered at `size`.
pub fn text_width(value: &str, size: f32) -> f32 {
    value.chars().count() as f32 * size * GLYPH_ADVANCE_RATIO
}

/// Smallest type we will shrink a label to before letting it wrap.
///
/// Wrapping is the bad outcome (the lines overlap), so the floor is set low
/// enough that no label in the game reaches it. If a new string does, shorten
/// the string rather than lowering this further.
const MIN_FIT_SIZE: f32 = 9.0;

/// A centered text node sized to fit `max_width` on a single line.
///
/// Long labels used to run straight out of their buttons, and simply letting
/// Bevy wrap them is not an option here: `digital7mono.ttf` declares vertical
/// metrics far tighter than its glyphs actually draw, so wrapped lines — and
/// neighbouring nodes — overlap each other into an unreadable smear. Keeping
/// each label on one line and shrinking the type to make it fit is what the
/// font actually supports.
///
/// The node also carries vertical margin for the same reason: without it, the
/// ink of one line touches the line above it.
pub fn wrapped_text(value: impl Into<String>, style: TextStyle, max_width: f32) -> TextBundle {
    let value: String = value.into();
    let max_width = max_width.max(1.0);

    let mut style = style;
    let width = text_width(&value, style.font_size);
    if width > max_width {
        style.font_size = (style.font_size * max_width / width).max(MIN_FIT_SIZE);
    }

    let breathing_room = (style.font_size * 0.3).ceil();

    TextBundle {
        text: Text::from_section(value, style).with_alignment(TextAlignment::Center),
        style: Style {
            // Only reached by labels already shrunk to `MIN_FIT_SIZE`; past
            // that, wrapping is still better than overflowing.
            max_size: Size::new(Val::Px(max_width), Val::Auto),
            margin: UiRect::vertical(Val::Px(breathing_room)),
            ..Style::DEFAULT
        },
        ..default()
    }
}

/// Multi-colored text on one line, fitted the same way [`wrapped_text`] is.
///
/// Used for the wordmark, where each letter carries its own color. Fitting is
/// done on the whole string so the sections keep a common size.
pub fn wrapped_sections(
    parts: Vec<(String, Color)>,
    font: Handle<Font>,
    base_size: f32,
    max_width: f32,
) -> TextBundle {
    let max_width = max_width.max(1.0);
    let total: String = parts.iter().map(|(value, _)| value.as_str()).collect();

    let mut size = base_size;
    let width = text_width(&total, size);
    if width > max_width {
        size = (size * max_width / width).max(MIN_FIT_SIZE);
    }

    let sections: Vec<TextSection> = parts
        .into_iter()
        .map(|(value, color)| TextSection {
            value,
            style: TextStyle {
                font: font.clone(),
                font_size: size,
                color,
            },
        })
        .collect();

    TextBundle {
        text: Text::from_sections(sections).with_alignment(TextAlignment::Center),
        style: Style {
            max_size: Size::new(Val::Px(max_width), Val::Auto),
            margin: UiRect::vertical(Val::Px((size * 0.3).ceil())),
            ..Style::DEFAULT
        },
        ..default()
    }
}

/// Thickness of the faked borders described on [`OUTLINE`].
pub const HAIRLINE: f32 = 2.0;

/// Wrapper that draws a `HAIRLINE` border around whatever it contains.
///
/// Bevy 0.10's UI cannot stroke a node, so the border is a parent painted in
/// the border color with just enough padding to show around the child.
pub fn outlined_style(width: f32) -> Style {
    Style {
        // Taffy sizes the content box, so the border's own padding is added
        // outside `width`. Take it off here, or every outlined card ends up
        // `HAIRLINE * 2` wider than the column it is supposed to sit in.
        size: Size::new(Val::Px((width - HAIRLINE * 2.0).max(1.0)), Val::Auto),
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(Val::Px(HAIRLINE)),
        margin: UiRect::vertical(Val::Px(SPACE_XS)),
        ..Style::DEFAULT
    }
}

/// The panel that sits inside an [`outlined_style`] wrapper.
pub fn outlined_inner_style() -> Style {
    Style {
        size: Size::new(Val::Percent(100.0), Val::Auto),
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::Px(SPACE_SM)),
        ..Style::DEFAULT
    }
}

/// A solid color chip: the mode marker on a menu card, the target color on a
/// history row.
pub fn tile_style(size: f32) -> Style {
    Style {
        size: Size::new(Val::Px(size), Val::Px(size)),
        // Flex would otherwise shrink a chip to nothing to make room for a long
        // label, and the chip is the point of the row.
        min_size: Size::new(Val::Px(size), Val::Px(size)),
        ..Style::DEFAULT
    }
}

/// A full-width button that grows to fit its label.
///
/// Height is `Auto` over a `TOUCH_TARGET` floor, so a label that needs more
/// room makes the button taller rather than being clipped by it.
pub fn button_style(width: f32) -> Style {
    Style {
        size: Size::new(Val::Px(width), Val::Auto),
        min_size: Size::new(Val::Px(width), Val::Px(TOUCH_TARGET)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::Px(SPACE_SM)),
        margin: UiRect::all(Val::Px(SPACE_XS)),
        ..Style::DEFAULT
    }
}

/// Text width available inside a [`button_style`] of `width`.
pub fn button_text_width(width: f32) -> f32 {
    width - SPACE_SM * 2.0
}

pub fn font(asset_server: &Res<AssetServer>) -> Handle<Font> {
    asset_server.load("digital7mono.ttf")
}

pub fn text(asset_server: &Res<AssetServer>, size: f32, color: Color) -> TextStyle {
    TextStyle {
        font: font(asset_server),
        font_size: size,
        color,
    }
}

/// Small caps-style label above a value ("PONTOS", "RECORDE").
pub fn text_label(asset_server: &Res<AssetServer>) -> TextStyle {
    text(asset_server, TEXT_XS, MUTED)
}

pub fn text_body(asset_server: &Res<AssetServer>) -> TextStyle {
    text(asset_server, TEXT_SM, ON_SURFACE)
}

pub fn text_button(asset_server: &Res<AssetServer>) -> TextStyle {
    text(asset_server, TEXT_SM, ON_SURFACE)
}

pub fn text_title(asset_server: &Res<AssetServer>) -> TextStyle {
    text(asset_server, TEXT_LG, ON_SURFACE)
}

pub fn text_display(asset_server: &Res<AssetServer>, color: Color) -> TextStyle {
    text(asset_server, TEXT_DISPLAY, color)
}
