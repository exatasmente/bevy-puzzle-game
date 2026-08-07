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
