//! Oklab, so "how different are these two colors" means the same thing in
//! every hue.
//!
//! The puzzle used to vary raw sRGB channels by a fixed amount. That amount is
//! not a fixed *difficulty*: 0.05 on a channel is glaring in a mid grey and
//! nearly invisible in a dark saturated blue, so two rounds at the same level
//! could be trivial or impossible depending on which base color the dice
//! produced. Oklab is perceptually uniform enough that one number — the
//! distance between two colors — behaves like a difficulty dial.
//!
//! Reference: Björn Ottosson, "A perceptual color space for image processing".

use bevy::prelude::Color;

/// A color as lightness plus two opponent-color axes.
///
/// `l` runs 0..1. `a` and `b` are unbounded in principle but stay within about
/// ±0.4 for anything sRGB can display.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oklab {
    pub l: f32,
    pub a: f32,
    pub b: f32,
}

impl Oklab {
    pub fn new(l: f32, a: f32, b: f32) -> Self {
        Self { l, a, b }
    }

    /// Chroma and hue as a polar pair, for building colors by hue.
    pub fn from_lch(l: f32, chroma: f32, hue_radians: f32) -> Self {
        Self {
            l,
            a: chroma * hue_radians.cos(),
            b: chroma * hue_radians.sin(),
        }
    }

    /// Moves by `amount` along a unit direction.
    pub fn offset(&self, direction: (f32, f32, f32), amount: f32) -> Self {
        let (dl, da, db) = direction;
        Self {
            l: self.l + dl * amount,
            a: self.a + da * amount,
            b: self.b + db * amount,
        }
    }
}

fn srgb_to_linear(channel: f32) -> f32 {
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(channel: f32) -> f32 {
    if channel <= 0.0031308 {
        channel * 12.92
    } else {
        1.055 * channel.powf(1.0 / 2.4) - 0.055
    }
}

pub fn from_color(color: Color) -> Oklab {
    let r = srgb_to_linear(color.to_srgba().red);
    let g = srgb_to_linear(color.to_srgba().green);
    let b = srgb_to_linear(color.to_srgba().blue);

    let long = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
    let medium = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
    let short = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;

    let long = long.cbrt();
    let medium = medium.cbrt();
    let short = short.cbrt();

    Oklab {
        l: 0.2104542553 * long + 0.7936177850 * medium - 0.0040720468 * short,
        a: 1.9779984951 * long - 2.4285922050 * medium + 0.4505937099 * short,
        b: 0.0259040371 * long + 0.7827717662 * medium - 0.8086757660 * short,
    }
}

/// Converts back to sRGB, or `None` when the color is outside what the display
/// can show.
///
/// Callers want the `None` rather than a clamp: clamping silently drags the
/// color back toward the gamut boundary, which changes the very distance the
/// difficulty is set by. A round built on a clamped color would be easier than
/// its level claims.
pub fn to_color(lab: Oklab) -> Option<Color> {
    let long = lab.l + 0.3963377774 * lab.a + 0.2158037573 * lab.b;
    let medium = lab.l - 0.1055613458 * lab.a - 0.0638541728 * lab.b;
    let short = lab.l - 0.0894841775 * lab.a - 1.2914855480 * lab.b;

    let long = long * long * long;
    let medium = medium * medium * medium;
    let short = short * short * short;

    let r = 4.0767416621 * long - 3.3077115913 * medium + 0.2309699292 * short;
    let g = -1.2684380046 * long + 2.6097574011 * medium - 0.3413193965 * short;
    let b = -0.0041960863 * long - 0.7034186147 * medium + 1.7076147010 * short;

    // A hair of tolerance: values a rounding error outside the cube are the
    // gamut boundary itself, not a color we should throw away.
    const TOLERANCE: f32 = 0.0005;
    for channel in [r, g, b] {
        if channel < -TOLERANCE || channel > 1.0 + TOLERANCE {
            return None;
        }
    }

    Some(Color::srgb(
        linear_to_srgb(r.clamp(0.0, 1.0)),
        linear_to_srgb(g.clamp(0.0, 1.0)),
        linear_to_srgb(b.clamp(0.0, 1.0)),
    ))
}

/// Mixes two colors, `amount` of `b` into `a`, perceptually.
pub fn mix(a: Color, b: Color, amount: f32) -> Color {
    let a_lab = from_color(a);
    let b_lab = from_color(b);
    let amount = amount.clamp(0.0, 1.0);

    let mixed = Oklab::new(
        a_lab.l + (b_lab.l - a_lab.l) * amount,
        a_lab.a + (b_lab.a - a_lab.a) * amount,
        a_lab.b + (b_lab.b - a_lab.b) * amount,
    );

    to_color(mixed).unwrap_or(a)
}
