use bevy::prelude::Resource;


#[derive(Resource)]
pub struct Pagination {
    pub current_page: usize,
    pub max_page: usize,
    pub items_per_page: usize,
    pub entity: Option<bevy::prelude::Entity>,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            current_page: 0,
            max_page: 0,
            // Four rows plus the title, pager and two action buttons is what
            // fits on a 480px-tall phone without clipping.
            items_per_page: 4,
            entity: None,
        }
    }
    
}

impl Pagination {
    pub fn set_max_page(&mut self, max_page: usize) {
        if  max_page % self.items_per_page != 0 {
            self.max_page = max_page / self.items_per_page + 1;
        } else if max_page <= self.items_per_page {
            self.max_page = 1   
        } else {
            self.max_page = max_page / self.items_per_page;
        }

    }
    pub fn set_page(&mut self, page: usize) {
        self.current_page = page;
    }


    pub fn get_start_index(&self) -> usize {
        self.current_page * self.items_per_page
    }


    pub fn get_items_per_page(&self) -> usize {
        self.items_per_page
    }
    
    pub fn set_entity(&mut self, entity: bevy::prelude::Entity) {
        self.entity = Some(entity);
    }

    pub fn get_entity(&self) -> Option<bevy::prelude::Entity> {
        self.entity
    }

    pub fn clear_entity(&mut self) {
        self.entity = None;
    }

    /// Restores the initial state. Defined in terms of `Default` so the page
    /// size cannot drift away from it.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}