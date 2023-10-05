use crate::game::{OnGameScreen, Speed, Bob, Displacement};
use crate::pathfinding::FollowPath;
use crate::terrain::{GAP_LEFT, TILE_SIZE};
use crate::{prelude::*, GameState};
use bevy::math::swizzles::Vec3Swizzles;
use bevy::{prelude::*, window::PrimaryWindow};
use rand::Rng;

use crate::Layers;

const RUMBLEBEE_SPEED: f32 = 50.0;

pub struct RumblebeePlugin;
impl Plugin for RumblebeePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::InGame), rumblebee_setup)
            .add_systems(Update, set_unassigned_bees);
    }
}

#[derive(Component)]
pub struct RumbleBee {
    pub faction: terrain::Faction,
}

#[derive(Component)]
struct Beenitialized;

fn rumblebee_setup(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
){
    let window: &Window = window_query.get_single().unwrap();

    // Make the beez
    let num_beez = 10;
    for i in 0..num_beez {
        let pos = TilePos { x: 0, y : 0 };
        let bee_pos = Vec3::new(
            pos.x as f32 * TILE_SIZE + GAP_LEFT,//rng.gen_range(0.0..=1.0) * (window.width() - GAP_LEFT) + GAP_LEFT,
            pos.y as f32 * TILE_SIZE, //rng.gen_range(0.0..=1.0) * (window.height() - TILE_SIZE) + TILE_SIZE,
            Layers::MIDGROUND,
        );

        let texture = asset_server.load(if i < num_beez / 2 {
            "img/Creatures/Torsos/pink-torso.png"
        } else {
            "img/Creatures/Torsos/blue-torso.png"
        });

        let torso = SpriteBundle {
            texture,
            transform: Transform::from_translation(bee_pos),
            sprite: Sprite {
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..default()
            },
            ..default()
        };

        let arm = commands.spawn(SpriteBundle {
            texture: asset_server.load("img/Creatures/Arms/bent-punch.png"),
            transform: Transform::from_xyz(0.,0., Layers::MIDGROUND + 0.1),
            sprite: Sprite {
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..default()
            },
            ..default()
        }).id();

        let bee = commands.spawn((
            RumbleBee {
                faction: terrain::Faction::random(),
            },
            torso,
            OnGameScreen,
            FollowPath {
                end: bee_pos.xy(),
                done: true,
            },
            Speed { speed: RUMBLEBEE_SPEED },
            Bob,
            Displacement(Vec2 { x: 0., y: 0. }),
            Beenitialized
        )).id();

        commands.entity(bee).push_children(&[arm]);




        /*
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,

    let texture_handle = asset_server.load("img/gabe-idle-run.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 7, 1, None, None);

    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    // Use only the subset of sprites in the sheet that make up the run animation
    let animation_indices = AnimationIndices { first: 1, last: 6 };


        commands.spawn((
            SpriteSheetBundle {
                texture_atlas: texture_atlas_handle,
                sprite: TextureAtlasSprite::new(animation_indices.first),
                transform: Transform::from_xyz(window.width() / 2.0, 100.0, 0.1)
                    .with_scale(Vec3::splat(6.0)),
                ..default()
            },
            animation_indices,
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        ));
        */


    }

}

fn set_unassigned_bees(
    mut commands: Commands,
    mut ent: Query<(Entity, &mut Transform), (With<RumbleBee>, With<Beenitialized>)>,
    tilemap: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &TileStorage,
        &Navmesh,
        &Transform,
    ), Without<RumbleBee>>,
){
    //let window: &Window = window_query.get_single().unwrap();

    if !tilemap.is_empty() {
        let (map_size, grid_size, map_type, storage, navmesh, map_transform) = tilemap.single();
        for (ent, mut transform) in ent.iter_mut() {
            let mut target = TilePos { x: 0, y: 0 };
            let mut rng = rand::thread_rng();
            let mut ok = false;
            while !ok {
                target.x = rng.gen_range(0..map_size.x);
                target.y = rng.gen_range(0..map_size.y);
                ok = !navmesh.solid(target);
            }
            transform.translation.x = target.x as f32 * grid_size.x - 25.0 + GAP_LEFT;
            transform.translation.y = target.y as f32 * grid_size.y - 25.0;

            commands.entity(ent).remove::<Beenitialized>();
        }
    }
}
