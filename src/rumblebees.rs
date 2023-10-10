use crate::game::{
    OnGameScreen, Speed, Bob, Displacement, AnimationTimer,
    AnimationIndices, Wander
};
use crate::AssetCol;
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
pub struct Beenitialized;

#[derive(Component)]
pub struct BeeFight {
    pub opponent: Entity
}

#[derive(Component)]
pub enum BeeState {
    Wander,
    EggHunt,
    Fight {
        opponent: Entity
    }
}

// Punch arm
#[derive(Component)]
pub struct Army;

fn rumblebee_setup(
    mut commands: Commands,
    assets: Res<AssetCol>
){
    // Make the beez
    let num_beez = 8;
    for i in 0..num_beez {
        let pos = TilePos { x: 0, y : 0 };
        let bee_z = Layers::MIDGROUND + i as f32;
        let bee_pos = Vec3::new(
            pos.x as f32 * TILE_SIZE + GAP_LEFT,
            pos.y as f32 * TILE_SIZE,
            bee_z,
        );

        let is_blue = i < num_beez / 2;
        let bee_sprite = SpriteSheetBundle {
            texture_atlas: assets.chars.clone(),
            transform: Transform::from_translation(bee_pos).with_scale(Vec3::splat(50.0/80.0)),
            sprite: TextureAtlasSprite::new(if is_blue {0} else {1}),
            ..default()
        };

        let bee = commands.spawn((
            bee_sprite,
            RumbleBee {
                faction: match is_blue {
                    true => terrain::Faction::Blue,
                    false => terrain::Faction::Red
                }
            },
            Beenitialized,
            OnGameScreen,
            FollowPath {
                end: bee_pos.xy(),
                done: true,
            },
            Speed { speed: RUMBLEBEE_SPEED },
            Bob,
            Displacement(Vec2 { x: 0., y: 0. }),
        )).id();

        let arm = commands.spawn((
            SpriteSheetBundle {
                texture_atlas: assets.arms.clone(),
                transform: Transform::from_xyz(0.,0., 0.01).with_scale(Vec3::splat(50.0/80.0)),
                ..default()
            },
            Army
        )).id();

        let eyes = commands.spawn((
            SpriteSheetBundle {
                texture_atlas: assets.chars.clone(),
                transform: Transform::from_xyz(0.,0., 0.01).with_scale(Vec3::splat(50.0/80.0)),
                sprite: TextureAtlasSprite::new(9),
                ..default()
            },
            AnimationIndices { frames: vec![9, 10, 11, 10], cur: 0 },
            AnimationTimer(Timer::from_seconds(1.0 + (i as f32 * 0.1), TimerMode::Repeating)),
        )).id();

        let wings = commands.spawn((
            SpriteSheetBundle {
                texture_atlas: assets.chars.clone(),
                sprite: TextureAtlasSprite::new(6),
                transform: Transform::from_xyz(0.,2., 0.01).with_scale(Vec3::splat(50./80.)),
                ..default()
            },
            AnimationIndices { frames: vec![6, 7, 8, 7], cur: 0 },
            AnimationTimer(Timer::from_seconds(0.03 + (i as f32 * 0.01), TimerMode::Repeating)),
        )).id();

        // should be bee or bee_sprite?
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


fn wander_start(
    mut _commands: Commands,
    mut _ent: Query<Entity, Added<Wander>>
){
    // just got wander.
}
