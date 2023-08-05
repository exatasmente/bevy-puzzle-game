use std::collections::HashMap;

use bevy::{prelude::*, transform};
use rand::prelude::*;


pub const TILE_SIZE : f32 = 64.0;
pub const TOP_WALL_SPRITE_INDEX : usize = 10;

#[derive(Component,Reflect, Default, Clone, Debug, Copy, PartialEq, Eq)]
pub enum TileType {
    #[default]
    Free,
    OutOfBounds,
    Floor,
    Wall,
    Water,
    Grass,
    Lader,
    Door,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TileTypeAdjacency {
    Bottom,
    Top,
    Left,
    Right,
    BottomLeft,
    BottomRight,
    TopLeft,
    TopRight,
}

fn get_adjacency_type(i : usize) -> TileTypeAdjacency {
    match i {
        0 => TileTypeAdjacency::Bottom,
        1 => TileTypeAdjacency::Top,
        2 => TileTypeAdjacency::Left,
        3 => TileTypeAdjacency::Right,
        4 => TileTypeAdjacency::BottomLeft,
        5 => TileTypeAdjacency::BottomRight,
        6 => TileTypeAdjacency::TopLeft,
        7 => TileTypeAdjacency::TopRight,
        _ => TileTypeAdjacency::Bottom,
    }    
}

fn iter_tile_types() -> impl Iterator<Item = &'static TileType> {
    [TileType::Free, TileType::Floor, TileType::Wall, TileType::Water, TileType::Grass, TileType::Lader, TileType::Door].iter()
}

fn iter_adjacency_types() -> impl Iterator<Item = &'static TileTypeAdjacency> {
    [TileTypeAdjacency::Bottom, TileTypeAdjacency::Top, TileTypeAdjacency::Left, TileTypeAdjacency::Right, TileTypeAdjacency::BottomLeft, TileTypeAdjacency::BottomRight, TileTypeAdjacency::TopLeft, TileTypeAdjacency::TopRight].iter()
}

fn get_type_by_index(index : usize) -> TileType {
    match index {
        0 => TileType::Free,
        1 => TileType::Floor,
        2 => TileType::Wall,
        3 => TileType::Water,
        4 => TileType::Grass,
        5 => TileType::Lader,
        6 => TileType::Door,
        _ => TileType::Free,
    }
}

fn get_index_by_type(tile_type : TileType) -> usize {
    match tile_type {
        TileType::Free => 0,
        TileType::Floor => 1,
        TileType::Wall => 2,
        TileType::Water => 3,
        TileType::Grass => 4,
        TileType::Lader => 5,
        TileType::Door => 6,
        _ => 0,
    }
}

#[derive(Component, Clone, Reflect, Default, Debug)]
pub struct MapTile(TileType, usize, usize, usize, Transform, Vec<usize>);

#[derive(Resource)]
pub struct Map {
    pub tiles : Vec<Vec<MapTile>>,
    pub map_width : usize,
    pub map_height : usize,
    pub map_size : usize,
}

struct TileParams {
    tile_type : TileType,
    transform : Transform,
    sprite_index : usize,

}

fn get_wall_sprite_index_by_position(x : usize, y : usize, map_width : usize, map_height : usize) -> (usize, Vec<usize>) {
    
    let mut index = 0;
    
    let is_top_wall = y == map_height - 1;
    let is_top_left_corner = is_top_wall && x == 0;
    let is_top_right_corner = is_top_wall && x == map_width - 1;
    let is_bottom_wall = y == 0;
    let is_bottom_left_corner = is_bottom_wall && x == 0;
    let is_bottom_right_corner = is_bottom_wall && x == map_width - 1;
    let is_left_wall = x == 0;
    let is_right_wall = x == map_width - 1;
    let mut extras = Vec::new();

    if is_top_left_corner {
        index = TOP_WALL_SPRITE_INDEX;
        extras.push(16);
    } else if is_top_right_corner {
        index = TOP_WALL_SPRITE_INDEX;
        extras.push(17);
    } else if is_bottom_left_corner {
        index = 15;
        extras.push(0);
        extras.push(16);
    } else if is_bottom_right_corner {
        index = 15;
        extras.push(0);
        extras.push(17);
    } else if is_top_wall {
        index = TOP_WALL_SPRITE_INDEX;
    } else if is_bottom_wall {
        index = 15;
        extras.push(0);
    } else if is_left_wall {
        index = 16;
        extras.push(0);
    } else if is_right_wall {
        index = 17;
        extras.push(0);
    } else {
        index = TOP_WALL_SPRITE_INDEX;
    }

    (index, extras)
}

impl Map {
    /*[[1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
       [1, 0, 1, 0, 0, 1, 0, 0, 0, 1],
       [1, 0, 0, 0, 0, 1, 0, 0, 0, 1],
       [1, 0, 1, 0, 0, 1, 0, 0, 0, 1],
       [1, 0, 1, 0, 0, 1, 0, 0, 0, 1],
       [1, 0, 1, 0, 1, 1, 1, 0, 1, 1],
       [1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
       [1, 0, 1, 0, 1, 1, 1, 0, 1, 1],
       [1, 0, 1, 0, 0, 1, 0, 0, 0, 1],
       [1, 0, 1, 0, 0, 1, 0, 0, 0, 1],
       [1, 0, 1, 1, 1, 1, 1, 1, 1, 1],
    ]*/

    fn new(map_width : usize, map_height : usize) -> Self {
        let tiles = Vec::new();
        
        let mut map = Self {
            tiles : tiles,
            map_width,
            map_height,
            map_size : map_width * map_height
        };


        for y in 0..map_height {
            let mut row = Vec::new();
            for x in 0..map_width {
                let tile_type = if map.is_edge_tile(x, y) {
                    TileType::Wall
                } else {
                    TileType::Floor
                };
                
                let (sprite_index, extra_sprites) = if tile_type == TileType::Wall {
                    get_wall_sprite_index_by_position(x, y, map.map_width, map.map_height)
                } else {
                    (0, Vec::new())
                };

                let tile = MapTile(tile_type, x, y, sprite_index , Transform::from_translation(Vec3::new(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE, 0.0)), extra_sprites);
                row.push(tile);
            }
            map.tiles.push(row);
        }

        
        map
    }

    pub fn generate_map(&mut self, min_floor_size : usize) {
        let mut  possible_rooms: usize = self.map_width * self.map_height / (min_floor_size * min_floor_size);
        let mut rng = rand::thread_rng();
        
        while possible_rooms > 0 {
            let x = rng.gen_range(0..self.map_width / 2);
            let y = rng.gen_range(0..self.map_height / 2);
            
            let n_of_doors = rng.gen_range(1..4);

            if x + min_floor_size > self.map_width || y + min_floor_size > self.map_height {
                continue;
            }

            self.generate_room(x, y, min_floor_size, min_floor_size, n_of_doors);
            possible_rooms -= 1;
        }
    

    }

    fn handle_bottom_adjacents_tiles(&mut self, x : usize, y : usize, tile_type : TileType) {
        let (adj_x, adj_y) = self.get_adj_x_y(TileTypeAdjacency::Bottom);
        let adj_x = x as f32 + adj_x;
        let adj_y = y as f32 + adj_y;

        match tile_type {   
            TileType::Wall =>  {
                let mut sprite_index = 0;
                let mut extra_sprites = Vec::new();
                match self.get_tile_type(adj_x, adj_y) {
                    TileType::Floor  =>  {
                        sprite_index = TOP_WALL_SPRITE_INDEX;
                    },
                    TileType::OutOfBounds => {
                        let (index, extra) = get_wall_sprite_index_by_position(x as usize, y as usize, self.map_width, self.map_height);
                        sprite_index = index;
                        extra_sprites = extra;
                        
                    },
                    TileType::Wall => {
                        sprite_index = TOP_WALL_SPRITE_INDEX;
                        self.tiles[adj_y as usize][adj_x  as usize] = MapTile(TileType::Floor, adj_x as usize, adj_y as usize, 0 , Transform::from_translation(Vec3::new (adj_x * TILE_SIZE, adj_y* TILE_SIZE, 0.0)), Vec::new())
                    },
                    _ => {}
                }
                
                self.tiles[y][x] = MapTile(TileType::Wall, x, y, sprite_index , Transform::from_translation(Vec3::new(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE, 0.0)), extra_sprites)
            },
            TileType::Floor => match self.get_tile_type(adj_x, adj_y) {
                TileType::OutOfBounds => {
                    let (sprite_index, extra_sprites) = get_wall_sprite_index_by_position(x as usize, y as usize, self.map_width, self.map_height);
                    self.tiles[adj_y as usize][adj_x  as usize] = MapTile(TileType::Wall, adj_x as usize, adj_y as usize, sprite_index , Transform::from_translation(Vec3::new(adj_x as f32 * TILE_SIZE, adj_y as f32 * TILE_SIZE, 0.0)),extra_sprites)
                },
                _ => {}
            }
            _ => {}
        }
    }

    fn generate_room(&mut self, x : usize, y : usize, width : usize, height : usize, n_of_doors : usize) {

        let mut doors_left = n_of_doors;
        println!("Generating room at: {:?} {:?} {:?} {:?}", x, y, width, height);
        for y_j in 0..width {
            for x_i in 0..height {
                let tile_type = if x_i == 0 || x_i == width - 1 || y_j == 0 || y_j == height - 1 {
                    TileType::Wall
                } else {
                    TileType::Floor
                };

                let is_left_wall = x_i == 0;
                let is_right_wall = x_i == width - 1;
                let is_bottom_wall = y_j == 0;
                if is_left_wall {
                    self.tiles[y + y_j][x + x_i] = MapTile(TileType::Wall, x + x_i, y + y_j, 16 , Transform::from_translation(Vec3::new((x + x_i) as f32 * TILE_SIZE, (y + y_j) as f32 * TILE_SIZE, 0.0)), Vec::new())
                } else if is_right_wall {
                    self.tiles[y + y_j][x + x_i] = MapTile(TileType::Wall, x + x_i, y + y_j, 17 , Transform::from_translation(Vec3::new((x + x_i) as f32 * TILE_SIZE, (y + y_j) as f32 * TILE_SIZE, 0.0)), Vec::new())
                } else if is_bottom_wall {
                    self.tiles[y + y_j][x + x_i] = MapTile(TileType::Wall, x + x_i, y + y_j, 15 , Transform::from_translation(Vec3::new((x + x_i) as f32 * TILE_SIZE, (y + y_j) as f32 * TILE_SIZE, 0.0)), Vec::new())
                } else {
                    self.handle_bottom_adjacents_tiles(x + x_i, y + y_j, tile_type);
                }
                
            
            }

        }

    }

    fn from_tiles(tiles : Vec<Vec<MapTile>>) -> Self {
        let map_width = tiles.len();
        let map_height = tiles[0].len();
        let map_size = map_width * map_height;

        Self {
            tiles,
            map_width,
            map_height,
            map_size,
        }
    }

    fn from_list(tiles : Vec<Vec<TileParams>>) -> Self {
        let map_width = tiles.len();
        let map_height = tiles[0].len();
        let map_size = map_width * map_height;

        let mut map = Self {
            tiles : Vec::new(),
            map_width,
            map_height,
            map_size,
        };

        for x in 0..map_width {
            let mut row = Vec::new();
            for y in 0..map_height {
                let tile = MapTile(tiles[x][y].tile_type, x, y, tiles[x][y].sprite_index, tiles[x][y].transform, Vec::new());
                row.push(tile);
            }
            map.tiles.push(row);
        }

        map
    }

    fn get_tile_type(&mut self, x : f32, y : f32) -> TileType {
        let tile = self.get_tile(x, y);

        if tile.is_none() {
            return TileType::OutOfBounds;
        }

        tile.unwrap().get_tile_type()
    }


    fn get_tile(&self, x : f32, y : f32) -> Option<&MapTile> {
        if x < self.map_width as f32 && y < self.map_height as f32 && x >= 0.0 && y >= 0.0 {
            Some(&self.tiles[y as usize][x as usize])
        } else {
            None
        }
    }

    fn get_tile_mut(&mut self, x : f32, y : f32) -> Option<&mut MapTile> {
        println!("Getting tile: {:?} {:?} {:?}, {:?}", x, y, self.map_width, self.map_height);
        if x < self.map_width  as f32 && y < self.map_height as f32 && x >= 0.0 && y >= 0.0 {
            Some(&mut self.tiles[y as usize][x as usize])
        } else {
            None
        }
    }

    fn for_each_tile<F>(&mut self, mut f : F) where F : FnMut(&mut MapTile, i32, i32) {
        let mut x = 0;
        let mut y = 0;
        for row in self.tiles.iter_mut() {
            y = 0;
            for tile in row.iter_mut() {

                f(tile, x, y);

                x += 1;
            }

            y += 1;
        }
    }
    pub fn  is_tile_walkable(&mut self, tile_x : f32, tile_y : f32, x_offset : f32, y_offset : f32) -> bool {
        
        let tile = self.get_tile_mut(((tile_x + x_offset) / TILE_SIZE), ((tile_y + y_offset) / TILE_SIZE));
        
        
        if tile.is_none() {
            return false;
        }

        let tile = tile.unwrap();

        match tile.get_tile_type() {
            TileType::Free => true,
            TileType::Floor => true,
            TileType::Water => true,
            TileType::Grass => true,
            TileType::Lader => true,
            TileType::Wall => {
            
                if tile.get_sprite_index() == TOP_WALL_SPRITE_INDEX {
                    return false;
                }

                if tile.get_extra_sprites().is_empty() {
                    return false;
                }
                

                tile.get_extra_sprites().iter().any(|&x| x == 0)
            },
            TileType::OutOfBounds => false,
            _ => false,
        }
    }

    fn get_adj_x_y(&mut self, adjacency : TileTypeAdjacency) -> (f32, f32) {
        let mut x : f32 = 0.0;
        let mut y : f32 = 0.0;

        match adjacency {
            TileTypeAdjacency::TopLeft => {
                x = -1.0;
                y = 1.0;
            },
            TileTypeAdjacency::Top => {
                x = 0.0;
                y = 1.0;
            },
            TileTypeAdjacency::TopRight => {
                x += 1.0;
                y += 1.0;
            },
            TileTypeAdjacency::Left => {
                x = -1.0;
                y = 0.0;
            },
            TileTypeAdjacency::Right => {
                x = 1.0;
                y = 0.0;
            },
            TileTypeAdjacency::BottomLeft => {
                x = -1.0;
                y = -1.0;
            },
            TileTypeAdjacency::Bottom => {
                x = 0.0;
                y = -1.0;
            },
            TileTypeAdjacency::BottomRight => {
                x = -1.0;
                y = -1.0;
            },
            
            _ => {},
        }
        
        (x, y)
    }

    fn is_edge_tile_adjacency(&mut self, tile_x : usize, tile_y : usize, adjacency : TileTypeAdjacency) -> bool {
        
        let (mut x, mut y) = self.get_adj_x_y(adjacency);
        let adj_x = tile_x as f32 + x;
        let adj_y = tile_y as f32 + y;
        

        self.is_edge_tile(adj_x as usize, adj_y as usize)
    }

    fn get_tile_type_adjacency(&mut self, tile_x : usize, tile_y : usize, adjacency : TileTypeAdjacency) -> TileType {
        let mut tile_type = TileType::Free;

        let (mut x, mut y) = self.get_adj_x_y(adjacency);
        

        x = (tile_x as f32) + x;
        y = (tile_y as f32) + y;

        if x < 0.0 || y < 0.0 || x >= self.map_width as f32 || y >= self.map_height as f32 {
            return TileType::OutOfBounds;
        }

        self.get_tile_type(x, y)

    }

    fn get_tile_type_adjacency_values(&mut self, tile_x : usize, tile_y : usize) -> Vec<TileType> {
        let mut count = Vec::new();

        for tile_type_adjacency in iter_adjacency_types() {
            let tile_type =  self.get_tile_type_adjacency(tile_x, tile_y, *tile_type_adjacency);

            count.push(tile_type);
        }

        count
    }

    fn is_edge_tile(&mut self, tile_x : usize, tile_y : usize) -> bool {
        if tile_x <= 0 || tile_y <= 0 || tile_x >= self.map_width - 1 || tile_y >= self.map_height - 1 {
            
            true
        } else {
            
            false
        }
    }
}

fn is_inside(start_x : f32, start_y : f32, end_x : f32, end_y : f32, x : f32, y : f32) -> bool {
    if x >= start_x && x <= end_x && y >= start_y && y <= end_y {
        return true;
    }

    false
}

fn get_rotation_in_rad(rotation : f32) -> f32 {
    (rotation as f32).to_radians()
}

impl MapTile {
    pub fn new(tile_type : TileType, x : usize, y : usize, sprite_index : usize, transform : Transform, extra_sprites : Vec<usize>) -> Self {
        Self(tile_type, x, y, sprite_index, transform, extra_sprites)
    }

    fn set_tile_type(&mut self, tile_type : TileType, options : (usize, f32)) {
        self.0 = tile_type;
        self.3 = options.0;
        self.4.rotate(Quat::from_rotation_z(options.1.to_radians()));
    }

    fn get_tile_type(&self) -> TileType {
        self.0
    }

    fn get_tile_x(&self) -> usize {
        self.1
    }

    fn get_tile_y(&self) -> usize {
        self.2
    }

    fn get_sprite_index(&self) -> usize {
        self.3
    }

    fn get_transform(&self) -> Transform {
        self.4
    }

    fn set_transform(&mut self, transform : Transform) {
        let z  = self.4.rotation.z;
        self.4 = transform;
        self.4.rotate(Quat::from_rotation_z(z));
    }
    
    fn get_extra_sprites(&mut self) -> Vec<usize> {
        self.5.clone()
    }
}
    
pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(generate_map.in_schedule(OnEnter(crate::AppState::Game)))
            .add_system(despaw_map.in_schedule(OnExit(crate::AppState::Game)))
            .register_type::<MapTile>()
            .register_type::<TileType>();
    }
}

pub struct MapGenerator {
    generation_method : fn(usize, usize)-> Map,

}

impl MapGenerator {
    pub fn new(generation_method : fn(usize, usize)-> Map) -> Self {
        Self {
            generation_method,
        }
    }
    pub fn generate_map(&mut self, map_width : usize, map_height : usize) -> Map {
        let mut map = Map::new(map_width, map_height);
        (self.generation_method)(map_width, map_height)
        
    }
}


fn level_1_map_generation_method(map_width : usize, map_height : usize) -> Map {
    // level 1 is a 30x30 with a alley of 4x15 in the middle, at end of the alley ther is another alley of 4x15 forming a T shape
    // on each side of the alley there is a 10x10 house with a living room, kitchen, backyard. 
    // create the code to generate the map    
    let level_1_file = std::fs::read_to_string("assets/level_1.txt").unwrap();

    let mut x = 0;
    let mut y = 0;

    let mut tiles = Vec::new();

    for line in level_1_file.lines() {
        println!("Line: {:?}", y);
        let mut row = Vec::new();
        for tile in line.split(",") {
            row.push(get_type_by_index(tile.parse::<usize>().unwrap()));
            x+=1;
        }

        tiles.push(row);
        x = 0;
        y+=1;
    }

    let mut map = Map::new(tiles[0].len(), tiles.len());
    x = 0;
    y = 0;

    for y in 0..tiles.len() -1  {
        for x in 0..tiles[y].len() - 1 {
            let tile_type = tiles[y][x];
            let mut transform = Transform::from_translation(Vec3::new(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE, 0.0));
            let mut sprite_index = 0;
            let mut extra_sprites = Vec::new();
            if tile_type == TileType::Wall {
                (sprite_index, extra_sprites) = get_wall_sprite_index_by_position(x, y, map.map_width, map.map_height);
            } else {
                let (adj_top_x, adj_top_y) = map.get_adj_x_y(TileTypeAdjacency::Top);
                let (adj_left_x, adj_left_y) = map.get_adj_x_y(TileTypeAdjacency::Left);
                let adj_top_x = x as f32 + adj_top_x;
                let adj_top_y = y as f32 + adj_top_y;
                let adj_left_x = x as f32 + adj_left_x;
                let adj_left_y = y as f32 + adj_left_y;

                let adj_top_type = map.get_tile_type(adj_top_x, adj_top_y);
                let adj_left_type = map.get_tile_type(adj_left_x, adj_left_y);

            
                if adj_top_type == TileType::Wall && adj_left_type == TileType::Wall {
                    (sprite_index, extra_sprites) = (0, Vec::new());
                } else if adj_top_type == TileType::Wall {                    
                    (sprite_index, extra_sprites) = (20, Vec::new());
                    transform.scale = Vec3::new(1.4, 1.0, 1.0);
                } else if adj_left_type == TileType::Wall {
                    (sprite_index, extra_sprites) = (21, Vec::new());
                    transform.scale = Vec3::new(1.0, 1.3, 1.0);
                } else {
                    (sprite_index, extra_sprites) = (19, Vec::new());
                    transform.scale = Vec3::new(1.25, 1.3, 1.0);
                }
                    
            }
            let tile = MapTile(tile_type, x, y, sprite_index , transform, extra_sprites);
            map.tiles[y][x] = tile;
        }
    }
    map

}

fn generate_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("floors.png");
    let mut texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(TILE_SIZE,TILE_SIZE), 2, 9, None, Some(Vec2::new(0.0, 0.0)));
        texture_atlas.add_texture(Rect::new(1.0, 1.0, 1.0, 1.0));
        texture_atlas.add_texture(Rect::new(16.0, 14.0, 64.0, 64.0));
        texture_atlas.add_texture(Rect::new(16.0, 0.0, 64.0, 64.0));
        texture_atlas.add_texture(Rect::new(0.0, 14.0, 64.0, 64.0));
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let mut map_generator  = MapGenerator::new(level_1_map_generation_method);
    let mut map = map_generator.generate_map(4, 4);

    map.for_each_tile(|tile,x , y| {
        let sprite_index = tile.get_sprite_index();
        commands.spawn((SpriteSheetBundle {
            texture_atlas : texture_atlas_handle.clone(),
            sprite : TextureAtlasSprite::new(sprite_index),
            transform: tile.get_transform(),
            ..Default::default()
        } ,
         tile.clone())).with_children(|parent| {
            let mut z = 0.0;
            for sprite in tile.get_extra_sprites() {
                parent.spawn(SpriteSheetBundle {
                    texture_atlas : texture_atlas_handle.clone(),
                    sprite : TextureAtlasSprite::new(sprite),
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, z)),
                    ..Default::default()
                });
                
            
            }
            });             
    });

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