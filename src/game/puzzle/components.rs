use bevy::prelude::*;
use bevy_utils::Duration;
use rand::prelude::*;

#[derive(Component)]
pub struct PuzzleColor {
    pub index : usize,
    pub is_correct_color : bool,
    pub color : Color,
    pub x : f32,
    pub y : f32,
}

impl PuzzleColor {
    pub fn as_level_color(&self) -> LevelColor {
        LevelColor {
            color : self.color,
            x : self.x,
            y : self.y,
            is_correct_color : self.is_correct_color,
            
        }
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
}

impl GameMode {
    pub fn iter() -> impl Iterator<Item = GameMode> {
        [GameMode::Infinite, GameMode::AgainstTheClock, GameMode::TimeTrial].iter().copied()
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            GameMode::Infinite => "Infinito",
            GameMode::AgainstTheClock => "Contra o Tempo",
            GameMode::TimeTrial => "Soma de Tempo",
        }
    }

    /// One line telling the player what they are choosing, so the mode select
    /// is an informed choice rather than three unlabelled doors.
    pub fn description(&self) -> &'static str {
        match self {
            GameMode::Infinite => "Sem tempo. Jogue no seu ritmo.",
            GameMode::AgainstTheClock => "60 segundos. Marque o maximo que puder.",
            GameMode::TimeTrial => "30 segundos. Cada acerto soma 3s.",
        }
    }

    /// Stable key for persisted best scores. Never change these strings without
    /// migrating stored values.
    pub fn storage_key(&self) -> &'static str {
        match self {
            GameMode::Infinite => "infinite",
            GameMode::AgainstTheClock => "against_the_clock",
            GameMode::TimeTrial => "time_trial",
        }
    }

    /// Whether a run in this mode can ever end on its own.
    pub fn is_timed(&self) -> bool {
        !matches!(self, GameMode::Infinite)
    }
}


#[derive(Resource, Debug, Reflect)]
pub struct ColorPuzzle {
    score: usize,
    current_colors: Vec<Color>,
    correct_color_index: usize,
    pub game_mode: GameMode,
    pub seconds_added_per_success: f32,
    pub shape_size: f32,
    pub start_seconds: f32,
    pub transition_seconds: f32,
    pub width: f32,
    pub height: f32,
    pub screen_padding : f32,
}


impl Default for ColorPuzzle {
    
    fn default() -> Self {
        Self::new()
    }
}

/// Whether two colors are the same square on screen.
///
/// The puzzle identifies the target by color rather than by entity, so this is
/// the definition of "you picked the right one".
pub fn colors_match(a: Color, b: Color) -> bool {
    a.r() == b.r() && a.g() == b.g() && a.b() == b.b() && a.a() == b.a()
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

/// Smallest and largest number of squares on screen.
const MIN_COLORS: usize = 4;
const MAX_COLORS: usize = 12;

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

/// How far apart the distractor colors sit from the target, by level.
///
/// This is the main difficulty dial. Previously difficulty came only from
/// putting *more* squares on screen while the color distance stayed fixed,
/// which made early jumps feel abrupt and late rounds merely crowded. Narrowing
/// the distance instead keeps the challenge tracking the player's improving
/// discrimination — the flow channel — rather than their patience.
pub fn color_variation_for_level(level: usize) -> f32 {
    let steps = (level.saturating_sub(1)) as f32;
    (0.16 - steps * 0.015).max(0.05)
}

/// Number of squares on screen at a level. Grows one at a time and stops well
/// before the board turns into a wall of confetti.
pub fn color_count_for_level(level: usize) -> usize {
    (MIN_COLORS + level.saturating_sub(1)).min(MAX_COLORS)
}


impl ColorPuzzle {
   pub  fn new() -> Self {
        let mut puzzle =  Self {
            score: 0,
            current_colors: vec![],
            correct_color_index: 0,
            game_mode: GameMode::TimeTrial,
            seconds_added_per_success: 3.0,
            shape_size: 200.0,
            start_seconds: 60.0,
            transition_seconds: 1.,
            width: 800.0,
            height: 600.0,
            screen_padding : 50.0,
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
        }
   
        
    }

    pub fn set_window_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;


        self.shape_size = if width / 4.0 > 140.0 {
            140.0
        } else {
            width / 4.0
        };
    }

    pub fn get_width(&self) -> f32 {
        self.width - self.screen_padding
    }

    pub fn get_height(&self) -> f32 {
        self.height - self.screen_padding
    }

    pub fn get_correct_color_index(&self) -> usize {
        self.correct_color_index
    }


    pub fn generate_colors(&mut self) {
        let mut rng = rand::thread_rng();

        let level = self.level();
        let count = color_count_for_level(level);
        let variation = color_variation_for_level(level);
        // Every distractor has to be *visibly* off the base in at least one
        // channel. Without this floor the random walk can land on a duplicate of
        // the target, which reads to the player as "I picked the right one and
        // the game said no" — the fastest way to destroy trust in a color game.
        let min_delta = (variation * 0.4).max(0.025);

        // Keep the base away from the extremes so variation has room to move in
        // both directions without clamping flattening it back out.
        let channel = |rng: &mut ThreadRng| rng.gen_range(0.12..0.72);
        let dominant = rng.gen_range(0..4);
        let mut red = channel(&mut rng);
        let mut green = channel(&mut rng);
        let mut blue = channel(&mut rng);

        // Push one channel up so the round has a recognisable hue instead of
        // another wash of grey.
        match dominant {
            0 => red = rng.gen_range(0.45..0.80),
            1 => green = rng.gen_range(0.45..0.80),
            2 => blue = rng.gen_range(0.45..0.80),
            _ => {}
        }

        let base_color = Color::rgb(red, green, blue);

        let mut colors = vec![base_color];

        while colors.len() < count {
            let mut candidate = base_color;

            // Guarantee movement in one channel, then jitter the rest freely.
            let forced_channel = rng.gen_range(0..3);
            for index in 0..3 {
                let magnitude = if index == forced_channel {
                    rng.gen_range(min_delta..variation.max(min_delta * 1.5))
                } else {
                    rng.gen_range(0.0..variation)
                };
                // Both directions: if distractors could only get lighter, the
                // darkest square on screen would be a free tell.
                let delta = if rng.gen_bool(0.5) { magnitude } else { -magnitude };

                match index {
                    0 => candidate.set_r((candidate.r() + delta).clamp(0.0, 1.0)),
                    1 => candidate.set_g((candidate.g() + delta).clamp(0.0, 1.0)),
                    _ => candidate.set_b((candidate.b() + delta).clamp(0.0, 1.0)),
                };
            }

            // Exact-equality duplicates would produce two "correct" squares,
            // since `is_correct_color` compares channels directly.
            if colors.iter().any(|existing| colors_match(*existing, candidate)) {
                continue;
            }

            colors.push(candidate);
        }

        self.correct_color_index = rng.gen_range(0..colors.len());
        self.current_colors = colors;
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

    pub fn get_color(&self) -> Color {
        self.current_colors[self.correct_color_index]
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

    pub fn is_correct_color(&self, index : usize) -> bool {
        colors_match(self.current_colors[index], self.get_color())
    }

    pub fn setup_timer(&mut self) -> Timer {
        Timer::from_seconds(self.start_seconds, TimerMode::Once)
    }

    pub fn reset(&mut self) {
        self.score = 0;
    }

    pub fn for_each_color<F>(&self, mut f: F)
    where
        F: FnMut(usize, Color, bool),
    {
        for (index, color) in self.current_colors.iter().enumerate() {
            f(index, *color, self.is_correct_color(index));
        }
    }
    
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub struct LevelColor {
    pub color : Color,
    pub x : f32,
    pub y : f32,
    pub is_correct_color : bool,
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
        F: FnMut(usize, LevelColor),
    {
        for (index, color) in self.colors.iter().enumerate() {
            f(index, *color);
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

    pub fn get_previous_level_history(&self, index : usize) -> Option<&LevelHistory> {
        if index == 0 {
            return None;
        }
        
        self.levels.get(index - 1)
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