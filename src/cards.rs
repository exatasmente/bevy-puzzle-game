use bevy::prelude::*;
#[derive(Component)]
pub struct Card {
    pub id : usize,
    naipe: String,
    value: String,
}

enum Naipe {
    Copas,
    Espadas,
    Ouros,
    Paus,
}

