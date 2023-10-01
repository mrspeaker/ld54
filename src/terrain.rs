use bevy::input::{mouse::MouseButtonInput, ButtonState};
use bevy::math::Vec4Swizzles;
use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;
use bevy_ecs_tilemap::prelude::*;
use bevy_kira_audio::prelude::*;
use rand::seq::SliceRandom;

use crate::GameState;

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
            ); // Is this running even in non-game state?
    }
}

const MAX_PLANT_HEIGHT: u8 = 3;

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
pub struct Cursora;

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

    let map_size = TilemapSize { x: 25, y: 15 };
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

    let tile_size = TilemapTileSize { x: 40.0, y: 40.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert((
        TilemapBundle {
            grid_size,
            map_type,
            size: map_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(texture.clone()),
            tile_size,
            transform: Transform::from_xyz(tile_size.x / 2.0, tile_size.y / 2.0, 0.5),
            ..Default::default()
        },
        LastUpdate(0.0),
        TileOffset(1),
        LastTile(TilePos::new(0, 0)),
    ));

    commands.spawn((
        SpriteBundle {
            texture: assets.load("img/cursor.png"),
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            ..default()
        },
        Cursora,
    ));
}

fn get_tile_idx(x: u32, y:u32, size: TilemapSize) -> TileTextureIndex {

    let tilemap = b"\
    .........................\
    .........................\
    ......................###\
    ########............##...\
    ..................##.....\
    ........................#\
    ......####.........######\
    .........................\
    ####.........####........\
    ....#..............###...\
    .....#...................\
    .........................\
    X.........XXXX...........\
    XX#######XXX########..#XX\
    XXXXXXXXXXXXXXXXXXXXXXXXX";

    let sxu: usize = size.x.try_into().unwrap();
    let syu: usize = size.y.try_into().unwrap();
    let xu: usize = x.try_into().unwrap();
    let yu: usize = y.try_into().unwrap();
    info!("{} {} {}", xu, yu, sxu);

    let ch = tilemap[((syu - yu) - 1) * sxu + xu];

    let idx = match ch {
        b'#' => 18,
        b'X' => 16,
        _ => 0,
    };
    TileTextureIndex(idx)
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
        Without<Cursora>,
    >,
    mut tile_q: Query<&mut TileTextureIndex>,
    mut cursor: Query<&mut Transform, With<Cursora>>,
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
            cursor_pos.translation.x = tile_pos.x as f32 * grid_size.x + 20.0;
            cursor_pos.translation.y = tile_pos.y as f32 * grid_size.y + 20.0;

            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                let is_same = tile_pos.x != last_tile.0.x || tile_pos.y != last_tile.0.y;
                if !is_same {
                    last_tile.0 = tile_pos;
                }

                if let Ok(mut t) = tile_q.get_mut(tile_entity) {
                    if pointer.pressed {
                        pointer.tile = if t.0 == 0 { 7 } else { 0 };
                        pointer.pressed = false;
                    }

                    if pointer.is_down && t.0 != pointer.tile {
                        t.0 = pointer.tile;
                        audio.play(assets.load("sounds/blip.ogg")).with_volume(0.3);
                        if pointer.tile == 7 {
                            commands.entity(tile_entity).insert((Colliding, Topsoil));
                        }
                    }
                }
            }
        }
    }
}

// Example of getting/modifying neighbours:
// let neighboring_entities = Neighbors::get_square_neighboring_positions(

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
                                if tile_texture.0 == 0 {
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
                        tile_texture.0 = 8;
                    }
                }
            }
        }
    }
}

/*
Old tilemap for posterity

use bevy::sprite::MaterialMesh2dBundle;
//    mut meshes: ResMut<Assets<Mesh>>,
//    mut materials: ResMut<Assets<ColorMaterial>>,

    pub fn color_from_char(byte: u8) -> Color {
        let c = byte as char;
        match c {
            '#' => Color::GRAY,
            '.' => Color::BLUE,
            _ => panic!("Bad char in tilemap"),
        }
    }

    const TILE_SIZE: f32 = 20.0;

    //            1         2         3         4         5         6         7
    //  01234567890123456789012345678901234567890123456789012345678901234567890123456789
    let tilemap = b"\
        ................................................................................\
        ................................................................................\
        ................................................................................\
        ................................................................................\
        ................................................................................\
        ########........................................................................\
        ##############..................................................................\
        .........#####..................................................................\
        ................................................................................\
        ................................................................................\
        .....................................................................###########\
        ......................................................................##########\
        ..........................................................#.....................\
        ..........................................................##....................\
        ################..........................................###....###############\
        ##################....................................##########################\
        #####################....###....######.............###.#########################\
        ################################################################################\
        ################################################################################\
        ################################################################################";

    let tile_x_count = 80;
    let tile_y_count = tilemap.len() / tile_x_count;

    for y in 0..tile_y_count {
        for x in 0..tile_x_count {

            let tile_centre_offset = Vec3::new(TILE_SIZE, TILE_SIZE, 0.0) / 2.0;
            let pos = Vec3::new(x as f32, y as f32, 0.0) * TILE_SIZE + tile_centre_offset;

            let tilemap_index = (((tile_y_count - 1) - y) * tile_x_count) + x;
            let color = color_from_char(tilemap[tilemap_index]);

            /*commands.spawn(MaterialMesh2Bdundle {
                mesh: meshes
                    .add(shape::Quad::new(Vec2::new(TILE_SIZE, TILE_SIZE)).into())
                    .into(),
                material: materials.add(ColorMaterial::from(color)),
                transform: Transform::from_translation(pos),
                ..default()
            });*/
        }
    }
  */
