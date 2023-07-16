use bevy::prelude::*;
use bevy_utils::Duration;

#[derive(Component,Reflect, Default)]
pub struct AnimationController {
    current_index : usize,
    frame_indices : Vec<usize>,
    first_run : bool,
    pub animation_timer : Timer,
}

#[derive(Bundle)]
pub struct AnimationBundle {
    sprite_sheet_bundle : SpriteSheetBundle, 
    animation_controller : AnimationController
}

impl AnimationController {
    fn new(frame_indices : Vec<usize>, timer : Timer, current_index : Option<usize>) -> Self {
        let mut instance = Self {
            frame_indices : Vec::new(),
            current_index : match current_index {
                Some(current_index) => current_index,
                None => frame_indices[0],
            },
            first_run : true,
            animation_timer : timer,
        };

        instance.set_values(frame_indices[0], frame_indices[frame_indices.len() - 1]);

        instance
    }

    fn get_first(&self) -> usize {
        self.frame_indices[0]
    }
    fn is_first_run(&self) -> bool {
        self.first_run
    }

    fn set_first_run(&mut self, first_run : bool) {
        self.first_run = first_run;
    }

    fn get_last(&self) -> usize {
        self.frame_indices[self.frame_indices.len() - 1]
    }

    fn on_loop(&mut self, delta : Duration) -> usize {
        self.animation_timer.tick(delta);
        
        if !self.animation_timer.just_finished(){
            if self.current_index <= self.frame_indices.len() - 1 {
                return self.frame_indices[self.current_index];
            }
            
            return self.get_last();
        }

        if self.is_first_run() {
            self.set_first_run(false);
            return self.frame_indices[0];
        }
        
        self.get_next_frame()
    }

    pub fn set_values(&mut self, first : usize, last : usize) {
        self.set_frame_indices((first..=last).collect());
    }

    fn get_current_index(&self) -> usize {
        self.current_index
    }

    fn get_next_frame(&mut self) -> usize {
        self.current_index += 1;

        
        if self.current_index > self.frame_indices.len() - 1 {
            let is_once = self.animation_timer.mode() == TimerMode::Once;
            self.current_index = 0;
            if is_once {
                return usize::MAX;
            }
    
            return self.get_first();
        }

        
        
        return self.frame_indices[self.current_index];

    }

    pub fn set_frame_indices(&mut self, frame_indices : Vec<usize>) {
        self.frame_indices = frame_indices;
        self.reset();
    }

    pub fn reset(&mut self) {
        self.set_first_run(true);
        self.current_index = 0;
        self.animation_timer.reset();
    }
       
}

impl AnimationBundle {
    pub fn new(
        texture_atlas_handle : Handle<TextureAtlas>,
        transform : Transform,
        frame_indices : Vec<usize>, 
        timer : Timer, 
        current_index : Option<usize>
    ) -> Self {
        
        let animation_controller = AnimationController::new(frame_indices, timer, current_index);


        Self {
            sprite_sheet_bundle : SpriteSheetBundle {
                texture_atlas : texture_atlas_handle,
                transform : transform,
                sprite : TextureAtlasSprite::new(animation_controller.get_current_index()),
                ..Default::default()
            }, 
            animation_controller
        }
    }
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(animate_sprite.run_if(in_state(crate::AppState::Game)));
    }
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &mut AnimationController,
        &mut TextureAtlasSprite,
    )>,
) {
    for (mut indices, mut sprite) in &mut query {
        let index = indices.on_loop(time.delta());
        if index == usize::MAX {   
            continue;
        }

        sprite.index = index;
    }
}