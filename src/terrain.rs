use bevy::math::Vec4Swizzles;
use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;
use bevy_ecs_tilemap::prelude::*;
use bevy_kira_audio::prelude::*;
use rand::seq::SliceRandom;

use crate::GameState;
use crate::Layers;
use crate::pathfinding::{
    Navmesh,
    update_navmesh_on_tile_change,
    remove_conflicting_paths_on_tile_change,
};
use crate::inventory::Inventory;
use crate::pointer::{Pointer, update_pointer};

pub const MAP_COLS: u32 = 23;
pub const MAP_ROWS: u32 = 15;
pub const TILE_SIZE: f32 = 40.0;
pub const GAP_LEFT: f32 = TILE_SIZE * 2.0;
pub const GAP_BOTTOM: f32 = TILE_SIZE * 0.0;

const MAX_PLANT_HEIGHT: u8 = 3;

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
pub struct TargetEgg;

#[derive(Component)]
struct Topsoil;

#[derive(Component)]
struct Colliding;

#[derive(Component)]
struct TileOffset(u16);

#[derive(Component)]
struct LastUpdate(f64);

#[derive(Component)]
pub struct Cursor;

#[derive(Component)]
pub struct Terrarium;

fn terrain_setup(mut commands: Commands, assets: Res<AssetServer>) {
    let texture = assets.load("img/tiles.png");

    let map_size = TilemapSize {
        x: MAP_COLS,
        y: MAP_ROWS,
    };
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);
    let mut navmesh =  Navmesh::new(map_size.x, map_size.y);

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
            tiles.push(tile_entity);
        }
    }

    commands.spawn_empty()
        .insert(Name::new("Map"))
        .push_children(&tiles);

    let tile_size = TilemapTileSize {
        x: TILE_SIZE,
        y: TILE_SIZE,
    };
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
                Layers::MIDGROUND - 1.0,
            ),
            ..Default::default()
        },
        LastUpdate(0.0),
        TileOffset(1),
        navmesh
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

fn spawn_tile(commands: &mut Commands, position: TilePos, tile: Tile, map_ent: Entity) -> Entity {
    let tbundle = TileBundle {
        position,
        tilemap_id: TilemapId(map_ent),
        texture_index: TileTextureIndex(tile.texture()),
        ..Default::default()
    };
    match tile {
        Tile::Dirt { topsoil: true, .. } => commands.spawn((tbundle, Topsoil, tile)),
        Tile::Stalk { .. } => commands.spawn((
            tbundle,
            Plant {
                ptype: Faction::Red,
                status: PlantStatus::Growing,
            },
            tile,
        )),
        Tile::Egg { style } => {
            commands.spawn((
            tbundle,
                Egg {
                    faction: if style == 0 { Faction::Red } else { Faction::Blue }
                },
                tile))
        },
        Tile::Air | Tile::Unknown => commands.spawn((tbundle, tile)),
        _ => commands.spawn((tbundle, tile)),
    }
    .id()
}

fn get_tile_from_ascii(pos: TilePos, size: TilemapSize) -> Tile {
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
    }
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
    mut tile_q: Query<&mut Tile>, // Every tile? Is that necessary?
    mut inv: ResMut<Inventory>,
    assets: Res<AssetServer>,
    audio: Res<Audio>,
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

        if let Ok(mut tile) = tile_q.get_mut(tile_entity) {
            // Update the tile texture and pointer
            pointer.set_active_item(*tile);

            if pointer.is_down && tile.texture() != pointer.tile.texture() {
                let (did_draw, dirts) = draw_tile(&pointer.tile, &tile, inv.dirt);
                if did_draw {
                    inv.dirt = dirts;
                    *tile = pointer.tile;

                    // Play some noise
                    audio.play(assets.load("sounds/blip.ogg")).with_volume(0.3);

                    // Don't think it can ever get here? We can't draw stalks.
                    match pointer.tile {
                        Tile::Stalk { .. } => {
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
    key_in: Res<Input<KeyCode>>,
    mut tilemap_query: Query<(&TileStorage, &TilemapSize)>,
    query: Query<(Entity, &TilePos), With<Topsoil>>,
    tile_query: Query<&Tile>,
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
                    possible_plants.push((ent, plant_stack));
                }
            }
            if let Some((soil_ent, plant_stack)) = possible_plants.choose(&mut rand::thread_rng()) {
                commands.entity(*soil_ent).insert(Tile::Dirt {
                    topsoil: false,
                    style: 5,
                });
                for plant_ent in plant_stack {
                    commands.entity(*plant_ent).insert((
                        Plant {
                            ptype: Faction::Red,
                            status: PlantStatus::Growing,
                        },
                        Tile::Stalk { style: 0 },
                    ));
                }
            }
        }
    }
}
