use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use crate::GameState;

pub struct TerrainPlugin;
impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(TilemapPlugin)
            .add_systems(OnEnter(GameState::InGame), terrain_setup);
    }
}

fn terrain_setup(
    mut commands: Commands,
    assets: Res<AssetServer>
) {
    info!("Info to make the texture appear, lol!"); // If I comment this, tilemap doesn't appear! Timing?
    let texture = assets.load("img/tiles.png");

    let map_size = TilemapSize { x: 32, y: 32 };
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex((x + y) % 5),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture.clone()),
        tile_size,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        ..Default::default()
    });

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
