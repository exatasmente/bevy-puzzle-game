use std::{hash::Hash};

use bevy::prelude::*;
use crate::mouse_motion::Draggable;

#[derive(Component,Reflect, Default, Clone, Copy)]
pub enum TileType {
    #[default]
    Grass,
    Water,
    Dirt,
    
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum MapTileAdjacency {
    TopLeft,
    Top,
    TopRight,
    Left,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

#[derive(Component, Clone, Copy, Reflect, Default)]
pub struct MapTile(TileType);

#[derive(Resource)]
pub struct Map {
    pub tiles : Vec<Vec<MapTile>>,
    pub map_width : usize,
    pub map_height : usize,
    pub map_size : usize,
}

impl Map {

    fn new(map_width : usize, map_height : usize) -> Self {
        let tiles = Vec::new();
        
        let mut map = Self {
            tiles : tiles,
            map_width,
            map_height,
            map_size : map_width * map_height,
        };

        map.generate_map();

        map
    }

    fn generate_map(&mut self) {

        let mut current_index = 0;
        let mut current_row = Vec::new();

        while current_index < self.map_size {
            if current_index % self.map_width == 0 && current_index != 0 {
                self.tiles.push(current_row);
                current_row = Vec::new();
            }

            current_row.push(MapTile::new(self.get_tile_type(current_index % self.map_width, current_index / self.map_width)));
            current_index += 1;
        }

        self.tiles.push(current_row);

    }

    fn get_tile_type(&self, x : usize, _y : usize) -> TileType {
        
        if x <= 4 {
            return TileType::Water;
        }

        if x > 4 && x < 8 {
            return TileType::Dirt;
        }

        return TileType::Grass;
        
    }


    fn get_tile(&self, x : usize, y : usize) -> Option<&MapTile> {
        if x < self.map_width && y < self.map_height {
            Some(&self.tiles[y][x])
        } else {
            None
        }
    }

    fn get_tile_mut(&mut self, x : usize, y : usize) -> Option<&mut MapTile> {
        if x < self.map_width && y < self.map_height {
            Some(&mut self.tiles[y][x])
        } else {
            None
        }
    }

    fn set_tile(&mut self, x : usize, y : usize, tile_type : TileType) {
        if x < self.map_width && y < self.map_height {
            self.tiles[y][x].set_tile_type(tile_type);
        }
    }

    fn for_each_tile<F>(&mut self, mut f : F) where F : FnMut(&mut MapTile, i32, i32) {
        let mut i = 0;
        let mut j = 0;
        for row in self.tiles.iter_mut() {
            j = 0;
            for tile in row.iter_mut() {

                f(tile, i, j);

                j += 1;
            }

            i += 1;
        }
    }

    
}

impl MapTile {
    pub fn new(tile_type : TileType) -> Self {
        Self(tile_type)
    }

    fn set_tile_type(&mut self, tile_type : TileType) {
        self.0 = tile_type;
    }

    fn get_tile_type(&self) -> TileType {
        self.0
    }
}
    
pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(generate_map.in_schedule(OnEnter(crate::AppState::Game)))
            .add_system(despaw_map.in_schedule(OnExit(crate::AppState::Game)));
    }
}

fn generate_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    _texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("3000.png");
    let house_texture_handle = asset_server.load("2.png");
    let map = Map::new(20, 10);
    commands.spawn((
        SpriteBundle {
            texture : texture_handle.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..default()
        },
    ));

    commands.spawn((
        SpriteBundle {
            texture : house_texture_handle.clone(),
            transform: Transform::from_translation(Vec3::new(-585.0, -78.0, 1.0)),
            ..default()
        },
        Draggable,
    ));
    commands.insert_resource(map);

    

}

fn despaw_map(
    _commands: Commands,
    _map_res : Res<Map>,
) {
    for _i in 0..16 {
        for _j in 0..16 {

        }
    }
}