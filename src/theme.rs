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

/// Scrim behind full-screen menus. Semi transparent so the puzzle color, which
/// is the game's whole identity, still reads through it.
pub const SCRIM: Color = Color::rgba(0.04, 0.04, 0.06, 0.72);
/// Panels sitting on top of the scrim.
pub const SURFACE: Color = Color::rgba(0.10, 0.10, 0.13, 0.92);
pub const ON_SURFACE: Color = Color::rgb(0.97, 0.97, 0.98);
/// Secondary text: present but demoted.
pub const MUTED: Color = Color::rgb(0.64, 0.65, 0.72);

/// Reward green. Reserved for gains — correct picks, records, streaks.
pub const SUCCESS: Color = Color::rgb(0.22, 0.80, 0.44);
/// Loss red. Reserved for misses and time running out.
pub const DANGER: Color = Color::rgb(0.94, 0.28, 0.32);
/// Celebration gold. Reserved for the rarest moments (records, level ups) so
/// it keeps its meaning.
pub const ACCENT: Color = Color::rgb(1.00, 0.78, 0.24);

// --- Buttons ---------------------------------------------------------------

pub const BUTTON: Color = Color::rgb(0.17, 0.17, 0.21);
pub const BUTTON_HOVERED: Color = Color::rgb(0.25, 0.25, 0.30);
pub const BUTTON_PRESSED: Color = Color::rgb(0.34, 0.34, 0.40);

/// The one action we want taken on a screen (currently "jogar novamente").
pub const BUTTON_PRIMARY: Color = Color::rgb(0.16, 0.55, 0.33);
pub const BUTTON_PRIMARY_HOVERED: Color = Color::rgb(0.20, 0.66, 0.40);
pub const BUTTON_PRIMARY_PRESSED: Color = Color::rgb(0.26, 0.78, 0.48);

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
const MIN_FIT_SIZE: f32 = 10.0;

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
