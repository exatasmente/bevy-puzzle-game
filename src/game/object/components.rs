use bevy::prelude::*;

#[derive(Component)]
pub struct PlayerCanInteract;

#[derive(Component, Reflect)]
pub struct Object {
    velocity : f32,
    direction : Vec2,
}

impl Object {
    pub fn new(velocity : f32, direction : Vec2 ) -> Self {
        Self {
            velocity,
            direction,
        }
    }

    pub fn get_velocity(&self) -> f32 {
        self.velocity
    }

    pub fn set_velocity(&mut self, velocity : f32) {
        self.velocity = velocity;
    }

    pub fn get_direction(&self) -> Vec2 {
        self.direction
    }

    pub fn set_direction(&mut self, direction : Vec2) {
        self.direction = direction;
    }

    pub fn generate_movement_vector(&self) -> Vec2 {
        let mut movement_vector = Vec2::new(0.0, 0.0);
        
        movement_vector.x = self.direction.x * self.velocity;
        movement_vector.y = self.direction.y * self.velocity;
        

        movement_vector
    }

    pub fn move_object(&mut self, current_position : Vec3) -> Vec3 {
        let movement_vector = self.generate_movement_vector();
        let mut current_position = current_position;

        current_position.x += movement_vector.x;
        current_position.y += movement_vector.y;
        current_position
    }


}
