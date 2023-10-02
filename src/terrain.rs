use bevy::input::{mouse::MouseButtonInput, ButtonState};
use bevy::math::Vec4Swizzles;
use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;
use bevy_ecs_tilemap::prelude::*;
use bevy_kira_audio::prelude::*;
use rand::seq::SliceRandom;

use crate::GameState;
use crate::Layers;

pub const MAP_COLS: u32 = 23;
pub const MAP_ROWS: u32 = 15;
pub const TILE_SIZE: f32 = 40.0;
pub const GAP_LEFT: f32 = TILE_SIZE * 2.0;
pub const GAP_BOTTOM: f32 = TILE_SIZE * 0.0;

const MAX_PLANT_HEIGHT: u8 = 3;

pub struct Tiles;
impl Tiles {
    pub const AIR: u32 = 0;
    pub const DIRT: u32 = 2;
    pub const DIRT2: u32 = 18;
    pub const ROCK: u32 = 11;
    pub const LEAVES: u32 = 7;
    pub const STALK: u32 = 8;
    pub const POO_PINK: u32 = 64;
    pub const EGG_PINK: u32 = 48;
    pub const POO_BLUE: u32 = 65;
    pub const EGG_BLUE: u32 = 49;
}

pub struct TerrainPlugin;
impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilemapPlugin)
            .init_resource::<Pointer>()
            .add_systems(OnEnter(GameState::InGame), terrain_setup)
            .add_systems(First, (update_pointer).chain())
            .add_systems(
                Update,
                (highlight_tile.after(update_pointer), spawn_plant)
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

enum PlantType {
    Green,
    Blue,
    Red,
}

enum PlantStatus {
    Dead,
    Growing,
    Fruiting
}

#[derive(Component)]
struct Plant {
    ptype: PlantType,
    status: PlantStatus
}

#[derive(Component)]
struct Topsoil;

#[derive(Component)]
struct Colliding;

#[derive(Component)]
struct TileOffset(u16);

#[derive(Component)]
struct LastUpdate(f64);

#[derive(Component)]
struct LastTile(TilePos);

#[derive(Component)]
pub struct Cursor;

#[derive(Component)]
pub struct Terrarium;


#[derive(Resource)]
pub struct Pointer {
    pos: Vec2,
    is_down: bool,
    pressed: bool,
    released: bool,
    tile: u32,
}
impl Default for Pointer {
    fn default() -> Self {
        Pointer {
            pos: Vec2::new(-1000.0, -1000.0),
            is_down: false,
            pressed: false,
            released: false,
            tile: 0,
        }
    }
}

fn terrain_setup(mut commands: Commands, assets: Res<AssetServer>) {
    let texture = assets.load("img/tiles.png");

    let map_size = TilemapSize { x: MAP_COLS, y: MAP_ROWS };
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);

    for y in 0..map_size.y {
        for x in 0..map_size.x {
            let tile_pos = TilePos { x, y };
            let mut tile_entity = commands.spawn(TileBundle {
                position: tile_pos,
                tilemap_id: TilemapId(tilemap_entity),
                texture_index: get_tile_idx(x, y, map_size),
                ..Default::default()
            });
            let _ = match y {
                0 => tile_entity.insert(Colliding),
                1 => tile_entity.insert((Colliding, Topsoil)),
                _ => tile_entity.insert(()),
            };
            tile_storage.set(&tile_pos, tile_entity.id());
        }
    }

    let tile_size = TilemapTileSize { x: TILE_SIZE, y: TILE_SIZE };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert((
        Terrarium,
        TilemapBundle {
            grid_size,
            map_type,
            size: map_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(texture.clone()),
            tile_size,
            transform: Transform::from_xyz(
                TILE_SIZE / 2.0 + GAP_LEFT,
                TILE_SIZE / 2.0 + GAP_BOTTOM,
                Layers::MIDGROUND - 1.0
            ),
            ..Default::default()
        },
        LastUpdate(0.0),
        TileOffset(1),
        LastTile(TilePos::new(0, 0)),
    ));

    commands.spawn((
        Cursor,
        SpriteBundle {
            texture: assets.load("img/cursor.png"),
            transform: Transform::from_xyz(0.0, 0.0, Layers::FOREGROUND),
            ..default()
        },
    ));
}

fn get_tile_idx(x: u32, y:u32, size: TilemapSize) -> TileTextureIndex {

    let tilemap = b"\
    .1.....................\
    .t..............L......\
    .t..............t...###\
    ######..........t.##...\
    ................##.....\
    ............2......LL.#\
    ....LLLL....t....#####%\
    .a..........t.b........\
    ##.........####........\
    L.#..............###...\
    ...#...........L.......\
    ...............t.......\
    ###......a.....t....###\
    %%%#################%XX\
    XXXXXXXXXXXXXXXXXXXXXXX";

    // TODO: how to do this nicely?
    let sxu: usize = size.x.try_into().unwrap();
    let syu: usize = size.y.try_into().unwrap();
    let xu: usize = x.try_into().unwrap();
    let yu: usize = y.try_into().unwrap();

    let ch = tilemap[((syu - yu) - 1) * sxu + xu];

    let idx = match ch {
        b'#' => Tiles::DIRT,
        b'%' => Tiles::DIRT2,
        b'X' => Tiles::ROCK,
        b'1' => Tiles::POO_PINK,
        b'a' => Tiles::EGG_PINK,
        b'2' => Tiles::POO_BLUE,
        b'b' => Tiles::EGG_BLUE,
        b't' => Tiles::STALK,
        b'L' => Tiles::LEAVES,
        _ => Tiles::AIR,
    };
    TileTextureIndex(idx as u32)
 }

pub fn update_pointer(
    camera_q: Query<(&GlobalTransform, &Camera)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut pointer: ResMut<Pointer>,
    mut events: EventReader<MouseButtonInput>,
) {
    for cursor_moved in &mut cursor_moved_events {
        for (cam_t, cam) in camera_q.iter() {
            if let Some(pos) = cam.viewport_to_world_2d(cam_t, cursor_moved.position) {
                pointer.pos = pos;
                // TODO: figure out how to set this reliably.
                // Currently calling pointer.pressed = false after handling in highlight-tile
                //pointer.pressed = false;
                //pointer.released = false;
                for ev in &mut events {
                    match ev.state {
                        ButtonState::Pressed => {
                            pointer.is_down = true;
                            pointer.pressed = true;
                        }
                        ButtonState::Released => {
                            pointer.is_down = false;
                            pointer.released = true;
                        }
                    }
                }
            }
        }
    }
}

fn highlight_tile(
    mut commands: Commands,
    mut pointer: ResMut<Pointer>,
    mut tilemap_q: Query<
        (
            &TilemapSize,
            &TilemapGridSize,
            &TilemapType,
            &TileStorage,
            &Transform,
            &mut LastTile,
        ),
        Without<Cursor>,
    >,
    mut tile_q: Query<&mut TileTextureIndex>,
    mut cursor: Query<&mut Transform, With<Cursor>>,
    assets: Res<AssetServer>,
    audio: Res<Audio>,
) {
    for (map_size, grid_size, map_type, tile_storage, map_transform, mut last_tile) in
        &mut tilemap_q
    {
        let cursor_in_map_pos: Vec2 = {
            // Extend the cursor_pos vec3 by 0.0 and 1.0
            let pos = Vec4::from((pointer.pos, 0.0, 1.0));
            let cursor_in_map_pos = map_transform.compute_matrix().inverse() * pos;
            cursor_in_map_pos.xy()
        };

        // world position to tile position.
        if let Some(tile_pos) =
            TilePos::from_world_pos(&cursor_in_map_pos, map_size, grid_size, map_type)
        {
            let mut cursor_pos = cursor.single_mut();
            cursor_pos.translation.x = tile_pos.x as f32 * grid_size.x + GAP_LEFT + TILE_SIZE / 2.0;
            cursor_pos.translation.y = tile_pos.y as f32 * grid_size.y + GAP_BOTTOM + TILE_SIZE / 2.0;

            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                let is_same = tile_pos.x != last_tile.0.x || tile_pos.y != last_tile.0.y;
                if !is_same {
                    last_tile.0 = tile_pos;
                }

                if let Ok(mut t) = tile_q.get_mut(tile_entity) {
                    if pointer.pressed {
                        pointer.tile = match t.0 {
                            Tiles::AIR => Tiles::DIRT,
                            Tiles::ROCK => Tiles::ROCK,
                            _ => Tiles::AIR
                        } as u32;
                        pointer.pressed = false;
                        pointer.is_down = pointer.tile != Tiles::ROCK;
                    }

                    if pointer.is_down && t.0 != pointer.tile {
                        t.0 = pointer.tile;
                        audio.play(assets.load("sounds/blip.ogg")).with_volume(0.3);
                        if pointer.tile == Tiles::LEAVES {
                            commands.entity(tile_entity).insert((Colliding, Topsoil));
                        }
                    }
                }
            }
        }
    }
}

fn spawn_plant(
    mut commands: Commands,
    key_in: Res<Input<KeyCode>>,
    mut tilemap_query: Query<(&TileStorage, &TilemapSize)>,
    query: Query<(Entity, &TilePos), With<Topsoil>>,
    mut tile_query: Query<&mut TileTextureIndex>,
) {
    for (tile_storage, map_size) in &mut tilemap_query {
        if key_in.pressed(KeyCode::Space) {
            let mut possible_plants: Vec<(Entity, Vec<Entity>)> = vec![];
            for (ent, pos) in &query {
                let mut pos = *pos;
                let mut plant_stack: Vec<Entity> = vec![];
                for _iter in 1..=MAX_PLANT_HEIGHT {
                    if let Some(newpos) =
                        Neighbors::get_square_neighboring_positions(&pos, map_size, false).north
                    {
                        pos = newpos;
                        if let Some(plant_ent) = tile_storage.get(&pos) {
                            if let Ok(tile_texture) = tile_query.get_mut(plant_ent) {
                                if tile_texture.0 == Tiles::AIR as u32 {
                                    plant_stack.push(plant_ent);
                                } else {
                                    break;
                                }
                            }
                        }
                    } else {
                        break;
                    }
                }
                if !plant_stack.is_empty() {
                    possible_plants.push((ent, plant_stack));
                }
            }
            if let Some((soil_ent, plant_stack)) = possible_plants.choose(&mut rand::thread_rng()) {
                commands.entity(*soil_ent).remove::<Topsoil>();
                for plant_ent in plant_stack {
                    commands.entity(*plant_ent).insert(Plant {
                        ptype: PlantType::Red,
                        status: PlantStatus::Growing
                    });
                    if let Ok(mut tile_texture) = tile_query.get_mut(*plant_ent) {
                        tile_texture.0 = Tiles::STALK as u32;
                    }
                }
            }
        }
    }
}
