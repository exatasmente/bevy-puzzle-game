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

    let mut empty = grow_empty(count, wanted_empty, &neighbours_of, rng);
    dissolve_slivers(count, &mut empty, &neighbours_of, rng);

    let filled: Vec<usize> = (0..count).filter(|index| !empty[*index]).collect();
    let groups = groups.clamp(1, filled.len().max(1));

    let assignment = grow_groups(count, &filled, groups, &neighbours_of, rng);

    // More groups than the curve asked for when the veins cut the board into
    // more regions than there are colours — every region needs a seed of its
    // own or it is not drawn at all. The palette is built from this number, so
    // it has to be what was actually used rather than what was requested.
    let group_count = assignment
        .iter()
        .flatten()
        .copied()
        .max()
        .map(|highest| highest + 1)
        .unwrap_or(0);

    // Islands that found no group stayed empty, so the answer has to be chosen
    // from what actually ended up filled.
    let filled: Vec<usize> = (0..count)
        .filter(|index| assignment[*index].is_some())
        .collect();
    let answer = choose_answer(&assignment, &filled, &neighbours_of, rng);

    Pattern {
        groups: assignment,
        group_count,
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

/// A region below this many cells cannot host a blob, and is dissolved.
const MIN_REGION: usize = 3;

/// Removes filled regions too small to read as a blob, keeping the number of
/// empty cells exactly as the level asked for.
///
/// A veined board occasionally strands one or two cells on their own, and
/// because every region gets a seed of its own, a stranded cell comes out in a
/// colour no other cell wears. That is precisely the description of the answer.
/// The player looking for the cell whose colour does not match its surroundings
/// finds it immediately, taps it, and is told they are wrong — for spotting
/// exactly what the round asked them to spot.
///
/// So the slivers are emptied, and an equal number of empty cells touching the
/// largest region are filled back in to keep the share honest. Filling only next
/// to an existing region is what stops the repair from stranding something new.
fn dissolve_slivers(
    count: usize,
    empty: &mut [bool],
    neighbours_of: &impl Fn(usize) -> Vec<usize>,
    rng: &mut impl Rng,
) {
    let filled: Vec<usize> = (0..count).filter(|index| !empty[*index]).collect();
    let regions = components(count, &filled, neighbours_of);

    // Never dissolve the last region standing, however small the board is.
    let mut owed = 0;
    for region in regions.iter().skip(1) {
        if region.len() >= MIN_REGION {
            continue;
        }
        for cell in region {
            empty[*cell] = true;
            owed += 1;
        }
    }

    let Some(main) = regions.first() else {
        return;
    };

    // Grow the main region back out by as many cells as were taken.
    let mut edge: Vec<usize> = main
        .iter()
        .flat_map(|cell| neighbours_of(*cell))
        .filter(|cell| empty[*cell])
        .collect();
    edge.sort_unstable();
    edge.dedup();

    while owed > 0 && !edge.is_empty() {
        let pick = rng.gen_range(0..edge.len());
        let cell = edge.swap_remove(pick);

        if !empty[cell] {
            continue;
        }

        empty[cell] = false;
        owed -= 1;
        edge.extend(neighbours_of(cell).into_iter().filter(|next| empty[*next]));
    }
}

/// Splits the filled cells into connected regions, largest first.
///
/// The empty veins routinely cut the board in two or more, and how the seeds are
/// spread over those pieces decides whether the board comes out looking like the
/// level asked for.
fn components(
    count: usize,
    filled: &[usize],
    neighbours_of: &impl Fn(usize) -> Vec<usize>,
) -> Vec<Vec<usize>> {
    let mut is_filled = vec![false; count];
    for cell in filled {
        is_filled[*cell] = true;
    }

    let mut seen = vec![false; count];
    let mut found: Vec<Vec<usize>> = Vec::new();

    for start in filled {
        if seen[*start] {
            continue;
        }

        let mut region = vec![*start];
        seen[*start] = true;
        let mut cursor = 0;

        while cursor < region.len() {
            let cell = region[cursor];
            cursor += 1;

            for neighbour in neighbours_of(cell) {
                if is_filled[neighbour] && !seen[neighbour] {
                    seen[neighbour] = true;
                    region.push(neighbour);
                }
            }
        }

        found.push(region);
    }

    found.sort_by_key(|region| std::cmp::Reverse(region.len()));
    found
}

/// Assigns every filled cell to a colour group by multi-source breadth-first
/// growth from `groups` seeds.
///
/// Breadth-first is what guarantees each group is *connected*: a cell is only
/// ever claimed by a neighbour that already belongs to the group, so there is
/// always a path home. Assigning by nearest-seed distance instead can strand
/// cells across an empty vein.
///
/// **The seeds are dealt per region, not drawn at random from the whole board.**
/// Drawn at random, a region the veins had cut off could easily receive no seed
/// at all, and every cell in it would stay unassigned — which is to say it would
/// be drawn as more empty ground. That is how a level asking for 20% empty
/// produced boards that were 46% empty, and at 50% produced boards that were
/// 82% empty: half the screen bare, the blobs gone, and the answer with nothing
/// left to hide among. Every region now gets at least one seed, and the rest are
/// dealt out largest-region-first so the big areas still get most of the colours.
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

    let regions = components(count, filled, neighbours_of);

    // One seed each, then hand out what is left in proportion to size. When
    // there are more regions than colours the smallest ones share, which is
    // fine: they are far apart on the screen and read as separate blobs anyway.
    let mut per_region = vec![1usize; regions.len()];
    let mut spare = groups.saturating_sub(regions.len());
    let cells: usize = regions.iter().map(|region| region.len()).sum();

    for (index, region) in regions.iter().enumerate() {
        if spare == 0 {
            break;
        }
        let share = ((groups * region.len()) / cells.max(1)).saturating_sub(1);
        let extra = share.min(spare);
        per_region[index] += extra;
        spare -= extra;
    }
    // Rounding leaves a seed or two over; the biggest region takes them.
    per_region[0] += spare;

    let mut seeds: Vec<usize> = Vec::with_capacity(groups);
    for (index, region) in regions.iter().enumerate() {
        // A region can only host as many blobs as it has cells to spare. Three
        // seeds in a three-cell region gives three groups of one, which is the
        // decoy `dissolve_slivers` exists to prevent, rebuilt from the inside.
        let room = (region.len() / MIN_REGION).max(1);
        seeds.extend(
            region
                .iter()
                .copied()
                .choose_multiple(rng, per_region[index].min(room)),
        );
    }

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

    merge_lone_groups(&mut assignment, filled, neighbours_of);
    compact_groups(&mut assignment);
    assignment
}

/// Folds any group that came out a single cell into a neighbouring group.
///
/// Capping the seeds by region makes this rare rather than impossible: the
/// growth is breadth-first from all seeds at once, so a seed dropped next to
/// another can still be walled in before it claims anything. Merging into a
/// *neighbour* keeps the receiving group connected, which is the invariant the
/// whole assignment is built around.
fn merge_lone_groups(
    assignment: &mut [Option<usize>],
    filled: &[usize],
    neighbours_of: &impl Fn(usize) -> Vec<usize>,
) {
    loop {
        let mut population: HashMap<usize, usize> = HashMap::new();
        for group in assignment.iter().flatten() {
            *population.entry(*group).or_insert(0) += 1;
        }

        let Some(&lonely) = filled.iter().find(|cell| {
            assignment[**cell].map_or(false, |group| population[&group] == 1)
        }) else {
            return;
        };

        let own = assignment[lonely];
        let Some(host) = neighbours_of(lonely)
            .into_iter()
            .find_map(|neighbour| assignment[neighbour].filter(|group| Some(*group) != own))
        else {
            // Nothing to join. Better a hole than a colour worn by one cell,
            // which is what the answer looks like.
            assignment[lonely] = None;
            continue;
        };

        assignment[lonely] = Some(host);
    }
}

/// Renumbers the groups so the ids run 0..n with nothing missing.
///
/// Merging leaves gaps, and the palette is built by indexing this number: a gap
/// would put a colour in the round's sweep that no cell on the board wears, so
/// the ground would stop somewhere and nothing would vanish.
fn compact_groups(assignment: &mut [Option<usize>]) {
    let mut renumbered: HashMap<usize, usize> = HashMap::new();

    for slot in assignment.iter_mut() {
        let Some(group) = *slot else {
            continue;
        };
        let next = renumbered.len();
        *slot = Some(*renumbered.entry(group).or_insert(next));
    }
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

    /// The level asks for a share of empty cells and has to get it — every
    /// round, not on average.
    ///
    /// This was the loosest test here, and the slack hid a real bug. Seeds were
    /// drawn at random from the whole board, so a region the empty veins had cut
    /// off could receive none, and every cell in it went unassigned — drawn as
    /// yet more bare ground. The mean stayed on target, which is why a tolerant
    /// test passed, but individual boards came out 46% empty at a level asking
    /// for 20%, and 82% empty at one asking for 50%: the blobs gone, and the
    /// answer with nothing left to hide among. Seeding per region fixed it, and
    /// the tolerance is now one cell — the rounding in the share itself.
    #[test]
    fn the_empty_share_is_respected() {
        const GROUPS: usize = 6;
        let mut rng = rng();

        for columns in [4, 8, 12, 16] {
            for empty_share in [0.2, 0.35, 0.5] {
                let pieces = board(columns);

                for _ in 0..50 {
                    let pattern = generate(&pieces, GROUPS, empty_share, &mut rng);

                    let empty = pieces.len() - pattern.filled_count();
                    // The same ceiling `generate` applies: a small board cannot
                    // give the curve everything it asks for and still leave the
                    // groups somewhere to live.
                    let wanted = ((pieces.len() as f32 * empty_share) as usize)
                        .min(pieces.len() - (GROUPS * 2 + 1));

                    assert!(
                        empty.abs_diff(wanted) <= 1,
                        "{} columns at {}: asked {} empty cells, got {}",
                        columns,
                        empty_share,
                        wanted,
                        empty
                    );
                }
            }
        }
    }

    /// Empty cells in veins, not speckle: most of them should touch another
    /// empty cell.
    /// No group may be a single cell.
    ///
    /// A lone hexagon in a colour nothing else wears is the description of the
    /// answer, so the player who spots it is being punished for playing the
    /// round correctly. Every group has to be big enough to read as a blob.
    #[test]
    fn no_group_is_a_lone_cell() {
        let mut rng = rng();

        for columns in [4, 8, 12, 16] {
            for empty_share in [0.2, 0.35, 0.5] {
                let pieces = board(columns);

                for _ in 0..50 {
                    let pattern = generate(&pieces, 8, empty_share, &mut rng);

                    let mut population = HashMap::new();
                    for group in pattern.groups.iter().flatten() {
                        *population.entry(*group).or_insert(0usize) += 1;
                    }

                    for (group, cells) in population {
                        assert!(
                            cells >= 2,
                            "{} columns at {}: group {} is one cell on its own",
                            columns,
                            empty_share,
                            group
                        );
                    }
                }
            }
        }
    }

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


