use bevy::prelude::*;


#[derive(Component, Reflect, Default, PartialEq, Clone, Copy, Debug)]
pub enum PlayerState {
    #[default]
    Idle,
    WalkForward,
    WalkBackward,
    WalkLeft,
    WalkRight,
}

#[derive(Component, Reflect, Default, PartialEq, Clone, Copy, Debug)]
pub enum StressLevel{
    #[default]
    Normal,
    Happy,
    Angry,
    Annoyed,
    Stressed,
    Afraid,
}

#[derive(Component, Reflect)]
pub struct Player {
    pub current_state : PlayerState,
    pub previous_state : PlayerState,
    pub speed : f32,

}

impl Player {    
    pub fn set_state(&mut self, state : PlayerState) {
        if state == self.current_state {
            return;
        }

        self.previous_state = self.current_state;
        self.current_state = state;
    }

    pub fn get_state_frames(&mut self) -> Vec<usize>{
        match self.current_state {
         PlayerState::Idle =>  vec![18,19,20, 21,22,23, 21,22,23, 21,22,23, 21,22,23, 21,22,23, 21,22,23, 21,22,23],
         PlayerState::WalkForward => (0..3).collect(),
         PlayerState::WalkBackward => (8..11).collect(),
         PlayerState::WalkLeft => (12..15).collect(),
         PlayerState::WalkRight =>(4..7).collect(),        
        } 
     }

     pub fn get_speed(&self) -> f32{
        self.speed
     }
}


#[derive(Component, Reflect)]
pub struct PlayerStats {
    hunger : usize,
    thirst : usize,
    stress : StressLevel,
    pub stats_timer : Timer,
}

impl PlayerStats {
    pub fn new() -> Self {
        Self {
            hunger : 0,
            thirst : 0,
            stress : StressLevel::Normal,
            stats_timer : Timer::from_seconds(10.0, TimerMode::Repeating),
        }
    }

    pub fn get_hunger(&self) -> usize {
        self.hunger
    }

    pub fn get_thirst(&self) -> usize {
        self.thirst
    }

    pub fn get_stress(&self) -> StressLevel {
        self.stress
    }


    pub fn increase_hunger(&mut self, hunger : usize) {
        self.hunger += hunger;
    }

    pub fn increase_thirst(&mut self, thirst : usize) {
        self.thirst += thirst;
    }

    pub fn decrease_hunger(&mut self, hunger : usize) {
        self.hunger -= hunger;
    }

    pub fn decrease_thirst(&mut self, thirst : usize) {
        self.thirst -= thirst;
    }

    pub fn update_stress_level(&mut self) {
        self.stress = self.calculate_stress_level();
    }

    pub fn is_hungry(&self) -> bool {
        self.hunger >= 5
    }

    pub fn is_thirsty(&self) -> bool {
        self.thirst >= 5
    }

    fn calculate_stress_level(&self) -> StressLevel {
        let value = self.calculate_stress_value();

        match value {
            0..=5 => StressLevel::Normal,
            5..=10 => StressLevel::Happy,
            10..=15 => StressLevel::Angry,
            15..=20 => StressLevel::Annoyed,
            20..=25 => StressLevel::Stressed,
            _ => StressLevel::Normal,
        }
    }

    fn calculate_stress_value(&self) -> usize {
        ((self.hunger + self.thirst) / 2) as usize
    }


}


#[derive(Bundle)]
pub struct PlayerBundle {
    player : Player, 
    animation_bundle : AnimationBundle,
    player_stats : PlayerStats,
}

impl PlayerBundle {
    pub fn new(animation_bundle : AnimationBundle, speed : f32) -> Self {
        Self {
             player : Player {
                    current_state : PlayerState::Idle,
                    previous_state : PlayerState::Idle,
                    speed
             }, 
             animation_bundle,
             player_stats : PlayerStats::new(),
        }
    }
}