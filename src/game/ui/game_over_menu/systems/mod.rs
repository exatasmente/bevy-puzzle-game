use bevy::prelude::Resource;

pub mod interactions;
pub mod layout;
pub mod updates;



#[derive(Resource)]
pub struct Pagination {
    pub current_page: usize,
    pub max_page: usize,
    pub items_per_page: usize,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            current_page: 0,
            max_page: 0,
            items_per_page: 5,
        }
    }
    
}

impl Pagination {
    pub fn new(max_page: usize) -> Self {
        Self {
            current_page: 0,
            max_page,
            items_per_page: 5,
        }
    }

    pub fn set_max_page(&mut self, max_page: usize) {
        if  max_page % self.items_per_page != 0 {
            self.max_page = max_page / self.items_per_page + 1;
        } else if max_page <= self.items_per_page {
            self.max_page = 1   
        } else {
            self.max_page = max_page / self.items_per_page;
        }

    }

    pub fn next_page(&mut self) {
        self.current_page = (self.current_page + 1) % self.max_page;
    }

    pub fn previous_page(&mut self) {
        self.current_page = (self.current_page + self.max_page - 1) % self.max_page;
    }

    pub fn set_page(&mut self, page: usize) {
        self.current_page = page;
    }

    pub fn get_page(&self) -> usize {
        self.current_page
    }

    pub fn get_start_index(&self) -> usize {
        self.current_page * self.items_per_page
    }

    pub fn get_end_index(&self) -> usize {
        (self.current_page + 1) * self.items_per_page
    }

    pub fn get_items_per_page(&self) -> usize {
        self.items_per_page
    }
    
}