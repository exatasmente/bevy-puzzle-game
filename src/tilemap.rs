use bevy::prelude::*;

pub const TILE_SIZE : f32 = 80.0;

pub const NO_OF_COLUMNS: usize = 6;
pub const NO_OF_ROWS: usize = 5;
pub const NO_OF_CARDS: usize = NO_OF_COLUMNS * NO_OF_ROWS;
pub const CARD_BACK_SPRITE_INDEX: usize = 0;

const TILE_PATHS : [&str; 1] = [
    "char_tileset.png",
];

#[derive(Component)]
pub struct Card {
    pub id : usize,
    pub is_face_up : bool,
}
#[derive(Component)]
pub struct CardFaceUpState;
#[derive(Component)]
pub struct CardFaceDownState;
#[derive(Component)]
pub struct CardMovingState;
    
#[derive(Resource)]
pub struct TileMapSheet(pub Handle<TextureAtlas>);


pub struct TileMapPlugin;
#[derive(Component)]
struct ColorEntity {
    red: f32,
    green: f32,
    blue: f32,
}

fn mix_colors(colors: &[ColorEntity], percentages: &[f32]) -> Option<ColorEntity> {
    let mut mixed_color = ColorEntity {
        red: 0.0,
        green: 0.0,
        blue: 0.0,
    };

    if colors.len() != percentages.len() {
        return None; // Verifica se as listas têm o mesmo tamanho
    }

    let sum_percentages: f32 = percentages.iter().sum();
    if (sum_percentages - 100.0).abs() > f32::EPSILON {
        return None; // Verifica se a soma das porcentagens é igual a 100
    }

    for (color, &percentage) in colors.iter().zip(percentages) {
        let factor = percentage / 100.0;
        mixed_color.red += (color.red * factor) as f32;
        mixed_color.green += (color.green * factor) as f32;
        mixed_color.blue += (color.blue * factor) as f32;
    }

    Some(mixed_color)
}

impl Plugin for TileMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_tile_maps)
            .add_system(spawn_cards.in_schedule(OnEnter(crate::AppState::Game)))
            .add_system(despawn_cards.in_schedule(OnExit(crate::AppState::Game)));
        
    }
}

pub fn spawn_cards(
    mut commands: Commands,
    ascii: Res<TileMapSheet>,
) {
    
    
    
    let mut cards = Vec::new();
    let mut start_x = -((NO_OF_COLUMNS as f32 * TILE_SIZE) / 2.0) + (TILE_SIZE / 2.0);
    let mut start_y = -((NO_OF_ROWS as f32 * TILE_SIZE) / 2.0) + (TILE_SIZE / 2.0);
    for i in 0..NO_OF_CARDS {
        if i % NO_OF_COLUMNS == 0 {
            start_x = -((NO_OF_COLUMNS as f32 * TILE_SIZE) / 2.0) + (TILE_SIZE / 2.0);
            start_y += TILE_SIZE;
        } else {
            start_x += TILE_SIZE;
        }
        cards.push((SpriteSheetBundle {
            sprite:  get_sprite(i),
            texture_atlas: ascii.0.clone(),
            transform: Transform {
                translation: Vec3::new(start_x, start_y, 0.0),
                ..Default::default()
            },
            ..Default::default()
        }, Card {
            id: i,
            is_face_up: false,
        }, CardFaceDownState));
    }

    commands
        .spawn_batch(cards);
    
}

fn get_sprite(sprite_index: usize) -> TextureAtlasSprite {
    let sprite =  TextureAtlasSprite::new(sprite_index);
    sprite.clone()
}

fn despawn_cards(
    mut commands: Commands,
    query: Query<Entity, With<Card>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn load_tile_maps(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>
) {

    for path in TILE_PATHS.iter() {
        let image = assets.load(path.to_string());
        let atlas =
            TextureAtlas::from_grid(image, Vec2::splat(TILE_SIZE), NO_OF_COLUMNS, NO_OF_ROWS, Some(Vec2 { x: 0.0, y: 0.0 }), Some(Vec2 { x: 0.0, y: 0.0 }));

        let atlas_handle = texture_atlases.add(atlas);
        commands.insert_resource(TileMapSheet(atlas_handle));
    }
}