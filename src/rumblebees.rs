use crate::game::{OnGameScreen, Speed, Bob, Displacement, AnimationTimer, AnimationIndices};
use crate::pathfinding::FollowPath;
use crate::terrain::{GAP_LEFT, TILE_SIZE};
use crate::{prelude::*, GameState};
use bevy::math::swizzles::Vec3Swizzles;
use bevy::prelude::*;
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
    asset_server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
){
    // Make the beez
    let num_beez = 10;
    for i in 0..num_beez {
        let pos = TilePos { x: 0, y : 0 };
        let bee_z = Layers::MIDGROUND + i as f32;
        let bee_pos = Vec3::new(
            pos.x as f32 * TILE_SIZE + GAP_LEFT,
            pos.y as f32 * TILE_SIZE,
            bee_z,
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

        let arm = commands.spawn(SpriteBundle {
            texture: asset_server.load("img/Creatures/Arms/bent-arm.png"),
            transform: Transform::from_xyz(0.,2., 0.01),
            sprite: Sprite {
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..default()
            },
            ..default()
        }).id();

        let eyes = commands.spawn(SpriteBundle {
            texture: asset_server.load(match i % 3 {
                0 => "img/Creatures/Faces/happy-expression.png",
                1 => "img/Creatures/Faces/angry-expression.png",
                _ => "img/Creatures/Faces/strained-expression.png",
            }),
            transform: Transform::from_xyz(0.,0., 0.01),
            sprite: Sprite {
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..default()
            },
            ..default()
        }).id();

        let wing_handle = asset_server.load("img/wings.png");
        let wing_atlas = TextureAtlas::from_grid(wing_handle, Vec2::new(80.0, 80.0), 3, 1, None, None);
        let wing_atlas_handle = atlases.add(wing_atlas);
        let wing_anim = AnimationIndices { first: 0, last: 2 };
        let wings = commands.spawn((
            SpriteSheetBundle {
                texture_atlas: wing_atlas_handle,
                sprite: TextureAtlasSprite::new(wing_anim.first),
                transform: Transform::from_xyz(0.,2., 0.01).with_scale(Vec3::splat(50./80.)),
                ..default()
            },
            wing_anim,
            AnimationTimer(Timer::from_seconds(0.03 + (i as f32 * 0.01), TimerMode::Repeating)),
        )).id();

        commands.entity(bee).push_children(&[wings, arm, eyes]);

    }

}

fn set_unassigned_bees(
    mut commands: Commands,
    mut ent: Query<(Entity, &mut Transform), (With<RumbleBee>, With<Beenitialized>)>,
    tilemap: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &Navmesh,
    ), Without<RumbleBee>>,
){
    if !tilemap.is_empty() {
        let (map_size, grid_size, navmesh) = tilemap.single();
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
