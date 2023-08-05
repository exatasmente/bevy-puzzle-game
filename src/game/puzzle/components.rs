use bevy::prelude::*;
use rand::prelude::*;

#[derive(Component)]
pub struct PuzzleColor {
    pub index : usize,
    pub is_correct_color : bool,
}


pub struct StartLevelEvent;

#[derive(Debug, Reflect)]
pub enum GameMode {
    Infinite,
    AgainstTheClock,
    TimeTrial,
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
        5..=10 => 3,
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
            game_mode: GameMode::Infinite,
            difficulty: 1,
            objects_per_difficulty: 2,
            seconds_added_per_success: 3.0,
            shape_size: 200.0,
            start_seconds: 60.0,
            transition_seconds: 1.2,
            width: 800.0,
            height: 600.0,
            screen_padding : 50.0,
        };

        puzzle.generate_colors();

        puzzle
    }

    pub fn set_window_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    pub fn get_width(&self) -> f32 {
        self.width - self.screen_padding
    }

    pub fn get_height(&self) -> f32 {
        self.height - self.screen_padding
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

    pub fn increase_score(&mut self) {
        self.score += 1;
    }

    pub fn set_score(&mut self, score : usize) {
        self.score = score;
    }

    pub fn is_correct_color(&self, index : usize) -> bool {
        let color = self.current_colors[index];
        color.r() == self.get_color().r() && color.g() == self.get_color().g() && color.b() == self.get_color().b() && color.a() == self.get_color().a()
    }

    pub fn setup_timer(&mut self) -> Timer{
        Timer::from_seconds(self.start_seconds, TimerMode::Once)
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