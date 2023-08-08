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
            GameMode::Infinite => "Infinto",
            GameMode::AgainstTheClock => "Contra o Tempo",
            GameMode::TimeTrial => "Soma de Tempo",
        }
    }
}


#[derive(Resource, Debug, Reflect)]
pub struct ColorPuzzle {
    score: usize,
    current_colors: Vec<Color>,
    correct_color_index: usize,
    pub game_mode: GameMode,
    pub difficulty: usize,
    pub seconds_added_per_success: f32,
    pub objects_per_difficulty: usize,
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

pub fn score_to_increase_difficulty_formula(score: usize) -> usize {
    match score {
        0..=5 => 2,
        6..=10 => 3,
        11..=30 => 4,
        31..=50 => 5,
        51..=60 => 6,
        _ => 7,
    }
}


impl ColorPuzzle {
   pub  fn new() -> Self {
        let mut puzzle =  Self {
            score: 0,
            current_colors: vec![],
            correct_color_index: 0,
            game_mode: GameMode::TimeTrial,
            difficulty: 1,
            objects_per_difficulty: 2,
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

        let mut colors = vec![];

        let correct_color = Color::rgb(rng.gen(), rng.gen(), rng.gen());

        colors.push(correct_color);

        for _ in 0..self.get_score_color_count() {
            let mut color = correct_color.clone();
            
            

            let color_variation = rng.gen_range(0.0..0.1);

            color.set_r( color.r() + color_variation);
            color.set_g( color.g() + color_variation);
            color.set_b( color.b() + color_variation);
            

            colors.push(color);
        }

        self.current_colors = colors;
        self.correct_color_index = rng.gen_range(0..self.get_score_color_count());

    }

    pub fn get_score_color_count(&self) -> usize {
        (self.difficulty * score_to_increase_difficulty_formula(self.score)) * self.objects_per_difficulty
    }

    pub fn get_color(&self) -> Color {
        self.current_colors[self.correct_color_index]
    }

    pub fn get_score(&self) -> usize {
        self.score
    }

    pub fn increase_score(&mut self, game_timer : &mut GameTimer) {
        self.score += 1;

        match self.game_mode {
            GameMode::TimeTrial => {
                let remaining_time = game_timer.timer.duration().as_secs_f32();
                let new_duration = remaining_time + self.get_seconds_added_per_success();
                game_timer.timer.set_duration(Duration::from_secs_f32(new_duration));
            },
            _ => {}
        }
    }

    pub fn get_seconds_added_per_success(&self) -> f32 {
        self.seconds_added_per_success
    }

    pub fn is_correct_color(&self, index : usize) -> bool {
        let color = self.current_colors[index];
        color.r() == self.get_color().r() && color.g() == self.get_color().g() && color.b() == self.get_color().b() && color.a() == self.get_color().a()
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

    pub fn set_total_time(&mut self, total_time : f32) {
        self.total_time = total_time;
    }

    pub fn add_level(&mut self, level: LevelHistory) {
        self.levels_played += 1;
        
        if level.scored {
            self.max_streak = self.max_streak.max(self.current_streak);
            self.current_streak += 1;
            self.total_score += 1;
        } else {
            self.current_streak = 0;
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
    
    pub fn get_level_history(&mut self, index : usize) -> &LevelHistory {
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