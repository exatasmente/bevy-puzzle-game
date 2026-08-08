//! Wave function collapse over a small tile grid, for the `Mosaic` round.
//!
//! The other modes ask "which color is different". This one asks "which piece
//! does not fit", which is a different question for the player: not a judgement
//! about two colors, but a search for a broken rule. It needs a board that
//! *has* a rule — a tiling where every piece agrees with its neighbours — and
//! then exactly one piece that does not.
//!
//! WFC is the right generator for that and a poor one for most other things.
//! Its whole value is local coherence: it fills the grid with pieces that are
//! guaranteed to line up. Used to draw shapes for their own sake it would be
//! an expensive random number generator; used here, the guarantee *is* the
//! puzzle, because a break in it is only findable if everything else holds.
//!
//! ## The tiles
//!
//! Each tile is a piece of pipe: its four edges either carry an arm or do not.
//! Two neighbours fit when the edges they share agree. The set is deliberately
//! *incomplete* — there is no dead end (one arm) and no cross (four arms):
//!
//! | kind       | arms | rotations |
//! | ---------- | ---- | --------- |
//! | `Empty`    | 0    | 1         |
//! | `Straight` | 2    | 2         |
//! | `Corner`   | 2    | 4         |
//! | `Tee`      | 3    | 4         |
//!
//! Eleven tiles for sixteen possible edge patterns. That gap is what makes the
//! constraints bite: with a complete set every assignment of edges would be
//! realizable, propagation would never rule anything out, and "WFC" would be a
//! grand name for rolling a die per cell.
//!
//! This module is deliberately free of Bevy types so it can be tested on its
//! own — which it is, at the bottom of the file.

use rand::prelude::*;

/// Edge indices, clockwise from the top.
pub const TOP: usize = 0;
pub const RIGHT: usize = 1;
pub const BOTTOM: usize = 2;
pub const LEFT: usize = 3;

/// The edge a neighbour in direction `dir` presents back to us.
fn opposite(dir: usize) -> usize {
    (dir + 2) % 4
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileKind {
    Empty,
    Straight,
    Corner,
    Tee,
}

/// A tile and how far it has been turned, in clockwise quarter turns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tile {
    pub kind: TileKind,
    pub rotation: u8,
}

impl Tile {
    pub fn new(kind: TileKind, rotation: u8) -> Self {
        Self {
            kind,
            rotation: rotation % 4,
        }
    }

    /// Which edges carry an arm, clockwise from the top.
    pub fn edges(&self) -> [bool; 4] {
        let base = match self.kind {
            TileKind::Empty => [false, false, false, false],
            // Top to bottom.
            TileKind::Straight => [true, false, true, false],
            // Top to right.
            TileKind::Corner => [true, true, false, false],
            // Everything but the left.
            TileKind::Tee => [true, true, true, false],
        };

        let rotation = (self.rotation % 4) as usize;
        let mut edges = [false; 4];
        for index in 0..4 {
            // Turning the tile clockwise moves the top edge to the right, so
            // the edge now at `index` is the one that was `rotation` steps
            // anticlockwise of it.
            edges[index] = base[(index + 4 - rotation) % 4];
        }

        edges
    }

}

/// Every distinct tile. Rotations that produce a tile already in the set are
/// left out, so each edge pattern appears exactly once.
pub fn tile_pool() -> Vec<Tile> {
    let mut pool: Vec<Tile> = Vec::new();

    for kind in [
        TileKind::Empty,
        TileKind::Straight,
        TileKind::Corner,
        TileKind::Tee,
    ] {
        for rotation in 0..4 {
            let tile = Tile::new(kind, rotation);
            if !pool.iter().any(|existing| existing.edges() == tile.edges()) {
                pool.push(tile);
            }
        }
    }

    pool
}

/// A generated board: a tiling that fits together everywhere except at one
/// cell.
#[derive(Debug, Clone)]
pub struct Mosaic {
    pub columns: usize,
    pub rows: usize,
    pub tiles: Vec<Tile>,
    /// The cell that does not fit — the answer.
    pub broken: usize,
    /// How many of its four edges disagree with what surrounds them. This is
    /// the difficulty dial: four is unmissable, one is a single short arm
    /// pointing at nothing.
    pub violations: usize,
}

impl Mosaic {
    pub fn tile(&self, column: usize, row: usize) -> Tile {
        self.tiles[row * self.columns + column]
    }

    /// The edge value a cell's neighbour presents in direction `dir`.
    ///
    /// Off the edge of the board counts as "no arm": the board is a closed
    /// composition, and an arm running off it reads as broken whether or not
    /// there is a neighbour to disagree with.
    fn expected_edge(&self, index: usize, dir: usize) -> bool {
        match self.neighbour(index, dir) {
            Some(neighbour) => self.tiles[neighbour].edges()[opposite(dir)],
            None => false,
        }
    }

    fn neighbour(&self, index: usize, dir: usize) -> Option<usize> {
        neighbour_of(index, dir, self.columns, self.rows)
    }

    /// How many of a cell's edges disagree with their surroundings.
    pub fn violations_at(&self, index: usize) -> usize {
        let edges = self.tiles[index].edges();
        (0..4)
            .filter(|dir| edges[*dir] != self.expected_edge(index, *dir))
            .count()
    }
}

/// Bitmask over `tile_pool()` indices. Eleven tiles, so a `u16` is plenty.
type Domain = u16;

/// Generates a coherent tiling and then breaks exactly one cell of it.
///
/// `violations` is a request, not a promise. Two constraints can get in the
/// way: the pool has no tile one edge away from `Empty` (that would be a dead
/// end), and a break has to stay unambiguous — see [`corrupt`]. The generator
/// tries every cell before settling for the closest it can do, and reports
/// what it actually produced.
pub fn generate(columns: usize, rows: usize, violations: usize, rng: &mut impl Rng) -> Mosaic {
    let pool = tile_pool();
    let tiles = solve(columns, rows, &pool, rng).unwrap_or_else(|| {
        // An all-empty board satisfies every constraint. Reaching this means
        // the solver hit contradictions on every attempt, which should not
        // happen on grids this size — but a boring board beats a panic.
        vec![Tile::new(TileKind::Empty, 0); columns * rows]
    });

    let mut mosaic = Mosaic {
        columns,
        rows,
        tiles,
        broken: 0,
        violations: 0,
    };

    corrupt(&mut mosaic, &pool, violations.clamp(1, 4), rng);
    mosaic
}

/// Share of cells that must carry a piece for the board to be worth playing.
///
/// An all-empty tiling satisfies every constraint, so the solver is perfectly
/// happy to produce one — and it makes a hopeless round: with no pattern
/// around it, the broken piece is the only thing on screen and the player is
/// not checking a rule, just spotting the one object. The generator keeps
/// solving until the board has something to say.
const MIN_FILLED: f32 = 0.6;

/// Fills the grid with tiles that all agree with their neighbours.
///
/// Returns `None` if every attempt ran into a contradiction.
fn solve(columns: usize, rows: usize, pool: &[Tile], rng: &mut impl Rng) -> Option<Vec<Tile>> {
    // Grids here are at most a few dozen cells, so restarting costs less than
    // the bookkeeping a backtracking solver would need — and it doubles as the
    // retry for a board that came out too sparse.
    const ATTEMPTS: usize = 32;

    let cells = (columns * rows) as f32;
    let mut best: Option<(Vec<Tile>, usize)> = None;

    for _ in 0..ATTEMPTS {
        let Some(tiles) = attempt(columns, rows, pool, rng) else {
            continue;
        };

        let filled = tiles
            .iter()
            .filter(|tile| tile.kind != TileKind::Empty)
            .count();

        if filled as f32 / cells >= MIN_FILLED {
            return Some(tiles);
        }

        if best.as_ref().map(|(_, most)| filled > *most).unwrap_or(true) {
            best = Some((tiles, filled));
        }
    }

    best.map(|(tiles, _)| tiles)
}

fn attempt(columns: usize, rows: usize, pool: &[Tile], rng: &mut impl Rng) -> Option<Vec<Tile>> {
    let cells = columns * rows;
    let full: Domain = (1 << pool.len()) - 1;
    let mut domains = vec![full; cells];

    // The board is closed: no arm may point off it. This is what gives the
    // border cells something to satisfy, and it is why the finished mosaic
    // reads as one object instead of a crop of a larger pattern.
    for index in 0..cells {
        let column = index % columns;
        let row = index / columns;

        for dir in 0..4 {
            let outside = match dir {
                TOP => row == 0,
                BOTTOM => row + 1 == rows,
                RIGHT => column + 1 == columns,
                LEFT => column == 0,
                _ => false,
            };

            if outside {
                domains[index] &= mask_with_edge(pool, dir, false);
            }
        }

        if domains[index] == 0 {
            return None;
        }
    }

    propagate(&mut domains, columns, rows, pool)?;

    while let Some(index) = lowest_entropy(&domains, rng) {
        let chosen = choose(domains[index], pool, rng)?;
        domains[index] = 1 << chosen;
        propagate(&mut domains, columns, rows, pool)?;
    }

    domains
        .iter()
        .map(|domain| Some(pool[domain.trailing_zeros() as usize]))
        .collect()
}

/// The set of tiles whose edge in `dir` has the given value.
fn mask_with_edge(pool: &[Tile], dir: usize, value: bool) -> Domain {
    let mut mask = 0;
    for (index, tile) in pool.iter().enumerate() {
        if tile.edges()[dir] == value {
            mask |= 1 << index;
        }
    }
    mask
}

/// Narrows every domain until nothing more can be ruled out.
///
/// Returns `None` on a contradiction — a cell with no tile left.
fn propagate(
    domains: &mut [Domain],
    columns: usize,
    rows: usize,
    pool: &[Tile],
) -> Option<()> {
    let mut queue: Vec<usize> = (0..domains.len()).collect();

    while let Some(index) = queue.pop() {
        for dir in 0..4 {
            let Some(neighbour) = neighbour_of(index, dir, columns, rows) else {
                continue;
            };

            // Whatever this cell ends up being, its edge in `dir` is one of
            // these values — so the neighbour's facing edge must be one of
            // them too.
            let mut allowed: Domain = 0;
            for value in [true, false] {
                if domains[index] & mask_with_edge(pool, dir, value) != 0 {
                    allowed |= mask_with_edge(pool, opposite(dir), value);
                }
            }

            let narrowed = domains[neighbour] & allowed;
            if narrowed == 0 {
                return None;
            }

            if narrowed != domains[neighbour] {
                domains[neighbour] = narrowed;
                queue.push(neighbour);
            }
        }
    }

    Some(())
}

/// Cells are indexed the way `BoardGrid` lays them out: left to right, then
/// top to bottom, so row 0 is the *top* row. Getting this backwards mirrors
/// every tile vertically — adjacency still holds, so the tests would pass, but
/// arms would be drawn pointing the wrong way.
fn neighbour_of(index: usize, dir: usize, columns: usize, rows: usize) -> Option<usize> {
    let column = index % columns;
    let row = index / columns;

    match dir {
        TOP => (row > 0).then(|| (row - 1) * columns + column),
        BOTTOM => (row + 1 < rows).then(|| (row + 1) * columns + column),
        RIGHT => (column + 1 < columns).then(|| row * columns + column + 1),
        LEFT => (column > 0).then(|| row * columns + column - 1),
        _ => None,
    }
}

/// The undecided cell with the fewest options left, ties broken at random.
fn lowest_entropy(domains: &[Domain], rng: &mut impl Rng) -> Option<usize> {
    let mut best = usize::MAX;
    let mut candidates: Vec<usize> = Vec::new();

    for (index, domain) in domains.iter().enumerate() {
        let options = domain.count_ones() as usize;
        if options <= 1 {
            continue;
        }

        if options < best {
            best = options;
            candidates.clear();
        }

        if options == best {
            candidates.push(index);
        }
    }

    candidates.choose(rng).copied()
}

/// Picks one tile out of a domain.
///
/// `Empty` is weighted down: it fits beside anything that presents no arm, so
/// an unweighted choice fills most of the board with blanks and leaves nothing
/// for the eye to check.
fn choose(domain: Domain, pool: &[Tile], rng: &mut impl Rng) -> Option<usize> {
    let mut options: Vec<(usize, u32)> = Vec::new();
    let mut total = 0;

    for (index, tile) in pool.iter().enumerate() {
        if domain & (1 << index) == 0 {
            continue;
        }

        // Empty fits beside anything that presents no arm, so an unweighted
        // choice snowballs: one blank cell makes blank neighbours cheap, and
        // the board drains.
        let weight = if tile.kind == TileKind::Empty { 1 } else { 6 };
        total += weight;
        options.push((index, weight));
    }

    if options.is_empty() {
        return None;
    }

    let mut roll = rng.gen_range(0..total);
    for (index, weight) in options {
        if roll < weight {
            return Some(index);
        }
        roll -= weight;
    }

    None
}

/// Replaces one cell's tile with one that disagrees with its surroundings.
///
/// ## Why the answer has to be the cell with the *most* violations
///
/// A disagreement belongs to an edge, not to a cell: if this tile grows an arm
/// its neighbour does not meet, both of them now have a bad edge, and the
/// picture gives the player no way to tell which of the two was changed. So a
/// single interior violation is not a hard round, it is an unfair one — there
/// are two defensible answers and the game accepts one.
///
/// Two or more violations make the answer well defined again: the broken cell
/// carries all of them, while each implicated neighbour carries exactly one.
/// "The piece that is wrong on more than one side" is a rule the player can
/// actually apply.
///
/// One violation is still available, but only against the edge of the board,
/// where the other party to the disagreement is the void — which cannot be at
/// fault. An arm running off the board is unambiguous.
fn corrupt(mosaic: &mut Mosaic, pool: &[Tile], wanted: usize, rng: &mut impl Rng) {
    let mut cells: Vec<usize> = (0..mosaic.tiles.len()).collect();
    cells.shuffle(rng);

    // Any unambiguous break, kept in case the exact request cannot be met.
    let mut fallback: Option<(usize, Tile, usize)> = None;

    for index in cells {
        let expected: Vec<bool> = (0..4).map(|dir| mosaic.expected_edge(index, dir)).collect();
        let original = mosaic.tiles[index];

        let mut candidates: Vec<Tile> = pool
            .iter()
            .copied()
            .filter(|tile| tile.edges() != original.edges())
            .collect();
        candidates.shuffle(rng);

        for candidate in candidates {
            let edges = candidate.edges();
            let broken_dirs: Vec<usize> =
                (0..4).filter(|dir| edges[*dir] != expected[*dir]).collect();
            let mismatches = broken_dirs.len();

            let unambiguous = mismatches >= 2
                || broken_dirs
                    .iter()
                    .all(|dir| mosaic.neighbour(index, *dir).is_none());

            if mismatches == 0 || !unambiguous {
                continue;
            }

            if mismatches == wanted {
                mosaic.tiles[index] = candidate;
                mosaic.broken = index;
                mosaic.violations = mismatches;
                return;
            }

            let closer = fallback
                .map(|(_, _, best)| {
                    (best as isize - wanted as isize).abs()
                        > (mismatches as isize - wanted as isize).abs()
                })
                .unwrap_or(true);

            if closer {
                fallback = Some((index, candidate, mismatches));
            }
        }
    }

    if let Some((index, tile, mismatches)) = fallback {
        mosaic.tiles[index] = tile;
        mosaic.broken = index;
        mosaic.violations = mismatches;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rng() -> StdRng {
        StdRng::seed_from_u64(20_260_808)
    }

    #[test]
    fn every_edge_pattern_appears_at_most_once() {
        let pool = tile_pool();
        for (index, tile) in pool.iter().enumerate() {
            for other in pool.iter().skip(index + 1) {
                assert_ne!(tile.edges(), other.edges(), "duplicate edge pattern");
            }
        }
    }

    #[test]
    fn the_pool_is_incomplete_on_purpose() {
        // Dead ends and crosses are excluded; without that gap the constraints
        // would never rule anything out. If this fails, propagation has quietly
        // become a no-op.
        let pool = tile_pool();
        assert_eq!(pool.len(), 11);
        assert!(!pool.iter().any(|tile| tile.edges() == [true; 4]));
        assert!(!pool
            .iter()
            .any(|tile| tile.edges().iter().filter(|edge| **edge).count() == 1));
    }

    #[test]
    fn rotation_turns_clockwise() {
        let corner = Tile::new(TileKind::Corner, 0);
        assert_eq!(corner.edges(), [true, true, false, false]);

        let turned = Tile::new(TileKind::Corner, 1);
        assert_eq!(turned.edges(), [false, true, true, false]);
    }

    /// The round has exactly one defensible answer.
    ///
    /// Not "only the broken cell has a bad edge" — it cannot be, since a
    /// disagreement is shared by both cells on either side of it. What has to
    /// hold is that the broken cell is the *only* one the rule "most bad
    /// edges" can point at. Without this the game marks one of two equally
    /// suspect pieces correct and the other a miss.
    #[test]
    fn the_broken_cell_is_the_only_defensible_answer() {
        let mut rng = rng();

        for (columns, rows) in [(2, 3), (3, 3), (3, 4), (4, 4), (4, 5)] {
            for wanted in 1..=4 {
                let mosaic = generate(columns, rows, wanted, &mut rng);
                let broken = mosaic.violations_at(mosaic.broken);

                assert!(broken > 0, "the broken cell must actually be broken");
                assert_eq!(broken, mosaic.violations, "reported count is wrong");

                for index in 0..mosaic.tiles.len() {
                    if index == mosaic.broken {
                        continue;
                    }

                    assert!(
                        mosaic.violations_at(index) < broken,
                        "{}x{} cell {} is as suspect as the answer",
                        columns,
                        rows,
                        index
                    );
                }
            }
        }
    }

    /// A single violation is only fair against the edge of the board, where
    /// the other party to the disagreement is the void.
    #[test]
    fn a_lone_violation_points_off_the_board() {
        let mut rng = rng();

        for _ in 0..40 {
            let mosaic = generate(4, 4, 1, &mut rng);
            if mosaic.violations != 1 {
                continue;
            }

            let index = mosaic.broken;
            let edges = mosaic.tiles[index].edges();
            let broken_dir = (0..4)
                .find(|dir| edges[*dir] != mosaic.expected_edge(index, *dir))
                .expect("a broken cell has a broken edge");

            assert!(
                mosaic.neighbour(index, broken_dir).is_none(),
                "a lone violation between two cells is ambiguous"
            );
        }
    }

    /// A board the player can actually read a rule off.
    #[test]
    fn the_board_is_not_mostly_empty() {
        let mut rng = rng();

        for (columns, rows) in [(2, 3), (3, 3), (3, 4), (4, 4), (4, 5)] {
            let mosaic = generate(columns, rows, 2, &mut rng);
            let filled = mosaic
                .tiles
                .iter()
                .filter(|tile| tile.kind != TileKind::Empty)
                .count();

            assert!(
                filled * 2 >= mosaic.tiles.len(),
                "{}x{} board has only {} pieces on {} cells",
                columns,
                rows,
                filled,
                mosaic.tiles.len()
            );
        }
    }

    /// The impostor must not be recognisable by its shape alone.
    ///
    /// At one broken edge, and at three, the tile set leaves exactly one
    /// reachable answer — a three-armed piece — so the round became "find the
    /// T" and the pattern stopped mattering. The game asks for two or four; if
    /// a future change reintroduces the others, this fails.
    #[test]
    fn the_impostor_is_not_always_the_same_shape() {
        let mut rng = rng();

        for wanted in [2, 4] {
            let mut kinds = std::collections::BTreeSet::new();

            for _ in 0..150 {
                let mosaic = generate(3, 4, wanted, &mut rng);
                kinds.insert(format!("{:?}", mosaic.tiles[mosaic.broken].kind));
            }

            assert!(
                kinds.len() >= 2,
                "{} broken edges always produces {:?}",
                wanted,
                kinds
            );
        }
    }

    #[test]
    fn the_requested_difficulty_is_usually_met() {
        let mut rng = rng();
        let mut met = 0;
        let attempts = 40;

        for _ in 0..attempts {
            let mosaic = generate(3, 4, 2, &mut rng);
            if mosaic.violations == 2 {
                met += 1;
            }
        }

        // Not all boards can be broken by a single edge — see `generate`. This
        // guards against the weaker claim silently becoming the common case.
        assert!(met > attempts / 2, "only {} of {} boards hit the target", met, attempts);
    }

    #[test]
    fn no_arm_leaves_the_board_except_at_the_break() {
        let mut rng = rng();
        let mosaic = generate(3, 4, 2, &mut rng);

        for index in 0..mosaic.tiles.len() {
            if index == mosaic.broken {
                continue;
            }

            let column = index % mosaic.columns;
            let row = index / mosaic.columns;
            let edges = mosaic.tiles[index].edges();

            if row == 0 {
                assert!(!edges[TOP]);
            }
            if row + 1 == mosaic.rows {
                assert!(!edges[BOTTOM]);
            }
            if column == 0 {
                assert!(!edges[LEFT]);
            }
            if column + 1 == mosaic.columns {
                assert!(!edges[RIGHT]);
            }
        }
    }
}

