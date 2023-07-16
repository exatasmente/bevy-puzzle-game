use bevy::prelude::*;

#[derive(Component, Reflect, Default, PartialEq, Clone, Copy, Debug)]
pub enum FoodType {
    #[default]
    Solid,
    Sauce,
    Water,
}

#[derive(Component, Reflect)]
pub struct FoodBowl {
    pub food : usize,
    pub max_food : usize,
    pub food_type : FoodType,
}

impl FoodBowl {
    pub fn new(food : usize, max_food : usize, food_type : FoodType) -> Self {
        Self {
            food,
            max_food,
            food_type,
        }
    }

    pub fn add_food(&mut self, food : usize) {
        self.food += food;
        if self.food > self.max_food {
            self.food = self.max_food;
        }
    }

    pub fn remove_food(&mut self, food : usize) {
        if self.food < food {
            self.food = 0;
        } else {
            self.food -= food;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.food == 0
    }

    pub fn is_full(&self) -> bool {
        self.food == self.max_food
    }

    pub fn is_water(&self) -> bool {
        self.food_type == FoodType::Water
    }

    pub fn get_food(&self) -> usize {
        self.food
    }
}