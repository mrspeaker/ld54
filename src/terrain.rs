use bevy::prelude::*;
use bevy::math::Vec4Swizzles;
use bevy_debug_text_overlay::screen_print;
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;

use crate::GameState;

pub struct TerrainPlugin;
impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(TilemapPlugin)
            .init_resource::<CursorPos>()
            .add_systems(OnEnter(GameState::InGame), terrain_setup)
            //.add_systems(Update, update_map.run_if(in_state(GameState::InGame)))
            .add_systems(First, (update_cursor_pos).chain())
            .add_systems(Update, highlight_tile);
    }
}

#[derive(Component)]
struct TileOffset(u16);

#[derive(Component)]
struct LastUpdate(f64);

fn terrain_setup(
    mut commands: Commands,
    assets: Res<AssetServer>
) {
    info!("Info to make the texture appear, lol!"); // If I comment this, tilemap doesn't appear! Timing?
    let texture = assets.load("img/tiles.png");

    let map_size = TilemapSize { x: 30, y: 20 };
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(0),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let tile_size = TilemapTileSize { x: 32.0, y: 32.0 };
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
            transform: Transform::from_xyz(tile_size.x / 2.0, 0.0,0.0),//get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
            ..Default::default()
        },
        LastUpdate(0.0),
        TileOffset(1)
    ));

}


fn update_map(
    time: Res<Time>,
    mut tilemap_query: Query<(
        &mut TileOffset,
        &mut LastUpdate,
        &TileStorage,
        &TilemapSize,
    )>,
    mut tile_query: Query<&mut TileTextureIndex>,
) {
    let current_time = time.elapsed_seconds_f64();
    for (mut offset_idx, mut last_update, tile_storage, map_size) in tilemap_query.iter_mut() {
        if current_time - last_update.0 > 0.1 {
            offset_idx.0 += 1;
            if offset_idx.0 > 5 {
                offset_idx.0 = 1;
            }

            let mut idx = offset_idx.0;

            for x in (2..20).step_by(4) {
                for y in (2..20).step_by(4) {
                    // Grab the neighboring tiles
                    let neighboring_entities = Neighbors::get_square_neighboring_positions(
                        &TilePos { x, y },
                        map_size,
                        true,
                    )
                    .entities(tile_storage);

                    // Iterate over neighbors
                    for neighbor_entity in neighboring_entities.iter() {
                        // Query the tile entities to change the colors
                        if let Ok(mut tile_texture) = tile_query.get_mut(*neighbor_entity) {
                            tile_texture.0 = idx as u32;
                        }
                    }
                }
                idx += 1;
                if idx > 5 {
                    idx = 1;
                }
            }
            last_update.0 = current_time;
        }
    }
}

#[derive(Resource)]
pub struct CursorPos(Vec2);
impl Default for CursorPos {
    fn default() -> Self {
        Self(Vec2::new(-1000.0, -1000.0))
    }
}

pub fn update_cursor_pos(
    camera_q: Query<(&GlobalTransform, &Camera)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_pos: ResMut<CursorPos>,
) {
    for cursor_moved in cursor_moved_events.iter() {
        for (cam_t, cam) in camera_q.iter() {
            if let Some(pos) = cam.viewport_to_world_2d(cam_t, cursor_moved.position) {
                *cursor_pos = CursorPos(pos);
            }
        }
    }
}


fn highlight_tile(
    mut commands: Commands,
    cursor_pos: Res<CursorPos>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &TileStorage,
        &Transform,
    )>,
    mut tile_q: Query<&mut TileTextureIndex>,
) {
    for (map_size, grid_size,
         map_type, tile_storage,
         map_transform) in tilemap_q.iter() {
        let cursor_pos: Vec2 = cursor_pos.0;
        let cursor_in_map_pos: Vec2 = {
            // Extend the cursor_pos vec3 by 0.0 and 1.0
            let cursor_pos = Vec4::from((cursor_pos, 0.0, 1.0));
            let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
            cursor_in_map_pos.xy()
        };
        // Once we have a world position we can transform it into a possible tile position.
        if let Some(tile_pos) =
            TilePos::from_world_pos(&cursor_in_map_pos, map_size, grid_size, map_type)
        {
            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                screen_print!("Tile: {:?}", tile_entity);

                //let r = tilemap_q.get_mut(tile_entity);
                if let Ok(mut t) = tile_q.get_mut(tile_entity) {
                    t.0 = if t.0 == 5 { 1 } else { 5 };
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

            /*commands.spawn(MaterialMesh2dBundle {
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
