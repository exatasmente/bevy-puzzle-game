use bevy::prelude::*;
use std::time::Duration;
use rand::prelude::*;

use crate::board::{self, Piece};
use crate::mosaic_pattern;
use crate::oklab::{self, Oklab};
use crate::theme;
use crate::wfc::{self, Tile};

#[derive(Component)]
pub struct PuzzleColor {
    pub index : usize,
    pub is_correct_color : bool,
    pub color : Color,
    pub x : f32,
    pub y : f32,
    /// The piece's outline, relative to its centre at `x`/`y`. Carried per
    /// piece because no two pieces share a shape, and so a replayed round is
    /// drawn and hit-tested exactly as it was played.
    pub corners : Vec<Vec2>,
    /// The piece drawn on this cell in `Mosaic`, and `None` in every other
    /// mode. Stored per cell for the same reason `size` is: a replayed round
    /// has to redraw the board that was actually played.
    pub tile : Option<Tile>,
}

impl PuzzleColor {
    pub fn as_level_color(&self) -> LevelColor {
        LevelColor {
            color : self.color,
            x : self.x,
            y : self.y,
            is_correct_color : self.is_correct_color,
            corners : self.corners.clone(),
            tile : self.tile,
        }
    }

    /// Whether a world-space point lands on this piece.
    pub fn contains(&self, point : Vec2) -> bool {
        board::contains(Vec2::new(self.x, self.y), &self.corners, point)
    }
}


pub struct RenderLevelHistoryEvent {
    pub index: usize,
}

pub struct NewGameEvent {
    pub game_mode: GameMode,
}


pub struct StartLevelEvent;

#[derive(Debug, Reflect, PartialEq, Eq, Clone, Copy)]
pub enum GameMode {
    Infinite,
    AgainstTheClock,
    TimeTrial,
    /// The board is shown, then goes blank, and the pick is made from memory.
    Memory,
    /// A tiled pattern with one piece that does not fit its neighbours.
    Mosaic,
}

impl GameMode {
    pub fn iter() -> impl Iterator<Item = GameMode> {
        [
            GameMode::Infinite,
            GameMode::AgainstTheClock,
            GameMode::TimeTrial,
            GameMode::Memory,
            GameMode::Mosaic,
        ]
        .iter()
        .copied()
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            GameMode::Infinite => "Infinito",
            GameMode::AgainstTheClock => "Contra o Tempo",
            GameMode::TimeTrial => "Soma de Tempo",
            GameMode::Memory => "Memoria",
            GameMode::Mosaic => "Mosaico",
        }
    }

    /// One line telling the player what they are choosing, so the mode select
    /// is an informed choice rather than four unlabelled doors.
    pub fn description(&self) -> &'static str {
        match self {
            // Kept short on purpose: the card gives a description about 25
            // characters of room before the type has to shrink past reading
            // size. Say the one thing that distinguishes the mode.
            GameMode::Infinite => "3 vidas. No seu ritmo.",
            GameMode::AgainstTheClock => "60s. Cada erro custa 3s.",
            GameMode::TimeTrial => "30s. +3s certo, -2s erro.",
            GameMode::Memory => "As cores somem. 3 vidas.",
            GameMode::Mosaic => "A peca que nao encaixa.",
        }
    }

    /// The mode's identity color, used for its marker on the menu.
    pub fn accent(&self) -> Color {
        match self {
            GameMode::Infinite => theme::PRIMARY,
            GameMode::AgainstTheClock => theme::SUCCESS,
            GameMode::TimeTrial => theme::LIME,
            GameMode::Memory => theme::INFO,
            GameMode::Mosaic => theme::PINK,
        }
    }

    /// Stable key for persisted best scores. Never change these strings without
    /// migrating stored values.
    pub fn storage_key(&self) -> &'static str {
        match self {
            GameMode::Infinite => "infinite",
            GameMode::AgainstTheClock => "against_the_clock",
            GameMode::TimeTrial => "time_trial",
            GameMode::Memory => "memory",
            GameMode::Mosaic => "mosaic",
        }
    }

    /// Whether a run in this mode can ever end on its own.
    ///
    /// Every untimed mode has to be listed here. A mode left out gets a
    /// zero-length timer from `setup`, which reads as finished on its first
    /// frame and ends the run before the player sees the board.
    pub fn is_timed(&self) -> bool {
        !matches!(
            self,
            GameMode::Infinite | GameMode::Memory | GameMode::Mosaic
        )
    }

    /// Whether the board blanks out before the pick.
    pub fn hides_colors(&self) -> bool {
        matches!(self, GameMode::Memory)
    }

    /// Whether the round is a tiled pattern rather than a field of colors.
    pub fn is_mosaic(&self) -> bool {
        matches!(self, GameMode::Mosaic)
    }

    /// How long a missed board stays up before the next round.
    ///
    /// Shorter when there is a clock, because the hold is charged twice there:
    /// once in the point, once in the seconds it eats.
    pub fn hold_seconds(&self) -> f32 {
        if self.is_timed() {
            0.45
        } else {
            0.7
        }
    }

    /// How many lives a run in this mode starts with, or `None` when the mode
    /// runs on a clock instead.
    ///
    /// Deliberately one or the other, never both: two resources that can each
    /// end a run means the player has to watch two things at once, and neither
    /// reads clearly. So the untimed modes — which until now could only end by
    /// the player pressing "encerrar partida" — get lives, and the timed ones
    /// charge a miss in seconds (`miss_penalty_seconds`).
    pub fn starting_lives(&self) -> Option<usize> {
        if self.is_timed() {
            None
        } else {
            Some(3)
        }
    }

    /// What a wrong pick costs in a timed mode.
    ///
    /// Missing used to be free everywhere: the board paused, the clock paused
    /// with it, and nothing at all was subtracted. Guessing was therefore
    /// strictly better than looking, which is the wrong lesson for a game about
    /// looking carefully.
    pub fn miss_penalty_seconds(&self) -> f32 {
        match self {
            GameMode::AgainstTheClock => 3.0,
            // Lighter, because a miss here already costs the three seconds the
            // pick would have earned — the mode charges twice on its own.
            GameMode::TimeTrial => 2.0,
            _ => 0.0,
        }
    }
}

/// Levels between one free life and the next.
///
/// Three lives and no way back turns `Infinite` into a short sprint, which is
/// the opposite of the thing it is named for. Every fifth level hands one back,
/// never above the mode's maximum — so a careful player can hold three
/// indefinitely and a reckless one still bleeds out.
pub const LEVELS_PER_EXTRA_LIFE: usize = 5;


#[derive(Resource, Debug, Reflect)]
pub struct ColorPuzzle {
    score: usize,
    /// Wrong picks left before the run ends. Always zero in a timed mode, which
    /// spends seconds on a miss instead — see `GameMode::starting_lives`.
    lives: usize,
    current_colors: Vec<Color>,
    /// The color all but one square share this round.
    base_color: Color,
    /// One piece per cell in `Mosaic`, empty in every other mode.
    #[reflect(ignore)]
    current_tiles: Vec<Tile>,
    /// Where this round's pieces sit and what shape they are. Empty in
    /// `Mosaic`, which is laid out on a grid instead.
    #[reflect(ignore)]
    current_slots: Vec<Piece>,
    /// Columns the mosaic was generated on. Only meaningful with `current_tiles`.
    current_columns: usize,
    /// The round's distinct colours, which the ground sweeps through before it
    /// settles on the answer's.
    current_palette: Vec<Color>,
    correct_color_index: usize,
    pub game_mode: GameMode,
    pub seconds_added_per_success: f32,
    pub shape_size: f32,
    pub start_seconds: f32,
    pub transition_seconds: f32,
    pub width: f32,
    pub height: f32,
}


impl Default for ColorPuzzle {
    
    fn default() -> Self {
        Self::new()
    }
}

/// Whether two colors are the same paint.
pub fn colors_match(a: Color, b: Color) -> bool {
    (a.r() - b.r()).abs() < 1e-4 && (a.g() - b.g()).abs() < 1e-4 && (a.b() - b.b()).abs() < 1e-4
}

// --- The difficulty curve --------------------------------------------------
//
// There is no level table any more, and no last level. The old table was
// exactly `2 + 3L(L-1)/2` from level two on, so the curve it described is kept
// and simply continues: five points to reach level two, then three more points
// per level, forever. Stored runs keep the level they had.
//
// Every dial below is a monotone function of the level with an asymptote. Being
// honest about the asymptote matters: below roughly 0.008 in Oklab a colour
// difference is a coin flip rather than a challenge, so the difficulty plateaus
// even though the levels keep counting.

/// Score at which `level` begins.
pub fn score_for_level(level: usize) -> usize {
    if level <= 1 {
        return 0;
    }

    // In `u64` because `usize` is 32 bits on wasm32, where the plain form
    // overflows somewhere past level fifty thousand.
    let level = level as u64;
    let start = 2u64.saturating_add(3u64.saturating_mul(level).saturating_mul(level - 1) / 2);

    start.min(usize::MAX as u64) as usize
}

/// 1-based level for a score: the inverse of [`score_for_level`].
pub fn level_for_score(score: usize) -> usize {
    if score < 5 {
        return 1;
    }

    // Solving `2 + 3L(L-1)/2 <= score`. `f64`, not `f32`: `f32` stops
    // representing consecutive integers around 1.6e7, well inside a reachable
    // score. The two corrections then make the boundary exact rather than
    // merely convincing.
    let estimate = (3.0 + (24.0 * score as f64 - 30.0).sqrt()) / 6.0;
    let mut level = estimate.floor().max(1.0) as usize;

    while score_for_level(level + 1) <= score {
        level += 1;
    }
    while level > 1 && score_for_level(level) > score {
        level -= 1;
    }

    level
}

/// Columns of hexagons on the board.
///
/// Stops at sixteen: the honeycomb fills the screen, so more columns only means
/// smaller cells, and past this they stop being worth aiming at.
pub fn columns_for_level(level: usize) -> usize {
    let steps = level.saturating_sub(1) as f32;
    let span = (board::MAX_COLUMNS - board::MIN_COLUMNS) as f32;

    board::MIN_COLUMNS + (span * (1.0 - (-steps / 9.0).exp())).round() as usize
}

/// Share of cells left empty, showing the ground.
///
/// This is the dial that keeps working after the board stops growing: every
/// empty cell is another hole for the answer to hide among once the sweep has
/// finished.
pub fn empty_share_for_level(level: usize) -> f32 {
    let steps = level.saturating_sub(1) as f32;
    0.50 - 0.30 * (-steps / 9.0).exp()
}

/// How many colours the round's mosaic is built from.
///
/// Small on purpose. The background sweeps colours, not cells, so the palette
/// size is how many steps that sweep has — a board of three hundred distinct
/// colours would flicker through them in two milliseconds each and show nothing.
pub fn palette_size_for_level(level: usize) -> usize {
    let steps = level.saturating_sub(1) as f32;
    4 + (4.0 * (1.0 - (-steps / 12.0).exp())).round() as usize
}

/// Perceptual distance between the answer and the group it hides in, in Oklab
/// units: about 0.02 is subtle, 0.01 is hard, below 0.005 is a coin flip.
///
/// Because the unit is perceptual, a level means the same thing whether the
/// round came out olive or navy — which was not true of the per-channel sRGB
/// variation this replaced.
pub const MIN_COLOR_DELTA: f32 = 0.010;
pub fn color_delta_for_level(level: usize) -> f32 {
    let steps = level.saturating_sub(1) as f32;
    // Starts at 0.050 rather than the 0.080 the old cluster model used. The
    // answer now lives inside a colour group with other groups nearby, and a
    // delta that large would carry it across the gap into the next group's
    // colour — at which point the ground settling would erase two things.
    MIN_COLOR_DELTA + 0.040 * (-steps / 6.0).exp()
}

/// How long the board stays visible in `Memory` before it blanks, by level.
///
/// Shrinks with the level so the mode gets harder in the dimension it is about
/// — how much you can hold — rather than only in color distance.
pub fn preview_seconds_for_level(level: usize) -> f32 {
    let steps = (level.saturating_sub(1)) as f32;
    (1.7 - steps * 0.12).max(0.7)
}

/// Grid a `Mosaic` round is played on, by level.
///
/// Fixed dimensions rather than a count, because the generator reasons about
/// neighbours: it needs to know the shape of the board, and a ragged last row
/// would leave pieces with nothing to disagree with.
pub fn mosaic_dimensions_for_level(level: usize) -> (usize, usize) {
    match level {
        1 => (2, 3),
        2 => (3, 3),
        3 | 4 => (3, 4),
        5 | 6 => (4, 4),
        7 | 8 => (4, 5),
        _ => (5, 5),
    }
}

/// How many of the odd piece's four edges disagree with their surroundings.
///
/// Only four and two are used, and that is a constraint of the tile set rather
/// than a preference. Three edges away from a piece with two arms or fewer is
/// always a three-armed piece, and one edge away from a legal piece — against
/// the board's outer edge, the only place a lone violation is fair — is also
/// always a three-armed piece. So those settings made the impostor a T every
/// single time, which is a tell the player learns in two rounds and never
/// unlearns. Two gives all four shapes; four gives the two-armed ones.
///
/// The difficulty past the first level therefore rides on the size of the
/// board, not on the number of broken edges.
pub fn mosaic_violations_for_level(level: usize) -> usize {
    if level <= 1 {
        4
    } else {
        2
    }
}

// --- Board grid ------------------------------------------------------------
//
// The board used to be squares dropped at random positions with rejection
// sampling. That made every round a different search problem: the eye had to
// find the squares before it could compare them, difficulty swung with whatever
// the sampler happened to produce, and squares could land under the HUD. A grid
// puts the comparison — which is the actual game — in front of the player, and
// makes the difficulty curve mean what the level table says it means.

/// Vertical strip at the top of the window owned by the HUD. The board is laid
/// out below it, so no square is ever hidden behind the score or the pause
/// button.
pub const HUD_RESERVED_HEIGHT: f32 = 132.0;

/// Gap between neighbouring cells. Big enough to read as separate squares,
/// small enough that adjacent colors can still be compared edge to edge.
pub const BOARD_GAP: f32 = 10.0;

/// The board never touches the window edge.
pub const BOARD_MARGIN: f32 = 16.0;

/// Cells stop growing here, so a four-square round on a desktop window does not
/// turn into four billboards.
pub const MAX_CELL_SIZE: f32 = 160.0;

/// Where every square of a round goes.
#[derive(Debug, Clone, Copy)]
pub struct BoardGrid {
    pub columns: usize,
    pub rows: usize,
    pub count: usize,
    pub cell_size: f32,
    /// World position of the bottom-left corner of the bottom-left cell.
    pub origin: Vec2,
}

impl BoardGrid {
    /// How many squares sit on `row` (0 = top). Only the last row can be short.
    fn items_in_row(&self, row: usize) -> usize {
        let remaining = self.count.saturating_sub(row * self.columns);
        remaining.min(self.columns)
    }

    /// Bottom-left corner of the square at `index`, filling left to right and
    /// top to bottom.
    pub fn cell_position(&self, index: usize) -> Vec2 {
        let column = index % self.columns;
        let row = index / self.columns;
        let step = self.cell_size + BOARD_GAP;

        // A short last row is centered under the others; left-aligning it makes
        // the whole board look accidentally off-center.
        let row_width = self.items_in_row(row) as f32 * self.cell_size
            + BOARD_GAP * self.items_in_row(row).saturating_sub(1) as f32;
        let full_width = self.columns as f32 * self.cell_size
            + BOARD_GAP * self.columns.saturating_sub(1) as f32;
        let row_offset = (full_width - row_width) / 2.0;

        Vec2::new(
            self.origin.x + row_offset + column as f32 * step,
            // Rows are counted from the top, but y grows upward.
            self.origin.y + (self.rows.saturating_sub(1) - row) as f32 * step,
        )
    }
}


impl ColorPuzzle {
   pub  fn new() -> Self {
        let mut puzzle =  Self {
            score: 0,
            // Seeded by the `setup` call below, which is the only thing that
            // knows the mode.
            lives: 0,
            current_colors: vec![],
            base_color: Color::rgb(0.5, 0.5, 0.5),
            current_tiles: vec![],
            current_slots: vec![],
            current_columns: 0,
            current_palette: vec![],
            correct_color_index: 0,
            game_mode: GameMode::TimeTrial,
            seconds_added_per_success: 3.0,
            shape_size: 200.0,
            start_seconds: 60.0,
            transition_seconds: 1.,
            width: 800.0,
            height: 600.0,
        };

        puzzle.setup(&GameMode::TimeTrial);

        puzzle.generate_colors();

        puzzle
    }

    pub fn setup(&mut self, game_mode: &GameMode) {
        self.reset();

        match game_mode {
            GameMode::Infinite => {
                self.start_seconds = 0.0;
                self.transition_seconds = 1.0;
                self.game_mode = GameMode::Infinite;
            },
            GameMode::AgainstTheClock => {
                self.start_seconds = 60.0;
                self.transition_seconds = 1.0;
                self.game_mode = GameMode::AgainstTheClock;
            },
            GameMode::TimeTrial => {
                self.start_seconds = 30.0;
                self.transition_seconds = 1.0;
                self.seconds_added_per_success = 3.0;
                self.game_mode = GameMode::TimeTrial;
            },
            GameMode::Mosaic => {
                // Untimed, like Memory: reading a pattern is slower than
                // comparing two colors, and a clock would only push the player
                // to guess.
                self.start_seconds = 0.0;
                self.transition_seconds = 0.35;
                self.game_mode = GameMode::Mosaic;
            },
            GameMode::Memory => {
                // No clock: the pressure in this mode is the preview running
                // out, and stacking a run timer on top of it only punishes the
                // player twice for the same thing.
                self.start_seconds = 0.0;
                // The board is hidden a beat after it appears, so the round
                // cannot spend a full second fading the background in first.
                self.transition_seconds = 0.35;
                self.game_mode = GameMode::Memory;
            },
        }

        // Last, because it reads the mode the match above just set. Doing it
        // here rather than at the three places a run begins is what keeps the
        // menu's play button, "jogar novamente" and a resumed run from each
        // having to remember to do it.
        self.lives = self.game_mode.starting_lives().unwrap_or(0);
    }

    pub fn set_window_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;

        // Nothing is laid out yet at this point; the next round is cut against
        // the size just stored.
    }

    /// World y of the bottom of the play area.
    fn play_bottom(&self) -> f32 {
        -self.height / 2.0 + BOARD_MARGIN
    }

    /// Play area: the window minus its margins and the strip the HUD owns.
    fn play_area(&self) -> Vec2 {
        Vec2::new(
            (self.width - BOARD_MARGIN * 2.0).max(1.0),
            (self.height - HUD_RESERVED_HEIGHT - BOARD_MARGIN * 2.0).max(1.0),
        )
    }



    /// Grid with the dimensions fixed by the caller.
    ///
    /// `Mosaic` needs this: its generator reasoned about a particular number of
    /// columns and rows, so the board has to be drawn on exactly those.
    pub fn grid_for_dimensions(&self, columns: usize, rows: usize) -> BoardGrid {
        let columns = columns.max(1);
        let rows = rows.max(1);
        let available = self.play_area();

        let width = (available.x - BOARD_GAP * (columns - 1) as f32) / columns as f32;
        let height = (available.y - BOARD_GAP * (rows - 1) as f32) / rows as f32;

        self.grid_from(columns, rows, columns * rows, width.min(height))
    }

    /// Sizes and centers a grid of known shape.
    fn grid_from(&self, columns: usize, rows: usize, count: usize, cell_size: f32) -> BoardGrid {
        let cell_size = cell_size.min(MAX_CELL_SIZE).max(8.0);

        let grid_width = columns as f32 * cell_size + BOARD_GAP * (columns - 1) as f32;
        let grid_height = rows as f32 * cell_size + BOARD_GAP * (rows - 1) as f32;

        // Center the grid in the play area rather than in the window: the HUD
        // takes its space off the top, so window-centered would sit low.
        let play_top = self.height / 2.0 - HUD_RESERVED_HEIGHT - BOARD_MARGIN;
        let play_bottom = -self.height / 2.0 + BOARD_MARGIN;
        let center_y = (play_top + play_bottom) / 2.0;

        BoardGrid {
            columns,
            rows,
            count,
            cell_size,
            origin: Vec2::new(-grid_width / 2.0, center_y - grid_height / 2.0),
        }
    }

    pub fn get_correct_color_index(&self) -> usize {
        self.correct_color_index
    }


    pub fn generate_colors(&mut self) {
        let mut rng = rand::thread_rng();

        let level = self.level();
        if self.game_mode.is_mosaic() {
            self.generate_mosaic(level, &mut rng);
            return;
        }

        let delta = color_delta_for_level(level);

        let slots = self.cut_board(columns_for_level(level));
        // The mosaic: which cells are empty, which colour group each filled
        // cell belongs to, and which one is the answer.
        let pattern = mosaic_pattern::generate(
            &slots,
            palette_size_for_level(level),
            empty_share_for_level(level),
            &mut rng,
        );

        // The centre of the round, kept off the extremes of lightness so the
        // palette has room to spread in any direction and stay displayable.
        let base_lab = Self::random_base(&mut rng);
        let base_color = oklab::to_color(base_lab).unwrap_or(Color::rgb(0.5, 0.5, 0.5));
        let palette = Self::palette(&mut rng, base_lab, pattern.group_count);

        // Only filled cells become pieces. An empty cell is simply absent —
        // it shows the ground, which is the whole point of it.
        let mut slots_in_play: Vec<Piece> = Vec::with_capacity(pattern.filled_count());
        let mut colors: Vec<Color> = Vec::with_capacity(pattern.filled_count());
        let mut correct = 0;

        for (index, piece) in slots.into_iter().enumerate() {
            let Some(group) = pattern.groups[index] else {
                continue;
            };

            if index == pattern.answer {
                correct = colors.len();
                // The answer wears its group's colour moved by the level's
                // delta: a near-twin of everything around it, and the only cell
                // on the board wearing exactly this colour.
                colors.push(Self::answer_color(&mut rng, &palette, group, delta));
            } else {
                colors.push(palette[group].1);
            }

            slots_in_play.push(piece);
        }

        self.correct_color_index = correct;
        self.base_color = base_color;
        self.current_tiles = vec![];
        self.current_columns = 0;
        self.current_slots = slots_in_play;
        self.current_palette = palette.into_iter().map(|(_, color)| color).collect();
        self.current_colors = colors;
    }

    /// The round's colour groups.
    ///
    /// Built by walking an arc of hue around the round's base rather than by
    /// nudging in random directions and hoping. Random nudges have to be
    /// rejection-sampled against each other, and the separation the round needs
    /// — comfortably more than the answer's delta — is a large distance in
    /// Oklab: most candidates fall outside what a screen can show, the budget
    /// runs out, and every group ends up the fallback colour. That is what
    /// happened here, and the board came out in one flat blue.
    ///
    /// An arc gives the guarantee directly: `groups` hues spread evenly are
    /// separated by construction, and staying near the base's lightness and
    /// chroma keeps them all displayable and looking like one family.
    fn palette(rng: &mut ThreadRng, base: Oklab, groups: usize) -> Vec<(Oklab, Color)> {
        let base_hue = base.b.atan2(base.a);
        let base_chroma = (base.a * base.a + base.b * base.b).sqrt().max(0.06);

        // A little over half the circle: far enough apart to tell one group
        // from the next, close enough that the board reads as one mosaic
        // rather than a paint chart.
        const ARC: f32 = std::f32::consts::PI * 1.15;

        (0..groups)
            .map(|group| {
                let share = if groups <= 1 {
                    0.5
                } else {
                    group as f32 / (groups - 1) as f32
                };

                let hue = base_hue - ARC / 2.0 + ARC * share + rng.gen_range(-0.05..0.05);
                let lightness = (base.l + rng.gen_range(-0.09..0.09)).clamp(0.45, 0.85);
                let mut chroma = (base_chroma * rng.gen_range(0.8..1.15)).clamp(0.05, 0.16);

                // Some hues cannot be as saturated as others at a given
                // lightness; walk the chroma down rather than clamp the colour,
                // which would move it off its hue.
                for _ in 0..10 {
                    let candidate = Oklab::from_lch(lightness, chroma, hue);
                    if let Some(color) = oklab::to_color(candidate) {
                        return (candidate, color);
                    }
                    chroma *= 0.85;
                }

                let flat = Oklab::from_lch(lightness, 0.0, hue);
                (flat, oklab::to_color(flat).unwrap_or(Color::rgb(0.5, 0.5, 0.5)))
            })
            .collect()
    }

    /// The answer's colour: its group's, moved by the level's delta.
    ///
    /// It has to stay clear of every *other* group as well. When the ground
    /// settles on this colour the answer disappears; if another group's colour
    /// were within a delta of it, that whole group would nearly disappear too
    /// and the round would have more than one defensible answer.
    fn answer_color(
        rng: &mut ThreadRng,
        palette: &[(Oklab, Color)],
        group: usize,
        delta: f32,
    ) -> Color {
        let own = palette[group].0;
        let clearance = (delta * 2.0).max(0.03);
        let mut fallback = palette[group].1;

        for _ in 0..48 {
            let Some((lab, color)) = Self::nudge_chromatic(rng, own, delta) else {
                continue;
            };

            fallback = color;

            let clear = palette
                .iter()
                .enumerate()
                .all(|(other, (lab_other, _))| {
                    other == group || Self::distance(lab, *lab_other) > clearance
                });

            if clear {
                return color;
            }
        }

        fallback
    }

    /// Lays this round's honeycomb over the play area.
    fn cut_board(&self, columns: usize) -> Vec<Piece> {
        let area = self.play_area();
        let min = Vec2::new(-area.x / 2.0, self.play_bottom());
        let max = Vec2::new(area.x / 2.0, self.play_bottom() + area.y);

        board::layout(min, max, columns)
    }

    /// This round's pieces, or an empty list in `Mosaic`.
    pub fn slots(&self) -> &[Piece] {
        &self.current_slots
    }

    /// The colours the ground travels through this round, in order, ending on
    /// the answer's.
    ///
    /// This is the round's other channel of information. Every group vanishes
    /// for a moment as the ground passes its colour; the answer is the one that
    /// vanishes at the end and stays gone. Without it the board would be a
    /// field of holes with no way to tell which one was a piece.
    pub fn sweep(&self) -> Vec<Color> {
        let answer = self.background_color();
        let mut sweep: Vec<Color> = self.current_palette.clone();

        // The answer's own colour is the destination, so it must not also be a
        // stop along the way.
        sweep.retain(|color| !colors_match(*color, answer));
        sweep.push(answer);
        sweep
    }

    /// Perceptual distance between two colors.
    fn distance(a: Oklab, b: Oklab) -> f32 {
        let dl = a.l - b.l;
        let da = a.a - b.a;
        let db = a.b - b.b;
        (dl * dl + da * da + db * db).sqrt()
    }

    /// Moves `base` by `amount` in a random direction that stays displayable.
    fn nudge(rng: &mut ThreadRng, base: Oklab, amount: f32) -> Option<(Oklab, Color)> {
        for _ in 0..24 {
            let hue = rng.gen_range(0.0..std::f32::consts::TAU);
            let lightness_share = rng.gen_range(-0.6_f32..0.6);
            let chromatic_share = (1.0 - lightness_share * lightness_share).sqrt();

            let candidate = base.offset(
                (
                    lightness_share,
                    chromatic_share * hue.cos(),
                    chromatic_share * hue.sin(),
                ),
                amount,
            );

            if let Some(color) = oklab::to_color(candidate) {
                return Some((candidate, color));
            }
        }

        None
    }

    /// Builds a `Mosaic` round: a tiling that fits together everywhere except
    /// at one piece.
    ///
    /// Every cell shares one color here — the pattern carries the puzzle, so a
    /// second variable would only muddy which rule the player is being asked to
    /// apply.
    fn generate_mosaic(&mut self, level: usize, rng: &mut ThreadRng) {
        self.current_slots = vec![];
        let (columns, rows) = mosaic_dimensions_for_level(level);
        let mosaic = wfc::generate(columns, rows, mosaic_violations_for_level(level), rng);

        let base_lab = Self::random_base(rng);
        let base_color = oklab::to_color(base_lab).unwrap_or(Color::rgb(0.5, 0.5, 0.5));

        self.base_color = base_color;
        self.current_colors = vec![base_color; mosaic.tiles.len()];
        self.correct_color_index = mosaic.broken;
        self.current_columns = mosaic.columns;
        self.current_tiles = mosaic.tiles;
    }

    /// A displayable, reasonably saturated color to build a round on.
    fn random_base(rng: &mut ThreadRng) -> Oklab {
        let lightness = rng.gen_range(0.58..0.78);
        let hue = rng.gen_range(0.0..std::f32::consts::TAU);

        // Walk the chroma down until the color fits in sRGB. Some hues simply
        // cannot be as saturated as others at a given lightness, and a clamped
        // color would quietly change the distance the level is set by.
        let mut chroma = rng.gen_range(0.09..0.16);
        for _ in 0..12 {
            let candidate = Oklab::from_lch(lightness, chroma, hue);
            if oklab::to_color(candidate).is_some() {
                return candidate;
            }
            chroma *= 0.85;
        }

        Oklab::from_lch(lightness, 0.0, hue)
    }

    /// Like [`Self::nudge`], but mostly chromatic: the lightness share is
    /// capped so the difference usually has to be judged as a hue or
    /// saturation shift rather than "that one is brighter".
    fn nudge_chromatic(rng: &mut ThreadRng, base: Oklab, amount: f32) -> Option<(Oklab, Color)> {
        for _ in 0..48 {
            let hue = rng.gen_range(0.0..std::f32::consts::TAU);
            let lightness_share = rng.gen_range(-0.45_f32..0.45);
            let chromatic_share = (1.0 - lightness_share * lightness_share).sqrt();

            let candidate = base.offset(
                (
                    lightness_share,
                    chromatic_share * hue.cos(),
                    chromatic_share * hue.sin(),
                ),
                amount,
            );

            if let Some(color) = oklab::to_color(candidate) {
                return Some((candidate, color));
            }
        }

        None
    }

    /// 1-based difficulty level, derived from the score.
    pub fn level(&self) -> usize {
        level_for_score(self.score)
    }

    /// 0.0..=1.0 toward the next level. Drives the HUD progress bar: a target
    /// the player can see approaching pulls harder than an invisible one.
    pub fn progress_to_next_level(&self) -> f32 {
        let level = self.level();
        let start = score_for_level(level);
        let next = score_for_level(level + 1);

        ((self.score - start) as f32 / (next - start).max(1) as f32).clamp(0.0, 1.0)
    }

    /// Points still needed for the next level. There is always a next level.
    pub fn points_to_next_level(&self) -> usize {
        score_for_level(self.level() + 1).saturating_sub(self.score)
    }

    /// The background for this round.
    ///
    /// In the color modes this is *exactly* the answer's color, so the piece
    /// the player is looking for is invisible and has to be found from the
    /// negative space its neighbours leave. That only works because the board
    /// is cut irregularly — see `src/board.rs`, which explains why the same
    /// idea on a grid collapses into "spot the empty cell".
    ///
    /// `Mosaic` is the exception: its puzzle is a pattern, so its pieces all
    /// have to be visible, and it gets a dimmed ground instead.
    pub fn background_color(&self) -> Color {
        if self.game_mode.is_mosaic() {
            return oklab::mix(theme::BACKGROUND, self.base_color, 0.28);
        }

        self.current_colors
            .get(self.correct_color_index)
            .copied()
            .unwrap_or(self.base_color)
    }

    /// The flat color every square wears while a `Memory` round is hidden.
    pub fn hidden_color(&self) -> Color {
        theme::SURFACE_HIDDEN
    }

    /// How long this round's board stays visible in `Memory`.
    pub fn preview_seconds(&self) -> f32 {
        preview_seconds_for_level(self.level())
    }

    pub fn get_score(&self) -> usize {
        self.score
    }

    /// Scores a point. Returns true when that point crossed a level boundary,
    /// so the caller can celebrate it as its own event rather than folding it
    /// into the ordinary per-pick feedback.
    pub fn increase_score(&mut self, game_timer : &mut GameTimer) -> bool {
        let level_before = self.level();
        self.score += 1;
        let leveled_up = self.level() > level_before;

        match self.game_mode {
            GameMode::TimeTrial => {
                let remaining_time = game_timer.timer.duration().as_secs_f32();
                let new_duration = remaining_time + self.get_seconds_added_per_success();
                game_timer.timer.set_duration(Duration::from_secs_f32(new_duration));
            },
            _ => {}
        }

        leveled_up
    }

    pub fn get_seconds_added_per_success(&self) -> f32 {
        self.seconds_added_per_success
    }

    // --- Lives -------------------------------------------------------------

    /// Wrong picks left. Zero in the timed modes, which do not use lives at
    /// all — check `uses_lives` before reading anything into it.
    pub fn lives(&self) -> usize {
        self.lives
    }

    /// The mode's full complement; zero when the mode has none.
    pub fn max_lives(&self) -> usize {
        self.game_mode.starting_lives().unwrap_or(0)
    }

    /// Whether this run can be lost by running out of lives.
    pub fn uses_lives(&self) -> bool {
        self.max_lives() > 0
    }

    /// Charges a miss. Returns true when that was the last life.
    pub fn lose_life(&mut self) -> bool {
        if !self.uses_lives() {
            return false;
        }

        self.lives = self.lives.saturating_sub(1);
        self.lives == 0
    }

    /// Whether the run has been lost. False in every timed mode, where the
    /// clock is what ends things.
    pub fn is_out_of_lives(&self) -> bool {
        self.uses_lives() && self.lives == 0
    }

    /// Whether reaching the current level earns a life back.
    pub fn level_grants_life(&self) -> bool {
        self.uses_lives() && self.level() % LEVELS_PER_EXTRA_LIFE == 0
    }

    /// Hands a life back, never above the maximum. Returns true when one was
    /// actually given, so the caller can say so instead of announcing nothing.
    pub fn gain_life(&mut self) -> bool {
        if !self.uses_lives() || self.lives >= self.max_lives() {
            return false;
        }

        self.lives += 1;
        true
    }

    /// Puts the lives back to where a stored run left them.
    ///
    /// Clamped to the maximum, because the stored value comes from a previous
    /// build's idea of how many a mode gets — and floored at one, because a run
    /// resumed with none left could not be played: the only thing that ends a
    /// run is a miss, and a miss cannot take a life that is already gone.
    pub fn restore_lives(&mut self, lives: usize) {
        let max = self.max_lives();
        self.lives = lives.clamp(usize::from(max > 0), max);
    }

    /// Whether the square at `index` is the odd one out.
    ///
    /// By index, not by color: every other square now shares one color by
    /// design, so comparing channels would be answering a different question.
    pub fn is_correct_color(&self, index : usize) -> bool {
        index == self.correct_color_index
    }

    pub fn setup_timer(&mut self) -> Timer {
        Timer::from_seconds(self.start_seconds, TimerMode::Once)
    }

    pub fn reset(&mut self) {
        self.score = 0;
        // Reads the *current* mode, which is right for a standalone reset. In
        // `setup` this runs before the mode is assigned, and the tail of that
        // function sets the lives again from the new one.
        self.lives = self.game_mode.starting_lives().unwrap_or(0);
    }

    /// Puts the score back to where a stored run left it, so the level, the
    /// piece count and the color distance all pick up where they were.
    pub fn restore_score(&mut self, score: usize) {
        self.score = score;
    }

    /// Walks the round's cells: color, whether it is the answer, and the piece
    /// drawn on it (`None` outside `Mosaic`).
    pub fn for_each_cell<F>(&self, mut f: F)
    where
        F: FnMut(usize, Color, bool, Option<Tile>),
    {
        for (index, color) in self.current_colors.iter().enumerate() {
            f(
                index,
                *color,
                self.is_correct_color(index),
                self.current_tiles.get(index).copied(),
            );
        }
    }

    /// The grid a `Mosaic` round is laid out on.
    ///
    /// Only `Mosaic` has one. Its puzzle is about how pieces meet, so its
    /// pieces have to be laid out where they can meet; the color modes are cut
    /// irregularly instead — see [`Self::slots`].
    pub fn mosaic_grid(&self) -> Option<BoardGrid> {
        if self.current_columns == 0 || self.current_tiles.is_empty() {
            return None;
        }

        let rows = self.current_tiles.len() / self.current_columns;
        Some(self.grid_for_dimensions(self.current_columns, rows))
    }
    
}

#[derive(Debug, Clone, PartialEq)]
pub struct LevelColor {
    pub color : Color,
    pub x : f32,
    pub y : f32,
    pub is_correct_color : bool,
    pub corners : Vec<Vec2>,
    pub tile : Option<Tile>,
}

pub struct LastInteractionEvent {
    clicked_position: Vec2,
    correct_color_index: usize,
    colors: Vec<LevelColor>,
    scored: bool,    
}

impl LastInteractionEvent {
    pub fn new(clicked_position : Vec2, correct_color_index : usize, colors : Vec<LevelColor>, scored : bool) -> Self {
        Self {
            clicked_position,
            correct_color_index,
            colors,
            scored,
        }
    }

    pub fn level_history(&self) -> LevelHistory {
        LevelHistory::new(self.clicked_position, self.correct_color_index, self.colors.clone(), self.scored)
    }
}

#[derive(Debug, Clone)]
pub struct LevelHistory {
    pub clicked_position: Vec2,
    pub correct_color_index: usize,
    pub colors: Vec<LevelColor>,
    pub scored: bool,
}

impl LevelHistory {
    
    pub fn new(clicked_position : Vec2, correct_color_index : usize, colors : Vec<LevelColor>, scored : bool) -> Self {
        Self {
            clicked_position,
            correct_color_index,
            colors,
            scored,
        }
    }

    pub fn for_each_color<F>(&self, mut f: F)
    where
        F: FnMut(usize, &LevelColor),
    {
        for (index, color) in self.colors.iter().enumerate() {
            f(index, color);
        }
    }
    pub fn get_correct_color(&self) -> Color {
        self.colors[self.correct_color_index].color
    }
}

#[derive(Resource)]
pub struct GameHistory {
    pub levels_played: usize,
    pub total_score: usize,
    pub max_streak: usize,
    pub total_time: f32,
    pub game_mode: GameMode,
    current_streak: usize,
    pub levels : Vec<LevelHistory>,
}

impl GameHistory {
    pub fn new() -> Self {
        Self {
            levels_played: 0,
            total_score: 0,
            current_streak: 0,
            max_streak: 0,
            game_mode: GameMode::Infinite,
            total_time: 0.0,
            levels: vec![],
        }
    }

    pub fn set_game_mode(&mut self, game_mode : GameMode) {
        self.game_mode = game_mode;
    }

    /// The run's live streak. Shown in the HUD during play: a streak the player
    /// can watch is something they can be afraid to lose.
    pub fn current_streak(&self) -> usize {
        self.current_streak
    }

    pub fn set_total_time(&mut self, total_time : f32) {
        self.total_time = total_time;
    }

    pub fn add_level(&mut self, level: LevelHistory) {
        self.levels_played += 1;
        
        if level.scored {
            self.current_streak += 1;
            self.total_score += 1;
        } else {
            self.current_streak = 0;
        }

        if self.current_streak > self.max_streak {
            self.max_streak = self.current_streak;
        }


        self.levels.push(level);
    }

    pub fn for_each_level<F>(&self, mut f: F, start_index : usize, n_of_items : usize)
    where
        F: FnMut(usize, &LevelHistory),
    {
        for (index, level) in self.levels.iter().enumerate().skip(start_index).take(n_of_items) {
            f(index, level);
        }
    }
    
    pub fn get_level_history(&self, index : usize) -> &LevelHistory {
        self.levels.get(index).unwrap()
    }

    /// Restores a resumed run's score, so the summary at the end counts the
    /// whole run and not just the part played after coming back.
    pub fn restore(&mut self, score: usize) {
        self.total_score = score;
    }

    pub fn reset(&mut self) {
        self.levels_played = 0;
        self.total_score = 0;
        self.current_streak = 0;
        self.max_streak = 0;
        self.total_time = 0.0;
        self.levels = vec![];
    }

    pub fn get_formatted_time(&self) -> String {
        let minutes = self.total_time as u32 / 60;
        let seconds = self.total_time as u32 % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }


}

impl Default for GameHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Resource, Reflect, Debug)]
pub struct GameTimer {
    pub timer: Timer,
}

/// Swallows input for the handful of frames between a pick and the new board's
/// entities actually existing.
///
/// The background transition used to do this job as a side effect of locking
/// input for its whole length. It is a real job, and it outlives the lock:
/// `spawn_objects` despawns the old pieces through `Commands`, which are not
/// applied until the end of the stage, so on the frame a round is regenerated
/// `player_interaction` can still see the *old* entities and bank a second
/// point against a round that is already over. This lock covers exactly that
/// window and nothing more — it is shorter than any human double tap, so a
/// player who wants to answer the instant the board appears can.
#[derive(Resource, Default)]
pub struct RoundIntro {
    timer: Option<Timer>,
}

impl RoundIntro {
    const LOCK_SECONDS: f32 = 0.12;

    pub fn arm(&mut self) {
        self.timer = Some(Timer::from_seconds(Self::LOCK_SECONDS, TimerMode::Once));
    }

    pub fn is_locked(&self) -> bool {
        self.timer.is_some()
    }

    pub fn clear(&mut self) {
        self.timer = None;
    }

    pub fn tick(&mut self, delta: std::time::Duration) {
        let Some(timer) = self.timer.as_mut() else {
            return;
        };

        timer.tick(delta);
        if timer.finished() {
            self.timer = None;
        }
    }
}

/// Drives a `Memory` round: board visible, then blank.
///
/// Kept as a resource rather than a component on the squares because the phase
/// belongs to the round, not to any one square — and because the squares are
/// despawned and respawned between rounds.
#[derive(Resource, Default)]
pub struct MemoryPhase {
    preview: Option<Timer>,
    hidden: bool,
}

impl MemoryPhase {
    /// Starts the preview for a new board.
    pub fn begin(&mut self, seconds: f32) {
        self.preview = Some(Timer::from_seconds(seconds, TimerMode::Once));
        self.hidden = false;
    }

    /// Nothing to hide: the mode is off, or the board is already blank.
    pub fn clear(&mut self) {
        self.preview = None;
        self.hidden = false;
    }

    /// Whether the colors are still on screen. Input is refused while they are:
    /// picking during the preview would make the mode a normal round.
    pub fn is_previewing(&self) -> bool {
        self.preview.is_some()
    }

    pub fn is_hidden(&self) -> bool {
        self.hidden
    }

    /// Advances the preview. Returns true on the frame it ends, which is the
    /// frame the board should go blank.
    pub fn tick(&mut self, delta: std::time::Duration) -> bool {
        let Some(timer) = self.preview.as_mut() else {
            return false;
        };

        timer.tick(delta);

        if timer.finished() {
            self.preview = None;
            self.hidden = true;
            return true;
        }

        false
    }
}

/// Holds the current board in place for a beat after a wrong pick.
///
/// Without this the next round is generated in the same frame as the miss, and
/// the "here was the right answer" outline would be drawn over a board that no
/// longer exists. The pause is what lets a miss teach something instead of just
/// costing a point; input stays locked for its duration so the player cannot
/// pick again into a board that is about to be replaced.
#[derive(Resource, Default)]
pub struct PendingLevelStart {
    timer: Option<Timer>,
}

impl PendingLevelStart {
    /// Holds the board for `seconds` — long enough to look at, short enough not
    /// to feel like a penalty. See `GameMode::hold_seconds`.
    pub fn hold(&mut self, seconds: f32) {
        self.timer = Some(Timer::from_seconds(seconds, TimerMode::Once));
    }

    pub fn is_holding(&self) -> bool {
        self.timer.is_some()
    }

    /// Advances the hold. Returns true on the frame it ends.
    pub fn tick(&mut self, delta: std::time::Duration) -> bool {
        let Some(timer) = self.timer.as_mut() else {
            return false;
        };

        timer.tick(delta);

        if timer.finished() {
            self.timer = None;
            return true;
        }

        false
    }

    pub fn clear(&mut self) {
        self.timer = None;
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    /// The curve replaced a nine-entry table, and it has to reproduce it: a
    /// stored run carries a score, and a player who left at level seven must
    /// come back to level seven.
    #[test]
    fn the_curve_reproduces_the_table_it_replaced() {
        const TABLE: [usize; 9] = [0, 5, 11, 20, 32, 47, 65, 86, 110];

        for (index, start) in TABLE.iter().enumerate() {
            assert_eq!(score_for_level(index + 1), *start, "level {}", index + 1);
        }
    }

    /// Exactly one of the two ways to lose per mode. A mode with both would
    /// make the player watch two falling numbers at once; a mode with neither
    /// is the bug this replaced — three of the five could only end by the
    /// player pressing "encerrar partida".
    #[test]
    fn every_mode_has_one_way_to_run_out() {
        for mode in GameMode::iter() {
            let has_lives = mode.starting_lives().is_some();
            let charges_time = mode.miss_penalty_seconds() > 0.0;

            assert_ne!(
                has_lives, charges_time,
                "{:?} should charge a miss one way or the other, not both or neither",
                mode.as_str()
            );
            assert_eq!(
                has_lives,
                !mode.is_timed(),
                "{} has a clock and lives, or neither",
                mode.as_str()
            );
        }
    }

    /// Lives count down to zero and stop there, and zero is what ends the run.
    #[test]
    fn the_last_life_ends_the_run() {
        let mut puzzle = ColorPuzzle::new();
        puzzle.setup(&GameMode::Infinite);

        let max = puzzle.max_lives();
        assert!(max > 0, "Infinite should have lives");
        assert_eq!(puzzle.lives(), max, "a run starts with a full complement");

        for remaining in (1..max).rev() {
            assert!(!puzzle.lose_life(), "the run ended early");
            assert_eq!(puzzle.lives(), remaining);
            assert!(!puzzle.is_out_of_lives());
        }

        assert!(puzzle.lose_life(), "the last life should end the run");
        assert!(puzzle.is_out_of_lives());

        // And it stays ended: nothing underflows past zero.
        assert!(puzzle.lose_life());
        assert_eq!(puzzle.lives(), 0);
    }

    /// A timed mode has no lives to lose, so a miss there must not be able to
    /// end the run through this path — its clock is what does that.
    #[test]
    fn a_timed_run_cannot_run_out_of_lives() {
        let mut puzzle = ColorPuzzle::new();
        puzzle.setup(&GameMode::AgainstTheClock);

        assert_eq!(puzzle.max_lives(), 0);
        assert!(!puzzle.lose_life());
        assert!(!puzzle.is_out_of_lives());
    }

    /// A life back every fifth level, and never a fourth one in a three-life
    /// mode: the recovery is what keeps `Infinite` from being a sprint, not a
    /// way to bank a buffer.
    #[test]
    fn lives_come_back_but_never_past_the_maximum() {
        let mut puzzle = ColorPuzzle::new();
        puzzle.setup(&GameMode::Infinite);

        assert!(!puzzle.gain_life(), "a full run has nothing to gain");
        assert_eq!(puzzle.lives(), puzzle.max_lives());

        puzzle.lose_life();
        assert!(puzzle.gain_life());
        assert_eq!(puzzle.lives(), puzzle.max_lives());

        // The grant lands on every fifth level and nowhere else.
        for level in 1..40 {
            puzzle.restore_score(score_for_level(level));
            assert_eq!(
                puzzle.level_grants_life(),
                level % LEVELS_PER_EXTRA_LIFE == 0,
                "level {}",
                level
            );
        }
    }

    /// Setting a mode up always seeds its lives — this is the single funnel all
    /// three of the places a run starts go through.
    #[test]
    fn setup_seeds_the_lives_for_every_mode() {
        let mut puzzle = ColorPuzzle::new();

        for mode in GameMode::iter() {
            puzzle.setup(&mode);
            assert_eq!(
                puzzle.lives(),
                mode.starting_lives().unwrap_or(0),
                "{} was set up with the wrong lives",
                mode.as_str()
            );
        }
    }

    /// A resumed run comes back where it left off, but never in a state it
    /// cannot be played from.
    #[test]
    fn a_restored_run_is_always_playable() {
        let mut puzzle = ColorPuzzle::new();
        puzzle.setup(&GameMode::Memory);

        puzzle.restore_lives(1);
        assert_eq!(puzzle.lives(), 1);

        // Zero would be a run that is already over and can never end.
        puzzle.restore_lives(0);
        assert_eq!(puzzle.lives(), 1);

        // A save from a build that was more generous does not overfill it.
        puzzle.restore_lives(99);
        assert_eq!(puzzle.lives(), puzzle.max_lives());
    }

    /// `level_for_score` is the inverse, at the boundaries as well as between
    /// them.
    #[test]
    fn levels_and_scores_agree() {
        for level in 1..2_000 {
            let start = score_for_level(level);
            assert_eq!(level_for_score(start), level, "at the start of level {}", level);

            if start > 0 {
                assert_eq!(
                    level_for_score(start - 1),
                    level - 1,
                    "one point short of level {}",
                    level
                );
            }
        }
    }

    /// No level is the last one, and nothing overflows on the way there — on
    /// wasm32 `usize` is 32 bits, which the plain form of the formula outgrows.
    #[test]
    fn the_curve_never_ends() {
        let mut previous = 0;

        for level in 1..100_000 {
            let start = score_for_level(level);
            assert!(start >= previous, "level {} went backwards", level);
            previous = start;
        }

        assert!(score_for_level(usize::MAX) > 0);
    }

    /// Every dial moves in one direction and settles, so a level is never
    /// easier than the one before it.
    #[test]
    fn the_dials_are_monotone() {
        let mut columns = 0;
        let mut empty = 0.0;
        let mut palette = 0;
        let mut delta = f32::MAX;
        let mut preview = f32::MAX;

        for level in 1..500 {
            let next_columns = columns_for_level(level);
            let next_empty = empty_share_for_level(level);
            let next_palette = palette_size_for_level(level);
            let next_delta = color_delta_for_level(level);
            let next_preview = preview_seconds_for_level(level);

            assert!(next_columns >= columns);
            assert!(next_empty >= empty - 1e-6);
            assert!(next_palette >= palette);
            assert!(next_delta <= delta + 1e-6);
            assert!(next_preview <= preview + 1e-6);

            columns = next_columns;
            empty = next_empty;
            palette = next_palette;
            delta = next_delta;
            preview = next_preview;
        }

        // And they settle where the plan says they do.
        assert_eq!(columns, board::MAX_COLUMNS);
        assert!((empty - 0.5).abs() < 0.01);
        assert_eq!(palette, 8);
        assert!((delta - MIN_COLOR_DELTA).abs() < 1e-4);
    }

    /// The board must come out in several colours.
    ///
    /// It once did not: the palette was built by nudging away from the base by
    /// a distance no displayable colour could reach, every attempt was rejected,
    /// and all groups fell back to the base — a board in one flat blue, with no
    /// pattern to read and no group for the ground to visit on its way past.
    /// The distance is a property of the construction now, and this is what
    /// says so.
    #[test]
    fn every_colour_group_is_visibly_its_own() {
        let mut rng = rand::thread_rng();

        for level in [1usize, 2, 5, 10, 20, 50] {
            let groups = palette_size_for_level(level);
            let delta = color_delta_for_level(level);

            for _ in 0..200 {
                let base = ColorPuzzle::random_base(&mut rng);
                let palette = ColorPuzzle::palette(&mut rng, base, groups);
                assert_eq!(palette.len(), groups);

                for (i, (a, _)) in palette.iter().enumerate() {
                    for (j, (b, _)) in palette.iter().enumerate().skip(i + 1) {
                        let distance = ColorPuzzle::distance(*a, *b);
                        // The bar is a floor on the construction, not on
                        // fairness. The arc's tightest case is eight groups at
                        // the chroma clamp, where neighbouring hues are about
                        // 25 degrees apart and the chord comes to roughly 0.02.
                        //
                        // Keeping the answer clear of the *other* groups is not
                        // this test's job and could not be a property of the
                        // palette anyway: `answer_color` retries directions
                        // until it finds one that clears them, and
                        // `the_answer_is_alone_in_its_colour` checks that it
                        // does. What this catches is the failure that actually
                        // happened — every group identical, distance zero.
                        assert!(
                            distance > 0.02,
                            "level {level}: groups {i} and {j} only {distance} apart \
                             (delta {delta})"
                        );
                    }
                }
            }
        }
    }

    /// The answer hides inside its own group and clear of every other one.
    #[test]
    fn the_answer_is_alone_in_its_colour() {
        let mut rng = rand::thread_rng();

        for level in [1usize, 5, 20] {
            let groups = palette_size_for_level(level);
            let delta = color_delta_for_level(level);

            for _ in 0..100 {
                let base = ColorPuzzle::random_base(&mut rng);
                let palette = ColorPuzzle::palette(&mut rng, base, groups);

                for group in 0..groups {
                    let answer = ColorPuzzle::answer_color(&mut rng, &palette, group, delta);

                    // Its own group is the one it must NOT be far from — that
                    // is the puzzle. Everything else it must be clear of.
                    assert!(
                        !colors_match(answer, palette[group].1),
                        "level {level}: the answer came out as its own group's colour"
                    );

                    for (other, (_, color)) in palette.iter().enumerate() {
                        if other == group {
                            continue;
                        }
                        assert!(
                            !colors_match(answer, *color),
                            "level {level}: the answer matched group {other}"
                        );
                    }
                }
            }
        }
    }
}
