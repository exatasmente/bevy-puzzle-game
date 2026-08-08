//! Irregular board layout.
//!
//! The board is cut into pieces that tile the play area — they fit together —
//! and each piece is then pulled back from its cut lines by a random amount, so
//! the gaps between them are uneven.
//!
//! ## Why not a grid
//!
//! Because the answer is a piece painted the same color as the background, and
//! a grid tells the player exactly where every piece must be. On a grid the
//! hidden piece is a hole at a known address: the eye finds the one empty cell
//! pre-attentively, without comparing any colors, and the difficulty dial stops
//! mattering. With the pieces at unpredictable positions and sizes there is no
//! address to check, and the hidden piece has to be found the hard way — by
//! reading the negative space its neighbours leave.
//!
//! That is also why the pieces tile rather than float: a hole in a mass of
//! interlocking pieces is bounded by the edges of the pieces around it, so it
//! is findable. Shapes scattered at random over empty space would leave the
//! player nothing to infer from, and the round would be a lottery.
//!
//! Kept free of Bevy beyond `Vec2` so the invariants below can be tested.

use bevy::prelude::Vec2;
use rand::prelude::*;

/// One piece of the board: bottom-left corner and size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Slot {
    pub position: Vec2,
    pub size: Vec2,
}

impl Slot {
    fn area(&self) -> f32 {
        self.size.x * self.size.y
    }
}

/// Smallest a piece may get in either axis, before its gaps are cut. Anything
/// less is a sliver the player cannot aim at on a phone.
const MIN_SIDE: f32 = 46.0;

/// Range a piece is inset from its cut lines, per side. The spread is what
/// makes the spacing uneven; the floor keeps neighbours from touching, which
/// would merge them into one shape.
const MIN_INSET: f32 = 3.0;
const MAX_INSET: f32 = 13.0;

/// Cuts `count` interlocking pieces out of the rectangle between `min` and
/// `max`, then insets each one irregularly.
///
/// Fewer pieces than asked for come back when the area cannot be cut that
/// small without producing slivers — better a sparser board than one with
/// pieces too small to tap.
pub fn layout(min: Vec2, max: Vec2, count: usize, rng: &mut impl Rng) -> Vec<Slot> {
    let mut pieces = vec![Slot {
        position: min,
        size: max - min,
    }];

    while pieces.len() < count.max(1) {
        // Always split the biggest piece. Splitting a random one lets the
        // early cuts pile up in one corner and produces a board of one huge
        // piece and a shower of small ones.
        let Some(index) = largest_splittable(&pieces) else {
            break;
        };

        let piece = pieces.swap_remove(index);
        let (a, b) = split(piece, rng);
        pieces.push(a);
        pieces.push(b);
    }

    for piece in pieces.iter_mut() {
        *piece = inset(*piece, rng);
    }

    // Index order would otherwise follow the order pieces were cut, which is
    // spatially clustered. Nothing should be able to infer position from index.
    pieces.shuffle(rng);
    pieces
}

fn largest_splittable(pieces: &[Slot]) -> Option<usize> {
    pieces
        .iter()
        .enumerate()
        .filter(|(_, piece)| can_split(piece))
        .max_by(|(_, a), (_, b)| a.area().total_cmp(&b.area()))
        .map(|(index, _)| index)
}

fn can_split(piece: &Slot) -> bool {
    piece.size.x >= MIN_SIDE * 2.0 + MAX_INSET * 2.0 || piece.size.y >= MIN_SIDE * 2.0 + MAX_INSET * 2.0
}

fn split(piece: Slot, rng: &mut impl Rng) -> (Slot, Slot) {
    let can_split_x = piece.size.x >= MIN_SIDE * 2.0 + MAX_INSET * 2.0;
    let can_split_y = piece.size.y >= MIN_SIDE * 2.0 + MAX_INSET * 2.0;

    // Cutting the longer side keeps pieces from drifting toward long ribbons,
    // but not always: the occasional cut across the short side is what stops
    // every piece from looking like the same rectangle.
    let vertical = if can_split_x && can_split_y {
        if piece.size.x > piece.size.y {
            rng.gen_bool(0.75)
        } else {
            rng.gen_bool(0.25)
        }
    } else {
        can_split_x
    };

    let ratio = rng.gen_range(0.35..0.65);

    if vertical {
        let width = piece.size.x * ratio;
        (
            Slot {
                position: piece.position,
                size: Vec2::new(width, piece.size.y),
            },
            Slot {
                position: Vec2::new(piece.position.x + width, piece.position.y),
                size: Vec2::new(piece.size.x - width, piece.size.y),
            },
        )
    } else {
        let height = piece.size.y * ratio;
        (
            Slot {
                position: piece.position,
                size: Vec2::new(piece.size.x, height),
            },
            Slot {
                position: Vec2::new(piece.position.x, piece.position.y + height),
                size: Vec2::new(piece.size.x, piece.size.y - height),
            },
        )
    }
}

/// Pulls a piece back from its cut lines by a different amount on every side.
fn inset(piece: Slot, rng: &mut impl Rng) -> Slot {
    let mut side = |available: f32| {
        // Never eat so much that the piece drops under the minimum.
        let room = ((available - MIN_SIDE) / 2.0).clamp(0.0, MAX_INSET);
        if room <= MIN_INSET {
            room.max(0.0)
        } else {
            rng.gen_range(MIN_INSET..room)
        }
    };

    let left = side(piece.size.x);
    let right = side(piece.size.x);
    let bottom = side(piece.size.y);
    let top = side(piece.size.y);

    Slot {
        position: Vec2::new(piece.position.x + left, piece.position.y + bottom),
        size: Vec2::new(
            (piece.size.x - left - right).max(1.0),
            (piece.size.y - bottom - top).max(1.0),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rng() -> StdRng {
        StdRng::seed_from_u64(8_2026)
    }

    fn overlaps(a: &Slot, b: &Slot) -> bool {
        a.position.x < b.position.x + b.size.x
            && b.position.x < a.position.x + a.size.x
            && a.position.y < b.position.y + b.size.y
            && b.position.y < a.position.y + a.size.y
    }

    /// Two pieces sharing space would give the player a tap that belongs to
    /// both, and a hidden piece that is partly covered.
    #[test]
    fn pieces_never_overlap() {
        let mut rng = rng();

        for count in 4..=12 {
            let pieces = layout(Vec2::new(-180.0, -300.0), Vec2::new(180.0, 260.0), count, &mut rng);

            for (index, piece) in pieces.iter().enumerate() {
                for other in pieces.iter().skip(index + 1) {
                    assert!(!overlaps(piece, other), "{:?} overlaps {:?}", piece, other);
                }
            }
        }
    }

    #[test]
    fn pieces_stay_inside_the_area() {
        let mut rng = rng();
        let min = Vec2::new(-180.0, -300.0);
        let max = Vec2::new(180.0, 260.0);

        for piece in layout(min, max, 12, &mut rng) {
            assert!(piece.position.x >= min.x && piece.position.y >= min.y);
            assert!(piece.position.x + piece.size.x <= max.x);
            assert!(piece.position.y + piece.size.y <= max.y);
        }
    }

    #[test]
    fn pieces_stay_tappable() {
        let mut rng = rng();

        for piece in layout(Vec2::new(-180.0, -300.0), Vec2::new(180.0, 260.0), 12, &mut rng) {
            assert!(
                piece.size.x >= 20.0 && piece.size.y >= 20.0,
                "{:?} is too small to aim at",
                piece
            );
        }
    }

    /// The point of the layout: no two rounds put pieces in the same places,
    /// and the pieces within a round are not all the same size.
    #[test]
    fn the_layout_is_irregular() {
        let mut rng = rng();
        let min = Vec2::new(-180.0, -300.0);
        let max = Vec2::new(180.0, 260.0);

        let first = layout(min, max, 9, &mut rng);
        let second = layout(min, max, 9, &mut rng);
        assert_ne!(first, second, "two rounds produced the same board");

        let widths: Vec<f32> = first.iter().map(|piece| piece.size.x).collect();
        let spread = widths
            .iter()
            .cloned()
            .fold(f32::MIN, f32::max)
            - widths.iter().cloned().fold(f32::MAX, f32::min);

        assert!(spread > 20.0, "pieces are all but the same width: {:?}", widths);
    }
}
