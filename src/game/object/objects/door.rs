use bevy::prelude::*;

#[derive(Component, Reflect, Default, PartialEq, Clone, Copy, Debug)]
pub enum DoorState {
    Open,
    #[default]
    Closed,
    Locked,
    Unlocked,
}

#[derive(Component, Reflect)]

pub struct Door {
    state : DoorState,
    key : usize,
}

impl Door {
    pub fn new(state : DoorState, key : usize) -> Self {
        Self {
            state,
            key,
        }
    }

    pub fn set_key(&mut self, key : usize) {
        self.key = key;
    }

    pub fn open(&mut self) {
        self.state = DoorState::Open;
    }

    pub fn close(&mut self) {
        self.state = DoorState::Closed;
    }

    pub fn lock(&mut self, key : usize) {
        if self.key != key {
            return;
        }

        self.state = DoorState::Locked;
    }

    pub fn unlock(&mut self, key : usize) {
        if self.key != key {
            return;
        }

        self.state = DoorState::Unlocked;
    }

    pub fn is_open(&self) -> bool {
        self.state == DoorState::Open
    }

    pub fn is_closed(&self) -> bool {
        self.state == DoorState::Closed
    }

    pub fn is_locked(&self) -> bool {
        self.state == DoorState::Locked
    }

    pub fn is_unlocked(&self) -> bool {
        self.state == DoorState::Unlocked
    }


}