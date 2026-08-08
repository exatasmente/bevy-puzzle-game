//! Irregular board layout: an uneven honeycomb.
//!
//! Seeds are scattered on a jittered grid and the play area is divided into the
//! region closest to each one — a Voronoi tessellation. The cells come out as
//! convex polygons of five to seven sides that tile the area exactly, like a
//! honeycomb that was drawn by hand. Each cell is then pulled back from its own
//! edges, which opens the gap between neighbours.
//!
//! ## Why not a grid
//!
//! Because the answer is a piece painted the same color as the background, and
//! a grid tells the player exactly where every piece must be. On a grid the
//! hidden piece is a hole at a known address: the eye finds the one empty cell
//! pre-attentively, without comparing any colors, and the difficulty dial stops
//! mattering. With the pieces at unpredictable positions, sizes and shapes there
//! is no address to check, and the hidden piece has to be found the hard way —
//! by reading the negative space its neighbours leave.
//!
//! That is also why the pieces tessellate rather than float: a hole in a mass of
//! interlocking pieces is bounded by the edges of the pieces around it, so it is
//! findable. Shapes scattered over empty space would leave the player nothing to
//! infer from, and the round would be a lottery.
//!
//! Kept free of Bevy beyond `Vec2` so the invariants below can be tested.

use bevy::prelude::Vec2;
use rand::prelude::*;

/// One piece of the board: a convex polygon, given as its centre and the
/// corners around it.
///
/// Corners are relative to `centre` so the piece can be spawned at a transform
/// and drawn from the same points every time — including on replay.
#[derive(Debug, Clone, PartialEq)]
pub struct Piece {
    pub centre: Vec2,
    pub corners: Vec<Vec2>,
}

impl Piece {
    /// Whether a world-space point is inside the piece.
    ///
    /// The polygon is convex and wound counter-clockwise, so a point is inside
    /// when it is to the left of every edge.
    pub fn contains(&self, point: Vec2) -> bool {
        let local = point - self.centre;
        let count = self.corners.len();

        if count < 3 {
            return false;
        }

        (0..count).all(|index| {
            let a = self.corners[index];
            let b = self.corners[(index + 1) % count];
            let edge = b - a;
            edge.x * (local.y - a.y) - edge.y * (local.x - a.x) >= -f32::EPSILON
        })
    }

    /// Half-extents of the piece's bounding box, for anything that needs a size
    /// rather than a shape.
    pub fn half_extents(&self) -> Vec2 {
        let mut max = Vec2::ZERO;
        for corner in &self.corners {
            max = max.max(corner.abs());
        }
        max
    }
}

/// How far a cell is pulled back from its own edges, opening the gap to its
/// neighbours. Randomised per piece, so the gaps are as uneven as the cells.
const MIN_GAP: f32 = 3.0;
const MAX_GAP: f32 = 7.0;

/// A cell whose inradius falls below this is dropped rather than drawn: it
/// would be a sliver nobody can tap, and an invisible sliver would be an
/// unfindable answer.
const MIN_INRADIUS: f32 = 24.0;

/// How far a seed may wander from its slot on the scaffold grid, as a fraction
/// of the slot. Zero would give a regular honeycomb; too much lets seeds pass
/// each other and produce slivers.
const JITTER: f32 = 0.34;

/// Divides the rectangle between `min` and `max` into interlocking pieces.
///
/// Returns as many pieces as it can place without producing slivers, which may
/// be fewer than `count`. Callers must size the round to what they get back
/// rather than to what they asked for.
pub fn layout(min: Vec2, max: Vec2, count: usize, rng: &mut impl Rng) -> Vec<Piece> {
    let count = count.max(1);
    let (mut seeds, playable) = scatter(min, max, count, rng);
    let frame = vec![
        min,
        Vec2::new(max.x, min.y),
        max,
        Vec2::new(min.x, max.y),
    ];

    // One round of Lloyd relaxation:each seed moves to the middle of its own
    // cell. Jitter alone leaves the occasional seed crowded against a
    // neighbour, and the sliver that produces has to be dropped — which puts a
    // second background-colored hole on the board, indistinguishable from the
    // answer. Relaxing evens the cells out without making them regular, since
    // the seeds keep the offsets the jitter gave them.
    for _ in 0..2 {
        for index in 0..playable {
            if let Some(cell) = cell_of(&seeds, index, &frame) {
                seeds[index] = centroid(&cell);
            }
        }
    }

    let mut pieces = Vec::with_capacity(playable);

    for index in 0..playable {
        let Some(cell) = cell_of(&seeds, index, &frame) else {
            continue;
        };

        let gap = rng.gen_range(MIN_GAP..MAX_GAP);
        let cell = shrink(&cell, gap);

        if cell.len() < 3 {
            continue;
        }

        let centre = centroid(&cell);
        if inradius(&cell, centre) < MIN_INRADIUS {
            continue;
        }

        pieces.push(Piece {
            centre,
            corners: cell.iter().map(|corner| *corner - centre).collect(),
        });
    }

    // Index order would otherwise run along the scaffold, left to right and
    // bottom to top. Nothing should be able to infer position from index.
    pieces.shuffle(rng);
    pieces
}

/// The region closer to `seeds[index]` than to any other seed, clipped to the
/// frame. `None` when the seed is crowded out entirely.
fn cell_of(seeds: &[Vec2], index: usize, frame: &[Vec2]) -> Option<Vec<Vec2>> {
    let seed = seeds[index];
    let mut cell = frame.to_vec();

    for (other_index, other) in seeds.iter().enumerate() {
        if other_index == index {
            continue;
        }

        let normal = *other - seed;
        let midpoint = (*other + seed) / 2.0;
        cell = clip(&cell, normal, midpoint.dot(normal));

        if cell.len() < 3 {
            return None;
        }
    }

    Some(cell)
}

/// Seeds on a jittered honeycomb lattice.
///
/// A uniform random scatter clumps — some seeds land almost on top of each
/// other and produce cells too thin to tap. Starting from a lattice and letting
/// each seed wander inside its own slot keeps them apart while leaving the
/// result plainly irregular.
///
/// The lattice has every other row offset by half a step, which is what makes
/// the cells hexagonal. This matters more than it sounds: seeds on a square
/// grid meet four neighbours and their cells come out as quadrilaterals — a
/// grid that has been shaken, which is exactly the look this layout exists to
/// avoid. Offset rows give each seed six neighbours, and six neighbours is a
/// honeycomb.
/// Returns every seed, and how many of them at the front of the list become
/// pieces. The rest are ghosts.
fn scatter(min: Vec2, max: Vec2, count: usize, rng: &mut impl Rng) -> (Vec<Vec2>, usize) {
    let area = max - min;
    let aspect = area.x / area.y.max(1.0);

    let mut columns = ((count as f32 * aspect).sqrt().round() as usize).max(1);
    let mut rows = (count + columns - 1) / columns;
    if columns * rows < count {
        columns += 1;
        rows = (count + columns - 1) / columns;
    }

    let step = Vec2::new(area.x / columns as f32, area.y / rows as f32);

    let mut slots: Vec<(isize, isize)> = (0..rows as isize)
        .flat_map(|row| (0..columns as isize).map(move |column| (column, row)))
        .collect();

    // Dropping the surplus slots at random is deliberate: the cells left next
    // to a hole grow into it, which is where the bigger pieces come from.
    slots.shuffle(rng);
    slots.truncate(count);
    let playable = slots.len();

    // A ring of seeds outside the area. They never become pieces; they exist so
    // that the cells along the edge are bounded by a bisector against a
    // neighbour rather than by the frame. Without them almost every cell on a
    // board this small is a frame-clipped quadrilateral, which is the grid look
    // this layout exists to avoid.
    for row in -1..=rows as isize {
        for column in -1..=columns as isize {
            let outside = row < 0
                || column < 0
                || row >= rows as isize
                || column >= columns as isize;

            if outside {
                slots.push((column, row));
            }
        }
    }

    let seeds = slots
        .into_iter()
        .map(|(column, row)| {
            let stagger = if row.rem_euclid(2) == 1 { 0.5 } else { 0.0 };
            let slot = min
                + Vec2::new(column as f32 + 0.5 + stagger, row as f32 + 0.5) * step;

            // Less sideways wander than vertical: a seed that crosses into the
            // column next door undoes the interlock the stagger just created.
            slot + Vec2::new(
                rng.gen_range(-JITTER * 0.6..JITTER * 0.6) * step.x,
                rng.gen_range(-JITTER..JITTER) * step.y,
            )
        })
        .collect();

    (seeds, playable)
}

/// Sutherland–Hodgman: keeps the part of a convex polygon on the near side of
/// a line, defined as `point · normal <= offset`.
fn clip(polygon: &[Vec2], normal: Vec2, offset: f32) -> Vec<Vec2> {
    let mut result = Vec::with_capacity(polygon.len() + 1);

    for index in 0..polygon.len() {
        let current = polygon[index];
        let next = polygon[(index + 1) % polygon.len()];

        let current_distance = current.dot(normal) - offset;
        let next_distance = next.dot(normal) - offset;

        if current_distance <= 0.0 {
            result.push(current);
        }

        // The edge crosses the line, so the crossing point is a new corner.
        if (current_distance > 0.0) != (next_distance > 0.0) {
            let span = current_distance - next_distance;
            if span.abs() > f32::EPSILON {
                result.push(current + (next - current) * (current_distance / span));
            }
        }
    }

    result
}

/// Moves every edge of a convex polygon inward by `amount`.
fn shrink(polygon: &[Vec2], amount: f32) -> Vec<Vec2> {
    let mut result = polygon.to_vec();

    for index in 0..polygon.len() {
        let a = polygon[index];
        let b = polygon[(index + 1) % polygon.len()];
        let edge = b - a;

        // Outward normal of a counter-clockwise edge.
        let normal = Vec2::new(edge.y, -edge.x);
        let length = normal.length();
        if length <= f32::EPSILON {
            continue;
        }

        let normal = normal / length;
        result = clip(&result, normal, a.dot(normal) - amount);

        if result.len() < 3 {
            return result;
        }
    }

    result
}

fn centroid(polygon: &[Vec2]) -> Vec2 {
    polygon.iter().fold(Vec2::ZERO, |sum, corner| sum + *corner) / polygon.len() as f32
}

/// Distance from `centre` to the nearest edge — how much room the piece really
/// offers a thumb, as opposed to how wide its bounding box is.
fn inradius(polygon: &[Vec2], centre: Vec2) -> f32 {
    let mut nearest = f32::MAX;

    for index in 0..polygon.len() {
        let a = polygon[index];
        let b = polygon[(index + 1) % polygon.len()];
        let edge = b - a;
        let length = edge.length();

        if length <= f32::EPSILON {
            continue;
        }

        let distance = ((b.x - a.x) * (a.y - centre.y) - (a.x - centre.x) * (b.y - a.y)).abs()
            / length;
        nearest = nearest.min(distance);
    }

    nearest
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rng() -> StdRng {
        StdRng::seed_from_u64(8_2026)
    }

    const MIN: Vec2 = Vec2::new(-180.0, -300.0);
    const MAX: Vec2 = Vec2::new(180.0, 260.0);

    fn corners_in_world(piece: &Piece) -> Vec<Vec2> {
        piece.corners.iter().map(|corner| *corner + piece.centre).collect()
    }

    /// Two pieces sharing space would give the player a tap that belongs to
    /// both, and a hidden piece partly covered by a visible one.
    #[test]
    fn pieces_never_overlap() {
        let mut rng = rng();

        for count in 6..=14 {
            let pieces = layout(MIN, MAX, count, &mut rng);

            for (index, piece) in pieces.iter().enumerate() {
                for other in pieces.iter().skip(index + 1) {
                    for corner in corners_in_world(piece) {
                        assert!(
                            !other.contains(corner),
                            "a corner of one piece is inside another"
                        );
                    }
                    assert!(!other.contains(piece.centre));
                    assert!(!piece.contains(other.centre));
                }
            }
        }
    }

    #[test]
    fn pieces_stay_inside_the_area() {
        let mut rng = rng();

        for piece in layout(MIN, MAX, 12, &mut rng) {
            for corner in corners_in_world(&piece) {
                assert!(corner.x >= MIN.x - 0.5 && corner.x <= MAX.x + 0.5, "{:?}", corner);
                assert!(corner.y >= MIN.y - 0.5 && corner.y <= MAX.y + 0.5, "{:?}", corner);
            }
        }
    }

    /// Every piece has to be worth aiming at, because any of them may be the
    /// one the player cannot see.
    #[test]
    fn pieces_stay_tappable() {
        let mut rng = rng();

        for piece in layout(MIN, MAX, 14, &mut rng) {
            let world = corners_in_world(&piece);
            assert!(
                inradius(&world, piece.centre) >= MIN_INRADIUS - 0.5,
                "{:?} is too thin to aim at",
                piece
            );
            assert!(piece.contains(piece.centre));
        }
    }

    /// The pieces are a honeycomb, not a grid.
    ///
    /// This is the test that catches the layout quietly regressing into
    /// quadrilaterals — which is what happens without the staggered rows, or
    /// without the ring of seeds outside the frame. Measured across many
    /// boards, four-sided pieces sit near 4%; the bar is set well clear of
    /// that so it fails on a real change and not on the dice.
    #[test]
    fn pieces_are_many_sided() {
        let mut rng = rng();
        let mut many_sided = 0;
        let mut total = 0;

        for _ in 0..60 {
            for piece in layout(MIN, MAX, 12, &mut rng) {
                total += 1;
                if piece.corners.len() >= 5 {
                    many_sided += 1;
                }
            }
        }

        assert!(
            many_sided * 100 >= total * 80,
            "only {} of {} pieces have five or more sides",
            many_sided,
            total
        );
    }

    /// No two rounds put pieces in the same places, and a round's pieces are
    /// not all the same size.
    #[test]
    fn the_layout_is_irregular() {
        let mut rng = rng();

        let first = layout(MIN, MAX, 9, &mut rng);
        let second = layout(MIN, MAX, 9, &mut rng);
        assert_ne!(first, second, "two rounds produced the same board");

        let widths: Vec<f32> = first.iter().map(|piece| piece.half_extents().x).collect();
        let spread = widths.iter().cloned().fold(f32::MIN, f32::max)
            - widths.iter().cloned().fold(f32::MAX, f32::min);

        assert!(spread > 10.0, "pieces are all but the same size: {:?}", widths);
    }
}


