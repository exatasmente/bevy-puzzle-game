use bevy::prelude::*;
use bevy_utils::Duration;
use rand::prelude::*;

use crate::board::{self, Piece};
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
        Piece { centre : Vec2::new(self.x, self.y), corners : self.corners.clone() }.contains(point)
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
            GameMode::Infinite => "Sem tempo. No seu ritmo.",
            GameMode::AgainstTheClock => "60 segundos no relogio.",
            GameMode::TimeTrial => "30s. Cada acerto soma 3s.",
            GameMode::Memory => "As cores somem. Lembre.",
            GameMode::Mosaic => "Ache a peca que nao encaixa.",
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
}


#[derive(Resource, Debug, Reflect)]
pub struct ColorPuzzle {
    score: usize,
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

/// Celebration text for a streak length, if that length deserves one.
///
/// Milestones are spaced out on purpose. Praise on every second pick stops
/// being information and becomes noise the player learns to ignore.
pub fn streak_milestone_label(streak: usize) -> Option<&'static str> {
    match streak {
        3 => Some("SEQUENCIA x3"),
        5 => Some("EM CHAMAS!"),
        10 => Some("IMPARAVEL!"),
        15 => Some("LENDARIO!"),
        other if other > 15 && other % 10 == 0 => Some("INACREDITAVEL!"),
        _ => None,
    }
}

/// Score at which each level begins. Index 0 is level 1.
///
/// Deliberately front-loaded: the first level up lands after five points, while
/// the player is still deciding whether this game is worth their attention. The
/// gaps widen after that so later levels stay meaningful.
const LEVEL_START_SCORES: [usize; 9] = [0, 5, 11, 20, 32, 47, 65, 86, 110];

/// Smallest and largest number of pieces on screen.
///
/// Higher than it was: the board is cut irregularly now, and four pieces come
/// out as four slabs. A hidden slab is a quadrant of empty screen, which is not
/// a puzzle. Six is the fewest that reads as a mosaic with a hole in it.
const MIN_COLORS: usize = 6;
const MAX_COLORS: usize = 14;

/// 1-based level for a score.
pub fn level_for_score(score: usize) -> usize {
    LEVEL_START_SCORES
        .iter()
        .rposition(|start| score >= *start)
        .unwrap_or(0)
        + 1
}

/// Score at which `level` begins. Levels past the table clamp to the last entry.
pub fn score_for_level(level: usize) -> usize {
    let index = level.saturating_sub(1).min(LEVEL_START_SCORES.len() - 1);
    LEVEL_START_SCORES[index]
}

pub fn max_level() -> usize {
    LEVEL_START_SCORES.len()
}

/// Perceptual distance between the odd square and the rest, by level.
///
/// This is the difficulty dial, in Oklab units: about 0.02 is subtle, 0.01 is
/// hard, and below 0.005 is a coin flip. Because the unit is perceptual, a
/// level means the same thing whether the round came out olive or navy — which
/// was not true of the old per-channel sRGB variation.
pub fn color_delta_for_level(level: usize) -> f32 {
    let steps = (level.saturating_sub(1)) as f32;
    (0.080 - steps * 0.0075).max(0.018)
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
        2 | 3 => (3, 3),
        4 | 5 => (3, 4),
        6 | 7 => (4, 4),
        _ => (4, 5),
    }
}

/// How many of the odd piece's four edges disagree with their surroundings.
///
/// Four is unmissable; two is a real search. One is only reachable against the
/// board's outer edge — `wfc::corrupt` explains why a lone disagreement between
/// two pieces would be unfair rather than hard — and the generator falls back
/// to two when it cannot place one.
pub fn mosaic_violations_for_level(level: usize) -> usize {
    match level {
        1 => 4,
        2 | 3 => 3,
        4 | 5 | 6 => 2,
        _ => 1,
    }
}

/// Number of squares on screen at a level. Grows one at a time and stops well
/// before the board turns into a wall of confetti.
pub fn color_count_for_level(level: usize) -> usize {
    (MIN_COLORS + level.saturating_sub(1)).min(MAX_COLORS)
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
            current_colors: vec![],
            base_color: Color::rgb(0.5, 0.5, 0.5),
            current_tiles: vec![],
            current_slots: vec![],
            current_columns: 0,
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

        // Cut the board first and take the round's size from it. The cut drops
        // any piece too thin to tap, so asking for a count and assuming it is
        // what you got would leave a color — possibly the answer — with no
        // piece to live on.
        let slots = self.cut_board(color_count_for_level(level), &mut rng);
        let count = slots.len().max(2);

        // The centre of the round, kept off the extremes of lightness so the
        // colors around it have room to move in any direction and still be
        // displayable.
        let base_lab = Self::random_base(&mut rng);
        let base_color = oklab::to_color(base_lab).unwrap_or(Color::rgb(0.5, 0.5, 0.5));

        // Every square gets its own color. They used to be literally identical,
        // which read as a flat wall of one paint and made the odd square a
        // difference from its *neighbours* rather than from the group.
        //
        // Each distractor is nudged off the base by at most `CLUSTER` of the
        // round's delta, so the group stays a tight cluster: the widest gap
        // inside it is 2 * CLUSTER * delta, while the odd color sits at least
        // (1 - CLUSTER) * delta from every one of them. With CLUSTER at 0.22
        // that is 0.44 against 0.78 — the outlier is still the outlier by a
        // wide margin, and there is exactly one defensible answer.
        const CLUSTER: f32 = 0.22;

        let mut labs: Vec<Oklab> = Vec::with_capacity(count);
        let mut colors: Vec<Color> = Vec::with_capacity(count);

        while colors.len() + 1 < count {
            let amount = rng.gen_range(CLUSTER * 0.45..CLUSTER) * delta;
            let Some((lab, color)) = Self::nudge(&mut rng, base_lab, amount) else {
                continue;
            };

            // No two squares may share a color: a repeat invites the player to
            // read the pair as a rule of its own.
            let separation = delta * CLUSTER * 0.4;
            if labs.iter().any(|other| Self::distance(*other, lab) < separation) {
                continue;
            }

            labs.push(lab);
            colors.push(color);
        }

        // The odd one out. Its direction is random, but weighted away from pure
        // lightness: a square that is simply lighter than the rest is the
        // easiest difference the visual system has, and letting the dice pick it
        // half the time made levels swing between trivial and hard.
        let odd_color = Self::nudge_chromatic(&mut rng, base_lab, delta)
            .map(|(_, color)| color)
            .unwrap_or(base_color);

        self.correct_color_index = rng.gen_range(0..count);
        colors.insert(self.correct_color_index, odd_color);

        self.base_color = base_color;
        self.current_tiles = vec![];
        self.current_columns = 0;
        self.current_slots = slots;
        self.current_colors = colors;
    }

    /// Cuts this round's pieces out of the play area.
    fn cut_board(&self, count: usize, rng: &mut ThreadRng) -> Vec<Piece> {
        let area = self.play_area();
        let min = Vec2::new(-area.x / 2.0, self.play_bottom());
        let max = Vec2::new(area.x / 2.0, self.play_bottom() + area.y);

        board::layout(min, max, count, rng)
    }

    /// This round's pieces, or an empty list in `Mosaic`.
    pub fn slots(&self) -> &[Piece] {
        &self.current_slots
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
        if level >= max_level() {
            return 1.0;
        }

        let start = score_for_level(level);
        let next = score_for_level(level + 1);
        if next <= start {
            return 1.0;
        }

        ((self.score - start) as f32 / (next - start) as f32).clamp(0.0, 1.0)
    }

    /// Points still needed for the next level, if there is one.
    pub fn points_to_next_level(&self) -> Option<usize> {
        let level = self.level();
        if level >= max_level() {
            return None;
        }

        Some(score_for_level(level + 1).saturating_sub(self.score))
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
    /// How long the missed board stays up. Long enough to look at, short enough
    /// not to feel like a penalty.
    const HOLD_SECONDS: f32 = 0.7;

    pub fn hold(&mut self) {
        self.timer = Some(Timer::from_seconds(Self::HOLD_SECONDS, TimerMode::Once));
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