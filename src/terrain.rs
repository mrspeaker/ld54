use std::time::Duration;

use bevy::math::Vec4Swizzles;
use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;
use bevy_ecs_tilemap::prelude::*;
use bevy_kira_audio::prelude::*;
use rand::Rng;
use rand::seq::IteratorRandom;
use rand::seq::SliceRandom;

use crate::AssetCol;
use crate::GameState;
use crate::Layers;
use crate::game::NavmeshPair;
use crate::game::remove_conflicting_paths_on_tile_change;
use crate::game::update_navmesh_on_tile_change;
use crate::game::{OnGameScreen,GameData};
use crate::pathfinding::Navmesh;
use crate::inventory::Inventory;
use crate::pointer::{Pointer, update_pointer};

pub const MAP_COLS: u32 = 23;
pub const MAP_ROWS: u32 = 15;
pub const TILE_SIZE: f32 = 40.0;
pub const GAP_LEFT: f32 = TILE_SIZE * 2.0;
pub const GAP_BOTTOM: f32 = TILE_SIZE * 0.0;

#[derive(Component, Copy, Clone, Debug)]
pub enum Tile {
    Air,
    Dirt { topsoil: bool, style: u8 },
    Rock { style: u8 },
    Stalk { style: u8 },
    Leaves { style: u8 },
    Egg { style: u8 },
    Poo { style: u8 },
    Unknown,
}

impl Tile {
    pub fn texture(&self) -> u32 {
        match self {
            Self::Air => 0,
            Self::Dirt { style, .. } if *style > 3 => 18,
            Self::Dirt { style, .. } => u32::from(*style) + 1,
            Self::Rock { style } => u32::from(*style) + 11,
            Self::Stalk { style } if *style == 2 => 41,
            Self::Stalk { style } => u32::from(*style) + 8,
            Self::Leaves { style } => u32::from(*style) + 7,
            Self::Poo { style } => u32::from(*style) + 48,
            Self::Egg { style } => u32::from(*style) + 64,
            Self::Unknown => 16,
        }
    }
    pub fn from_texture(tex: u32) -> Tile {
        match tex {
            0 => Tile::Air,
            1..=4 => Tile::Dirt {
                topsoil: true,
                style: tex as u8,
            },
            18 => Tile::Dirt {
                topsoil: false,
                style: 5,
            },
            11 => Tile::Rock { style: tex as u8 },
            8 => Tile::Stalk { style: tex as u8 },
            7 => Tile::Leaves { style: tex as u8 },
            48..=56 => Tile::Egg { style: tex as u8 },
            64..=72 => Tile::Poo { style: tex as u8 },
            _ => Tile::Unknown,
        }
    }
    pub fn is_solid(tile: Tile) -> bool {
        match tile {
            Tile::Air => false,
            Tile::Egg { .. } => false,
            _ => true
        }
    }
}

pub fn find_empty_tile(navmesh:&Navmesh, map_size:&TilemapSize) -> Option<TilePos> {
    let mut rng = rand::thread_rng();

    // Find a list of non-slid tiles
    // TODO: This is very inefficient - re-calculated every call.
    let mut free_spots:Vec<(u32,u32)> = vec![];
    let mut target = TilePos { x: 0, y: 0 };
    for j in 0..map_size.y {
        for i in 0..map_size.x {
            target.x = i;
            target.y = j;
            if !navmesh.solid(target) {
                free_spots.push((i, j));
            }
        }
    }

    if let Some(spot) = free_spots.choose(&mut rng) {
        // we got one.
        target.x = spot.0;
        target.y = spot.1;
        return Some(target);
    } else {
        // No free spots. What to do?
        info!("no free spots");
    }
    return None;
}

pub fn tilepos_to_px(tilepos: &TilePos, grid_size: &TilemapGridSize) -> Vec2 {
    Vec2 {
        x: tilepos.x as f32 * grid_size.x + 25.0 + GAP_LEFT,
        y: tilepos.y as f32 * grid_size.y + 25.0
    }
}

pub fn px_to_tilepos(pos: Vec2, grid_size: &TilemapGridSize) -> TilePos {
    TilePos {
        x: (pos.x / grid_size.x as f32) as u32,
        y: (pos.y / grid_size.y as f32) as u32,
    }
}


pub struct TerrainPlugin;
impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilemapPlugin)
            .init_resource::<Pointer>()
            .add_systems(OnEnter(GameState::InGame), terrain_setup)
            .add_systems(First, update_pointer)//).chain())//).chain())
            .add_systems(Update, (
                highlight_tile,
                update_tile,
                update_navmesh_on_tile_change.after(update_tile),
                remove_conflicting_paths_on_tile_change.after(update_tile),
                spawn_plant,
                // tick_tiles
            ).run_if(in_state(GameState::InGame)));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Faction {
    Green,
    Blue,
    Red,
}
impl Faction {
    const FACTIONS: &'static [Self] = &[
        Self::Red,
        Self::Green,
        Self::Red,
    ];
    #[must_use]
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        *Self::FACTIONS.choose(&mut rng).unwrap()
    }
}

#[derive(Debug)]
enum PlantStatus {
    Dead,
    Growing,
    Fruiting,
}

#[derive(Component, Debug)]
pub struct Plant {
    pub ptype: Faction,
    status: PlantStatus,
}

#[derive(Component, Debug)]
pub struct Egg {
    pub faction: Faction,
}

#[derive(Component)]
pub struct FindPos {
    target: TilePos
}

#[derive(Component)]
pub struct TargetEgg;

#[derive(Component)]
struct Topsoil;

#[derive(Component)]
struct Colliding;

#[derive(Component)]
struct TileOffset(u16);

#[derive(Component)]
pub struct Cursor;

#[derive(Component)]
pub struct Terrarium;

#[derive(Resource, Deref, DerefMut)]
struct PlantSpawner(Timer);

#[derive(Component)]
pub struct Health(pub u8);

fn terrain_setup(mut commands: Commands, assets: Res<AssetServer>) {
    let texture = assets.load("img/tiles.png");

    let map_size = TilemapSize {
        x: MAP_COLS,
        y: MAP_ROWS,
    };
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);
    let mut navmesh =  Navmesh::new(map_size.x, map_size.y);
    let mut navmesh_no_dirt = Navmesh::new(map_size.x, map_size.y);

    let mut tiles = Vec::new();

    for y in 0..map_size.y {
        for x in 0..map_size.x {
            let tile_pos = TilePos { x, y };
            let tile = get_tile_from_ascii(tile_pos, map_size);
            let tile_entity = spawn_tile(
                &mut commands,
                tile_pos,
                tile,
                tilemap_entity,
            );
            tile_storage.set(&tile_pos, tile_entity);
            navmesh.set_solid(tile_pos, match tile {
                Tile::Air => false,
                Tile::Egg { .. } => false,
                _ => true
            });
            navmesh_no_dirt.set_solid(tile_pos, match tile {
                Tile::Air => false,
                Tile::Egg { .. } => false,
                Tile::Dirt { .. } => false,
                _ => true
            });
            tiles.push(tile_entity);
        }
    }

    commands.insert_resource(PlantSpawner(
        Timer::new(Duration ::from_secs(6), TimerMode::Repeating),
    ));

    commands.spawn(OnGameScreen)
        .insert(Name::new("Map"))
        .push_children(&tiles);

    let tile_size = TilemapTileSize {
        x: TILE_SIZE,
        y: TILE_SIZE,
    };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert((
        OnGameScreen,
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
                Layers::MIDGROUND - 1.0,
            ),
            ..Default::default()
        },
        TileOffset(1),
        NavmeshPair {
            main: navmesh,
            alt: navmesh_no_dirt
        }
    ));

    commands.spawn((
        Cursor,
        SpriteBundle {
            texture: assets.load("img/cursor.png"),
            transform: Transform::from_xyz(0.0, 0.0, Layers::FOREGROUND),
            ..default()
        },
        OnGameScreen
    ));
}

fn spawn_tile(commands: &mut Commands, position: TilePos, tile: Tile, map_ent: Entity) -> Entity {
    let tbundle = TileBundle {
        position,
        tilemap_id: TilemapId(map_ent),
        texture_index: TileTextureIndex(tile.texture()),
        ..Default::default()
    };
    let health = Health(100);

    match tile {
        Tile::Dirt { topsoil: true, .. } => commands.spawn((tbundle, Topsoil, tile, health)),
        Tile::Stalk { .. } => commands.spawn((
            tbundle,
            Plant {
                ptype: Faction::Red,
                status: PlantStatus::Growing,
            },
            tile,
            health,
        )),
        Tile::Egg { style } => {
            commands.spawn((
                tbundle,
                Egg {
                    faction: if style == 0 { Faction::Red } else { Faction::Blue }
                },
                tile,
                health))
        },
        Tile::Air | Tile::Unknown => commands.spawn((tbundle, tile, health)),
        _ => commands.spawn((tbundle, tile, health)),
    }
    .id()
}

fn get_tile_from_ascii(pos: TilePos, size: TilemapSize) -> Tile {
    let tilemap = b"\
    .......................\
    .1................2....\
    .t................t....\
    .t.......####.....#####\
    #####..................\
    .......................\
    ....##...........#####%\
    .........a..b..........\
    ##.......####..........\
    ..#..............###...\
    ...#...................\
    .......................\
    ###.................###\
    %%%#################%XX\
    XXXXXXXXXXXXXXXXXXXXXXX";

    // TODO: how to do this nicely?
    let sxu: usize = size.x.try_into().unwrap();
    let syu: usize = size.y.try_into().unwrap();
    let xu: usize = pos.x.try_into().unwrap();
    let yu: usize = pos.y.try_into().unwrap();

    let ch = tilemap[((syu - yu) - 1) * sxu + xu];

    match ch {
        b'#' => Tile::Dirt {
            style: 1,
            topsoil: true,
        },
        b'%' => Tile::Dirt {
            style: 5,
            topsoil: false,
        },
        b'X' => Tile::Rock { style: 0 },
        b'1' => Tile::Egg { style: 0 },
        b'2' => Tile::Egg { style: 1 },
        b'a' => Tile::Poo { style: 0 },
        b'b' => Tile::Poo { style: 1 },
        b't' => Tile::Stalk { style: 0 },
        b'L' => Tile::Leaves { style: 1 },
        b'.' => Tile::Air,
        _ => Tile::Unknown,
    }.clone()
}

fn highlight_tile(
    mut commands: Commands,
    mut pointer: ResMut<Pointer>,
    mut cursor: Query<&mut Transform, With<Cursor>>,
    mut tilemap_q: Query<
        (
            &TilemapSize,
            &TilemapGridSize,
            &TilemapType,
            &TileStorage,
            &Transform,
        ),
        Without<Cursor>,
    >,
    mut tile_q: Query<(&mut Tile, &mut Health)>,
    inv: Res<Inventory>, // TODO: not using this anymore
    assets: Res<AssetCol>,
    audio: Res<Audio>,
    game_data: Res<GameData>
) {
    let (map_size, grid_size, map_type, tile_storage, map_transform) = tilemap_q.single_mut();

    let pointer_in_map_pos: Vec2 = {
        let pos = Vec4::from((pointer.pos, 0.0, 1.0));
        (map_transform.compute_matrix().inverse() * pos).xy()
    };

    // Get tile entity and tilepos from pointer pos
    if let Some((tile_entity, tile_pos)) = TilePos::from_world_pos(
        &pointer_in_map_pos,
        map_size,
        grid_size,
        map_type)
        .and_then(|tile_pos| {
            tile_storage.get(&tile_pos)
                .and_then(|ent| Some((ent, tile_pos)))
        })
    {
        // Upate cursor entity to tile position of pointer
        let mut cursor_pos = cursor.single_mut();
        cursor_pos.translation.x = tile_pos.x as f32 * grid_size.x + GAP_LEFT + TILE_SIZE / 2.0;
        cursor_pos.translation.y = tile_pos.y as f32 * grid_size.y + GAP_BOTTOM + TILE_SIZE / 2.0;

        // Don't draw if game over
        if game_data.game_over {
            return;
        }

        if let Ok((mut tile, mut health)) = tile_q.get_mut(tile_entity) {
            // Update the tile texture and pointer
            pointer.set_active_item(*tile);

            if pointer.is_down && tile.texture() != pointer.tile.texture() {
                let (did_draw, _dirts) = draw_tile(&pointer.tile, &tile, inv.dirt);
                if did_draw {
                    // inv.dirt = dirts; TODO: not using inventory system anymore
                    *tile = pointer.tile;
                    health.0 = 100; // Reset tile Health

                    // Play some noise
                    audio.play(assets.blip.clone()).with_volume(0.3);


                    // no good? - can't add same comp twice, will crash
                    match pointer.tile {
                        Tile::Dirt { .. } => {
                            commands.entity(tile_entity).insert(Topsoil);
                        }
                        _ => (),
                    }
                }
            }
        }
    }
}

fn update_tile(
    mut commands: Commands,
    mut tile_query: Query<(Entity, &mut TileTextureIndex, &Tile), Or<(Added<Tile>, Changed<Tile>)>>,
) {
    for (ent, mut tile_texture, tile) in &mut tile_query {
        tile_texture.0 = tile.texture();
        match tile {
            Tile::Dirt { topsoil: false, .. } => {
                commands.entity(ent).remove::<Topsoil>();
            },
            Tile::Dirt { topsoil: true, .. } => {
                commands.entity(ent).insert(Topsoil);
            }
            _ => (),
        };
    }
}


fn tick_tiles(
    mut tile_query: Query<(&mut TileColor, &mut Tile, &mut Health)>,
) {
    if let Some((mut sprite, mut tile, mut health)) = tile_query.iter_mut().choose(&mut rand::thread_rng()) {
        match *tile {
            Tile::Dirt {..} => {
                health.0 -= 10;
                if health.0 > 50 {
                    sprite.0.set_a(health.0 as f32 / 100.);
                } else {
                    *tile = Tile::Air;
                    sprite.0.set_a(1.0);
                    health.0 = 100;
                }
            },
            _ => ()
        }
    }
}

pub fn draw_tile(
    pointer_tile: &Tile,
    tile: &Tile,
    cur_dirts: u32,
) -> (bool, u32) {
    let mut did_draw = false;
    let mut dirts = cur_dirts;
    match (pointer_tile, *tile) {
        // Draw dirt over air
        (Tile::Dirt { .. }, Tile::Air) => {
            if dirts > 0 {
                dirts -= 1;
                did_draw = true;
            }
        }
        // Draw air over dirt
        (Tile::Air, Tile::Dirt {..}) => {
            dirts += 1;
            did_draw = true;
        }
        // No drawing
        _ => {
            did_draw = false;
        }
    }
    (did_draw, dirts)
}


fn spawn_plant(
    mut commands: Commands,
    mut tilemap_query: Query<(&TileStorage, &TilemapSize)>,
    topsoil: Query<(Entity, &TilePos), With<Topsoil>>,
    tile_query: Query<&Tile>,
    time: Res<Time>,
    mut plant_spawner: ResMut<PlantSpawner>,
    game_data: Res<GameData>
) {
    if game_data.game_over {
        return;
    }

    plant_spawner.tick(time.delta());
    if !plant_spawner.finished() {
        return;
    }

    let (tile_storage, map_size) = tilemap_query.single_mut();

    let mut possible_plants: Vec<(Entity, Vec<Entity>)> = vec![];

    for (topsoil_ent, topsoil_pos) in &topsoil {
        let mut pos = *topsoil_pos;
        let mut plant_stack: Vec<Entity> = vec![];
        let mut rng = rand::thread_rng();
        let height = rng.gen_range(1..=3);
        for _ in 1..=height {
            if let Some(newpos) =
                Neighbors::get_square_neighboring_positions(&pos, map_size, false).north
            {
                pos = newpos;
                if let Some(plant_ent) = tile_storage.get(&pos) {
                    if let Ok(tile) = tile_query.get(plant_ent) {
                        match tile {
                            Tile::Air => plant_stack.push(plant_ent),
                            _ => break,
                        }
                    }
                }
            } else {
                break;
            }
        }
        if !plant_stack.is_empty() {
            possible_plants.push((topsoil_ent, plant_stack));
        }
    }

    if let Some((soil_ent, plant_stack)) = possible_plants.choose(&mut rand::thread_rng()) {
        commands.entity(*soil_ent).insert(Tile::Dirt {
            topsoil: false,
            style: 5,
        });
        // Add stalks and egg
        let egg_spot = plant_stack.len() - 1;
        let mut i = 0;
        for plant_ent in plant_stack {
            if i == egg_spot {
                let mut rng = rand::thread_rng();
                let is_blue = rng.gen_bool(0.5);
                commands.entity(*plant_ent).insert((
                    Egg {
                        faction: if is_blue { Faction::Blue } else { Faction::Red }
                    },
                    Tile::Egg { style: if is_blue { 1 } else { 0 }  }
                ));
            } else {
                commands.entity(*plant_ent).insert((
                    Plant {
                        ptype: Faction::Red,
                        status: PlantStatus::Growing,
                    },
                    Tile::Stalk { style: 0 }
                ));
            }
            i+=1;
        }
    }
}
