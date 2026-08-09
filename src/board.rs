//! The honeycomb the board is drawn on.
//!
//! A regular pointy-top hex lattice, sized so it fills the play area, with an
//! even gap between neighbours. Every cell is the same hexagon — congruent, six
//! equal sides — because the round is read as a difference *between* cells, and
//! the eye can only difference against a field it takes as uniform.
//!
//! ## The lattice, and what makes the round fair
//!
//! A regular lattice puts every cell at a predictable address, and the answer is
//! a cell painted the background color, so it is a hole at a place the eye can
//! find without comparing anything. On its own that would be trivial. Two things
//! carry the round instead:
//!
//! - the board also contains cells that are *deliberately* empty (see
//!   `src/mosaic_pattern.rs`), so a hole is not by itself the answer, and
//! - the background sweeps every color on the board before settling on the
//!   answer's, so the player who watched knows which hole appeared last.
//!
//! That is why `layout` reports each piece's `column`/`row`: the pattern
//! generator needs to know which cells are neighbours, and a mosaic of scattered
//! single cells is noise rather than a pattern.
//!
//! Kept free of Bevy beyond `Vec2` so the invariants below can be tested.

use bevy::prelude::Vec2;

const SQRT_3: f32 = 1.732_050_8;

/// One cell of the honeycomb: a regular hexagon, given as its centre, the
/// corners around it, and where it sits on the lattice.
///
/// Corners are relative to `centre` so the piece can be spawned at a transform
/// and drawn from the same points every time — including on replay.
#[derive(Debug, Clone, PartialEq)]
pub struct Piece {
    pub centre: Vec2,
    pub corners: Vec<Vec2>,
    pub column: usize,
    pub row: usize,
}

/// Whether a world-space point falls inside a convex polygon given as a centre
/// and corners relative to it.
///
/// The polygon is wound counter-clockwise, so a point is inside when it is to
/// the left of every edge. Free-standing because the pieces on screen carry
/// their outline without carrying a whole `Piece`.
pub fn contains(centre: Vec2, corners: &[Vec2], point: Vec2) -> bool {
    let local = point - centre;
    let count = corners.len();

    if count < 3 {
        return false;
    }

    (0..count).all(|index| {
        let a = corners[index];
        let b = corners[(index + 1) % count];
        let edge = b - a;
        edge.x * (local.y - a.y) - edge.y * (local.x - a.x) >= -f32::EPSILON
    })
}

impl Piece {
    pub fn contains(&self, point: Vec2) -> bool {
        contains(self.centre, &self.corners, point)
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

/// Gap between neighbouring cells, as a share of the circumradius and then
/// clamped. Scaling it with the cell keeps the board looking the same at four
/// columns and at sixteen; the clamps stop it vanishing on a dense board or
/// eating a sparse one.
const GAP_RATIO: f32 = 0.13;
const MIN_GAP: f32 = 2.0;
const MAX_GAP: f32 = 7.0;

/// Cells stop growing here, so a four-column board on a desktop window does not
/// become four billboards.
const MAX_APOTHEM: f32 = 80.0;

/// Fewest and most columns the difficulty curve may ask for.
pub const MIN_COLUMNS: usize = 4;
pub const MAX_COLUMNS: usize = 16;

/// Fills the rectangle between `min` and `max` with a regular pointy-top hex
/// lattice of `columns` columns.
///
/// Rows are not a free parameter: with a *regular* hexagon the row spacing is
/// fixed by the column spacing, so the shape of the play area decides how many
/// rows there are. On a phone that is roughly 2.2 rows per column.
pub fn layout(min: Vec2, max: Vec2, columns: usize) -> Vec<Piece> {
    let columns = columns.clamp(MIN_COLUMNS, MAX_COLUMNS);
    let area = max - min;

    // Even rows hold `columns` hexes and span exactly `columns * width`.
    let width = (area.x / columns as f32).min(2.0 * MAX_APOTHEM);
    let radius = width / SQRT_3;

    // Lattice height is radius * (0.5 + 1.5 * rows): every row after the first
    // adds three quarters of a hexagon.
    let rows = (((area.y / radius) - 0.5) / 1.5).floor().max(1.0) as usize;

    let lattice = Vec2::new(
        columns as f32 * width,
        radius * (0.5 + 1.5 * rows as f32),
    );
    let origin = min + (area - lattice) / 2.0;

    let gap = (GAP_RATIO * radius).clamp(MIN_GAP, MAX_GAP);
    let corners = hexagon(radius - gap / SQRT_3);

    let mut pieces = Vec::with_capacity(rows * columns);

    for row in 0..rows {
        // Odd rows are shifted half a cell and hold one fewer, so they end up
        // centred inside the even rows instead of leaving a ragged edge.
        let odd = row % 2 == 1;
        let in_row = if odd { columns.saturating_sub(1) } else { columns };

        for column in 0..in_row {
            let centre = origin
                + Vec2::new(
                    width * (0.5 + column as f32 + if odd { 0.5 } else { 0.0 }),
                    radius * (1.0 + 1.5 * row as f32),
                );

            pieces.push(Piece {
                centre,
                corners: corners.clone(),
                column,
                row,
            });
        }
    }

    pieces
}

/// The six corners of a pointy-top hexagon, counter-clockwise from the top.
fn hexagon(radius: f32) -> Vec<Vec2> {
    (0..6)
        .map(|corner| {
            let angle = std::f32::consts::FRAC_PI_2 + corner as f32 * std::f32::consts::FRAC_PI_3;
            Vec2::new(radius * angle.cos(), radius * angle.sin())
        })
        .collect()
}

/// The lattice coordinates touching `(column, row)`.
///
/// Odd rows carry both the half-cell shift and one fewer cell, so which
/// neighbours a cell has depends on the parity of its row. Getting this wrong
/// does not crash anything — it quietly produces patterns whose "blobs" are not
/// actually connected.
pub fn neighbours(column: usize, row: usize) -> [(isize, isize); 6] {
    let column = column as isize;
    let row = row as isize;

    if row % 2 == 0 {
        [
            (column - 1, row),
            (column + 1, row),
            (column - 1, row - 1),
            (column, row - 1),
            (column - 1, row + 1),
            (column, row + 1),
        ]
    } else {
        [
            (column - 1, row),
            (column + 1, row),
            (column, row - 1),
            (column + 1, row - 1),
            (column, row + 1),
            (column + 1, row + 1),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIN: Vec2 = Vec2::new(-179.0, -290.0);
    const MAX: Vec2 = Vec2::new(179.0, 262.0);

    fn corners_in_world(piece: &Piece) -> Vec<Vec2> {
        piece
            .corners
            .iter()
            .map(|corner| *corner + piece.centre)
            .collect()
    }

    /// Distance from the centre to the nearest edge — what the thumb actually
    /// gets, as opposed to how wide the bounding box is.
    fn inradius(piece: &Piece) -> f32 {
        let corners = &piece.corners;
        let mut nearest = f32::MAX;

        for index in 0..corners.len() {
            let a = corners[index];
            let b = corners[(index + 1) % corners.len()];
            let edge = b - a;
            let length = edge.length();
            if length <= f32::EPSILON {
                continue;
            }
            nearest = nearest.min((edge.x * -a.y - -a.x * edge.y).abs() / length);
        }

        nearest
    }

    /// Two pieces sharing space would give the player a tap belonging to both.
    #[test]
    fn pieces_never_overlap() {
        for columns in MIN_COLUMNS..=MAX_COLUMNS {
            let pieces = layout(MIN, MAX, columns);

            for (index, piece) in pieces.iter().enumerate() {
                for other in pieces.iter().skip(index + 1) {
                    assert!(!other.contains(piece.centre), "{} columns", columns);
                    for corner in corners_in_world(piece) {
                        assert!(
                            !other.contains(corner),
                            "{} columns: a corner sits inside another piece",
                            columns
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn pieces_stay_inside_the_area() {
        for columns in MIN_COLUMNS..=MAX_COLUMNS {
            for piece in layout(MIN, MAX, columns) {
                for corner in corners_in_world(&piece) {
                    assert!(corner.x >= MIN.x - 0.5 && corner.x <= MAX.x + 0.5);
                    assert!(corner.y >= MIN.y - 0.5 && corner.y <= MAX.y + 0.5);
                }
            }
        }
    }

    /// The whole point of a regular lattice: the field the eye differences
    /// against is uniform, so every cell has to be the same hexagon.
    #[test]
    fn every_piece_is_the_same_hexagon() {
        for columns in MIN_COLUMNS..=MAX_COLUMNS {
            let pieces = layout(MIN, MAX, columns);
            let first = pieces.first().expect("a board has pieces").corners.clone();

            for piece in &pieces {
                assert_eq!(piece.corners.len(), 6);

                for (corner, expected) in piece.corners.iter().zip(&first) {
                    assert!(
                        (*corner - *expected).length() < 1e-3,
                        "{} columns: pieces are not congruent",
                        columns
                    );
                }

                // Equal sides, not merely six of them.
                let sides: Vec<f32> = (0..6)
                    .map(|index| (piece.corners[(index + 1) % 6] - piece.corners[index]).length())
                    .collect();
                let longest = sides.iter().cloned().fold(f32::MIN, f32::max);
                let shortest = sides.iter().cloned().fold(f32::MAX, f32::min);
                assert!(longest - shortest < 1e-3, "sides are uneven: {:?}", sides);
            }
        }
    }

    /// Neighbours must be mutual. A one-sided neighbour list produces "blobs"
    /// that are not connected, which is invisible until the board looks wrong.
    #[test]
    fn neighbours_are_symmetric() {
        let pieces = layout(MIN, MAX, 8);
        let present: std::collections::HashSet<(usize, usize)> =
            pieces.iter().map(|piece| (piece.column, piece.row)).collect();

        for piece in &pieces {
            for (column, row) in neighbours(piece.column, piece.row) {
                if column < 0 || row < 0 {
                    continue;
                }
                let neighbour = (column as usize, row as usize);
                if !present.contains(&neighbour) {
                    continue;
                }

                let back = neighbours(neighbour.0, neighbour.1);
                assert!(
                    back.contains(&(piece.column as isize, piece.row as isize)),
                    "{:?} lists {:?} but not the other way round",
                    (piece.column, piece.row),
                    neighbour
                );
            }
        }
    }

    /// Neighbours must also be *adjacent in space*, not just in the index
    /// arithmetic — this is what catches an off-by-one in the odd-row shift.
    #[test]
    fn neighbours_actually_touch() {
        let pieces = layout(MIN, MAX, 8);
        let by_cell: std::collections::HashMap<(usize, usize), &Piece> = pieces
            .iter()
            .map(|piece| ((piece.column, piece.row), piece))
            .collect();

        let step = {
            let a = by_cell[&(0, 0)];
            let b = by_cell[&(1, 0)];
            (b.centre - a.centre).length()
        };

        for piece in &pieces {
            for (column, row) in neighbours(piece.column, piece.row) {
                if column < 0 || row < 0 {
                    continue;
                }
                let Some(other) = by_cell.get(&(column as usize, row as usize)) else {
                    continue;
                };

                let distance = (other.centre - piece.centre).length();
                assert!(
                    (distance - step).abs() < step * 0.05,
                    "{:?} and {:?} are {} apart, not {}",
                    (piece.column, piece.row),
                    (column, row),
                    distance,
                    step
                );
            }
        }
    }

    /// A board that fills the screen. Sixteen columns on a phone puts the cells
    /// under the 48px touch guideline — that is a deliberate trade, and this
    /// test records where the floor actually is so a change to the geometry
    /// cannot lower it silently.
    #[test]
    fn cells_keep_a_usable_size() {
        let coarse = layout(MIN, MAX, MIN_COLUMNS);
        assert!(inradius(&coarse[0]) > 30.0);

        let dense = layout(MIN, MAX, MAX_COLUMNS);
        assert!(
            inradius(&dense[0]) > 8.0,
            "sixteen columns leaves an inradius of {}",
            inradius(&dense[0])
        );

        // And the board is worth calling a board.
        assert!(dense.len() > 200, "only {} cells at 16 columns", dense.len());
    }
}
