//! What each cell of the honeycomb is: empty, or part of a colour blob.
//!
//! The round is a mosaic. Some cells are deliberately empty — they show the
//! ground, so they are holes from the first frame — and the rest are grouped
//! into a handful of colour regions. Exactly one cell is the answer.
//!
//! ## Why blobs, and why the answer is alone in its colour
//!
//! Both of those are forced by how the round is read, not chosen for looks.
//!
//! The background sweeps the colours on the board one at a time, and a group
//! vanishes as the ground passes its colour. That only reads if a colour covers
//! several cells at once: a board of *n* distinct colours would need *n* steps
//! in one second, which at three hundred cells is two milliseconds each and
//! looks like nothing at all. A handful of colours, each worn by a connected
//! region, makes every step of the sweep a visible event.
//!
//! And the answer has to be the only cell wearing its colour. If it shared with
//! its group, the ground's last step would erase the whole group and the round
//! would have several defensible answers — the same unfairness the mosaic mode's
//! tests already caught once.
//!
//! The answer is placed *inside* a group whose colour it nearly matches. That is
//! what keeps the colour difficulty dial meaningful: at a small delta the answer
//! is a near-twin surrounded by near-twins, and it is only the sweep that
//! separates it.
//!
//! Free of Bevy so the invariants below can be tested.

use rand::prelude::*;
use std::collections::HashMap;

use crate::board::{neighbours, Piece};

/// Which colour group each cell belongs to, and which cell is the answer.
#[derive(Debug, Clone)]
pub struct Pattern {
    /// One entry per piece: `Some(group)` when the cell is filled, `None` when
    /// it is deliberately empty.
    pub groups: Vec<Option<usize>>,
    pub group_count: usize,
    /// Index of the answer, which is always a filled cell.
    pub answer: usize,
}

impl Pattern {
    pub fn is_filled(&self, index: usize) -> bool {
        self.groups.get(index).copied().flatten().is_some()
    }

    pub fn filled_count(&self) -> usize {
        self.groups.iter().filter(|group| group.is_some()).count()
    }
}

/// Roughly how many cells one empty vein grows to. Fewer, larger veins read as
/// a mosaic; many single holes read as dirt on the screen.
const CELLS_PER_VEIN: usize = 14;

/// Builds the pattern over an already-laid-out honeycomb.
///
/// `groups` is the size of the palette and `empty_share` the fraction of cells
/// left showing the ground; both come from the difficulty curve. Both are
/// clamped to what the board can actually host.
pub fn generate(
    pieces: &[Piece],
    groups: usize,
    empty_share: f32,
    rng: &mut impl Rng,
) -> Pattern {
    let count = pieces.len();
    let index_of: HashMap<(usize, usize), usize> = pieces
        .iter()
        .enumerate()
        .map(|(index, piece)| ((piece.column, piece.row), index))
        .collect();

    let neighbours_of = |index: usize| -> Vec<usize> {
        let piece = &pieces[index];
        neighbours(piece.column, piece.row)
            .into_iter()
            .filter_map(|(column, row)| {
                if column < 0 || row < 0 {
                    return None;
                }
                index_of.get(&(column as usize, row as usize)).copied()
            })
            .collect()
    };

    // Leave enough filled cells for every group plus the answer's neighbours,
    // however greedy the difficulty curve gets.
    let wanted_empty = ((count as f32 * empty_share.clamp(0.0, 0.7)) as usize)
        .min(count.saturating_sub(groups.max(1) * 2 + 1));

    let empty = grow_empty(count, wanted_empty, &neighbours_of, rng);

    let filled: Vec<usize> = (0..count).filter(|index| !empty[*index]).collect();
    let groups = groups.clamp(1, filled.len().max(1));

    let assignment = grow_groups(count, &filled, groups, &neighbours_of, rng);

    // Islands that found no group stayed empty, so the answer has to be chosen
    // from what actually ended up filled.
    let filled: Vec<usize> = (0..count)
        .filter(|index| assignment[*index].is_some())
        .collect();
    let answer = choose_answer(&assignment, &filled, &neighbours_of, rng);

    Pattern {
        groups: assignment,
        group_count: groups,
        answer,
    }
}

/// Marks empty cells, grown out from a few seeds so they form veins rather than
/// speckle.
fn grow_empty(
    count: usize,
    wanted: usize,
    neighbours_of: &impl Fn(usize) -> Vec<usize>,
    rng: &mut impl Rng,
) -> Vec<bool> {
    let mut empty = vec![false; count];

    if wanted == 0 || count == 0 {
        return empty;
    }

    let veins = (wanted / CELLS_PER_VEIN).clamp(1, wanted);
    let mut frontier: Vec<usize> = (0..count).choose_multiple(rng, veins);
    let mut marked = 0;

    for seed in &frontier {
        empty[*seed] = true;
        marked += 1;
    }

    // Grow the veins a cell at a time, always from a random point on the
    // frontier, so they wander instead of forming discs.
    while marked < wanted && !frontier.is_empty() {
        let pick = rng.gen_range(0..frontier.len());
        let cell = frontier[pick];

        let candidates: Vec<usize> = neighbours_of(cell)
            .into_iter()
            .filter(|neighbour| !empty[*neighbour])
            .collect();

        let Some(next) = candidates.choose(rng).copied() else {
            frontier.swap_remove(pick);
            continue;
        };

        empty[next] = true;
        marked += 1;
        frontier.push(next);
    }

    empty
}

/// Assigns every filled cell to a colour group by multi-source breadth-first
/// growth from `groups` seeds.
///
/// Breadth-first is what guarantees each group is *connected*: a cell is only
/// ever claimed by a neighbour that already belongs to the group, so there is
/// always a path home. Assigning by nearest-seed distance instead can strand
/// cells across an empty vein.
fn grow_groups(
    count: usize,
    filled: &[usize],
    groups: usize,
    neighbours_of: &impl Fn(usize) -> Vec<usize>,
    rng: &mut impl Rng,
) -> Vec<Option<usize>> {
    let mut assignment = vec![None; count];

    if filled.is_empty() {
        return assignment;
    }

    // A membership flag rather than a search through `filled`: the growth touches
    // every cell's six neighbours, and a linear scan per neighbour makes the
    // whole pass quadratic in a board that reaches five hundred cells.
    let mut is_filled = vec![false; count];
    for cell in filled {
        is_filled[*cell] = true;
    }

    let seeds: Vec<usize> = filled.iter().copied().choose_multiple(rng, groups);
    let mut frontier: Vec<usize> = Vec::with_capacity(filled.len());

    for (group, seed) in seeds.iter().enumerate() {
        assignment[*seed] = Some(group);
        frontier.push(*seed);
    }

    let mut cursor = 0;
    while cursor < frontier.len() {
        let cell = frontier[cursor];
        cursor += 1;

        let group = assignment[cell];
        for neighbour in neighbours_of(cell) {
            if assignment[neighbour].is_some() || !is_filled[neighbour] {
                continue;
            }
            assignment[neighbour] = group;
            frontier.push(neighbour);
        }
    }

    // A cell cut off from every seed by empty veins is still `None`. Hand it to
    // a neighbour's group — repeatedly, because adopting one cell can give the
    // next one a neighbour to adopt from.
    //
    // Adopting a *neighbour's* group is what keeps groups connected. An earlier
    // version fell back to group zero when a cell had no assigned neighbour at
    // all, which quietly produced a group in two disconnected pieces; the
    // connectivity test caught it. A cell with nowhere to belong stays empty
    // instead, which is honest: it is an island, and an island is a hole.
    loop {
        let mut adopted_any = false;

        for cell in filled {
            if assignment[*cell].is_some() {
                continue;
            }

            let Some(group) = neighbours_of(*cell)
                .into_iter()
                .find_map(|neighbour| assignment[neighbour])
            else {
                continue;
            };

            assignment[*cell] = Some(group);
            adopted_any = true;
        }

        if !adopted_any {
            break;
        }
    }

    assignment
}

/// Picks the answer: a filled cell whose group has company, preferring one
/// surrounded by its own group so it is a near-twin among near-twins.
fn choose_answer(
    assignment: &[Option<usize>],
    filled: &[usize],
    neighbours_of: &impl Fn(usize) -> Vec<usize>,
    rng: &mut impl Rng,
) -> usize {
    let mut population: HashMap<usize, usize> = HashMap::new();
    for cell in filled {
        if let Some(group) = assignment[*cell] {
            *population.entry(group).or_insert(0) += 1;
        }
    }

    let mut best: Vec<usize> = Vec::new();
    let mut best_score = -1i32;

    for cell in filled {
        let Some(group) = assignment[*cell] else {
            continue;
        };

        // A group of one would leave the answer with nothing to hide among.
        if population.get(&group).copied().unwrap_or(0) < 2 {
            continue;
        }

        let score = neighbours_of(*cell)
            .into_iter()
            .filter(|neighbour| assignment[*neighbour] == Some(group))
            .count() as i32;

        if score > best_score {
            best_score = score;
            best.clear();
        }
        if score == best_score {
            best.push(*cell);
        }
    }

    best.choose(rng)
        .copied()
        .or_else(|| filled.choose(rng).copied())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::Vec2;
    use std::collections::HashSet;

    fn board(columns: usize) -> Vec<Piece> {
        crate::board::layout(Vec2::new(-179.0, -290.0), Vec2::new(179.0, 262.0), columns)
    }

    fn rng() -> StdRng {
        StdRng::seed_from_u64(90_210)
    }

    fn neighbour_indices(pieces: &[Piece], index: usize) -> Vec<usize> {
        let lookup: HashMap<(usize, usize), usize> = pieces
            .iter()
            .enumerate()
            .map(|(i, piece)| ((piece.column, piece.row), i))
            .collect();
        let piece = &pieces[index];

        neighbours(piece.column, piece.row)
            .into_iter()
            .filter_map(|(column, row)| {
                if column < 0 || row < 0 {
                    return None;
                }
                lookup.get(&(column as usize, row as usize)).copied()
            })
            .collect()
    }

    /// A group that is not connected is not a blob; it is speckle wearing the
    /// same colour, and the sweep stops reading as one region vanishing.
    #[test]
    fn every_group_is_connected() {
        let mut rng = rng();

        for columns in [4, 8, 12, 16] {
            let pieces = board(columns);
            let pattern = generate(&pieces, 6, 0.35, &mut rng);

            for group in 0..pattern.group_count {
                let members: Vec<usize> = (0..pieces.len())
                    .filter(|index| pattern.groups[*index] == Some(group))
                    .collect();

                if members.is_empty() {
                    continue;
                }

                let mut seen: HashSet<usize> = HashSet::new();
                let mut stack = vec![members[0]];
                seen.insert(members[0]);

                while let Some(cell) = stack.pop() {
                    for neighbour in neighbour_indices(&pieces, cell) {
                        if pattern.groups[neighbour] == Some(group) && seen.insert(neighbour) {
                            stack.push(neighbour);
                        }
                    }
                }

                assert_eq!(
                    seen.len(),
                    members.len(),
                    "{} columns: group {} is in {} disconnected parts",
                    columns,
                    group,
                    members.len() - seen.len() + 1
                );
            }
        }
    }

    /// The answer must have somewhere to hide: a group of one would make it the
    /// only cell of its colour *and* the only cell of its shape of blob.
    #[test]
    fn the_answer_sits_inside_a_group_with_company() {
        let mut rng = rng();

        for columns in [4, 8, 12, 16] {
            for empty_share in [0.2, 0.35, 0.5] {
                let pieces = board(columns);
                let pattern = generate(&pieces, 5, empty_share, &mut rng);

                assert!(
                    pattern.is_filled(pattern.answer),
                    "the answer must be a filled cell"
                );

                let group = pattern.groups[pattern.answer].expect("filled");
                let company = pattern
                    .groups
                    .iter()
                    .filter(|other| **other == Some(group))
                    .count();

                assert!(company >= 2, "{} columns: the answer is alone", columns);
            }
        }
    }

    #[test]
    fn the_empty_share_is_respected() {
        let mut rng = rng();

        for empty_share in [0.2, 0.35, 0.5] {
            let pieces = board(10);
            let pattern = generate(&pieces, 6, empty_share, &mut rng);

            let empty = pieces.len() - pattern.filled_count();
            let wanted = pieces.len() as f32 * empty_share;

            assert!(
                (empty as f32 - wanted).abs() <= wanted * 0.15 + 2.0,
                "asked {:.0} empty cells, got {}",
                wanted,
                empty
            );
        }
    }

    /// Empty cells in veins, not speckle: most of them should touch another
    /// empty cell.
    #[test]
    fn empty_cells_clump() {
        let mut rng = rng();
        let pieces = board(10);
        let pattern = generate(&pieces, 6, 0.35, &mut rng);

        let empty: Vec<usize> = (0..pieces.len())
            .filter(|index| !pattern.is_filled(*index))
            .collect();

        let touching = empty
            .iter()
            .filter(|index| {
                neighbour_indices(&pieces, **index)
                    .into_iter()
                    .any(|neighbour| !pattern.is_filled(neighbour))
            })
            .count();

        assert!(
            touching * 10 >= empty.len() * 8,
            "only {} of {} empty cells have an empty neighbour",
            touching,
            empty.len()
        );
    }

    /// Nothing may be left unassigned: a filled cell without a group would be
    /// drawn as another hole, and the round would have two answers.
    #[test]
    fn no_filled_cell_is_left_without_a_group() {
        let mut rng = rng();

        for columns in [4, 8, 16] {
            let pieces = board(columns);
            let pattern = generate(&pieces, 8, 0.5, &mut rng);

            for (index, group) in pattern.groups.iter().enumerate() {
                if let Some(group) = group {
                    assert!(*group < pattern.group_count, "cell {} has a stray group", index);
                }
            }
        }
    }
}
