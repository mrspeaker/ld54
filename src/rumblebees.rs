use crate::game::{
    OnGameScreen, Speed, Bob, Displacement, AnimationTimer,
    AnimationIndices
};
use crate::AssetCol;
use crate::pathfinding::FollowPath;
use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;
use crate::terrain::{GAP_LEFT, TILE_SIZE, Tile, Egg, Faction};
use crate::{prelude::*, GameState};
use bevy::math::swizzles::Vec3Swizzles;
use bevy::prelude::*;
use bevy::utils::Instant;
use rand::Rng;
use std::ops::Sub;

use crate::Layers;

const RUMBLEBEE_SPEED: f32 = 50.0;

pub struct RumblebeePlugin;
impl Plugin for RumblebeePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::InGame), rumblebee_setup)
            .add_systems(
                Update,
                (
                    set_unassigned_bees,
                    find_target,
                    bee_fight_collisions,
                    bee_egg_collisions,
                    bee_fight,
                    big_bee_fight,
                    bee_dead
                ).run_if(in_state(GameState::InGame)),
            );
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
pub struct BeeKilled;


#[derive(Component)]
pub struct BigBeeFight {
    pub bee1: Entity,
    pub bee2: Entity,
    started: Instant
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
            transform: Transform::from_translation(bee_pos)
                .with_scale(Vec3::splat(50.0/80.0)),
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
                transform: Transform::from_xyz(0.,0., 0.01),
                ..default()
            },
            Army
        )).id();

        let eyes = commands.spawn((
            SpriteSheetBundle {
                texture_atlas: assets.chars.clone(),
                transform: Transform::from_xyz(0.,0., 0.01),
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
                transform: Transform::from_xyz(0.,2., 0.01),
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
            transform.translation.x = target.x as f32 * grid_size.x + 25.0 + GAP_LEFT;
            transform.translation.y = target.y as f32 * grid_size.y + 25.0;
            commands.entity(ent).remove::<Beenitialized>();
        }
    }
}

fn bee_fight(
    mut commands: Commands,
    beez: Query<(Entity, &RumbleBee, &Transform, &Children), Added<BeeFight>>,
    army: Query<Entity, With<Army>>,
){
    // Bees be fightin'.
    for (ent, _bee, _transform, children) in beez.iter() {
        commands.entity(ent)
            .remove::<Pathfinding>();

        for child in children {
            if let Ok(army) = army.get(*child) {
                commands.entity(army)
                    .insert(AnimationIndices { frames: vec![0, 1], cur: 0 })
                    .insert(AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)));
            }
        }

    }
}

fn bee_egg_collisions(
    mut commands: Commands,
    beez: Query<(Entity, &RumbleBee, &Transform)>,
    mut eggs: Query<(Entity, &Egg, &mut Tile, &TilePos)>,
    tilemap: Query<(&TileStorage, &TilemapSize, &TilemapGridSize)>,
    mut tile_query: Query<&mut Tile, Without<Egg>>,
){
    let (tile_storage, map_size, grid_size) = tilemap.single();

    for (_bee_ent, bee, bee_pos) in beez.iter() {
        for (egg_ent, egg, mut egg_tile, egg_pos) in eggs.iter_mut() {
            let pos = Vec3 {
                x: egg_pos.x as f32 * grid_size.x + 25. +GAP_LEFT,
                y: egg_pos.y as f32 * grid_size.y + 25.,
                z: bee_pos.translation.z
            };

            if bee_pos.translation.distance(pos) < 20.0 && bee.faction == egg.faction {
                // Got a egg.. turning it to poo for some reason
                commands.entity(egg_ent).remove::<Egg>();
                *egg_tile = Tile::Air;//Poo{ style: if bee.faction == Faction::Blue { 1 } else { 0 } };

                let mut next_pos = egg_pos.clone();
                let mut is_stalk = true;
                while is_stalk {
                    let down_pos = Neighbors::get_square_neighboring_positions(&next_pos, map_size, false).south;
                    match down_pos {
                        Some(down_pos) => {
                            if let Some(plant_ent) = tile_storage.get(&down_pos) {
                                if let Ok(mut tile) = tile_query.get_mut(plant_ent) {
                                    match &mut *tile {
                                        Tile::Stalk { style } => {
                                            if *style == 0 {
                                                *style = 1;
                                            }
                                            next_pos = down_pos;
                                        },
                                        _ => {
                                            is_stalk = false;
                                        },
                                    }
                                }
                            }
                        },
                        None => {
                            is_stalk = false;
                        }
                    }}
            }
        }
    }
}

fn bee_fight_collisions(
    mut commands: Commands,
    beez: Query<(Entity, &RumbleBee, &Transform), (Without<BeeFight>, Without<Beenitialized>)>,
    time: Res<Time>
){
    let entities: Vec<(Entity, &RumbleBee, &Transform)> = beez.iter().map(|(entity, rumblebee, transform)|
        (entity, rumblebee, transform)
    ).collect();

    for i in 0..entities.len() {
        for j in i+1..entities.len() {
            let (ent_a,bee_a,  pos_a) = &entities[i];
            let (ent_b,bee_b,  pos_b ) = &entities[j];
            if bee_a.faction == bee_b.faction {
                continue;
            }
            //check for collision between entity_a and entity_b here
            if pos_a.translation.distance(pos_b.translation) < 50.0 {
                // GET READY TO BRUMBLE!
                commands.entity(*ent_a).insert(BeeFight{
                    opponent: *ent_b
                });
                commands.entity(*ent_b).insert(BeeFight{
                    opponent: *ent_a
                });
                commands.spawn(BigBeeFight {
                    bee1: *ent_a,
                    bee2: *ent_b,
                    started: time.last_update().unwrap()
                });
            }
        }
    }

}

/// Set the organisms pathfinding to go to the given tile.
fn find_target(
    mut commands: Commands,
    entity: Query<
            (Entity, &Transform, &RumbleBee),
        (Without<Pathfinding>, Without<BeeFight>, Without<Beenitialized>)>,
    tilemap: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &Navmesh,
    )>,
    eggs: Query<(&Egg, &TilePos)>,
) {
    let (map_size, grid_size, map_type, navmesh) = tilemap.single();
    for entity in entity.iter() {
        let pos = &entity.1
            .translation.xy()
            // TODO: 25 is bee size / 2. Get from transform!
            .sub(Vec2 { x:GAP_LEFT + 25.0, y: 25.0 });
        let Some(entity_pos) =
            TilePos::from_world_pos(pos, map_size, grid_size, map_type)
        else {
            //Why are some not getting world pos?
            //info!("Entity outside map {:?} {} {}", &entity.1.translation.xy(), map_size.x as f32 * grid_size.x, map_size.y as f32 * grid_size.y);
            continue;
        };


        let mut targets = eggs.iter().filter_map(|(egg, pos)| {
            (egg.faction == entity.2.faction).then_some((egg, pos))
        });

        let mut target_path: Option<Pathfinding> = None;
        // have a target egg - go to it!
        if let Some(first) = targets.next() {
            if let Some(path) = Pathfinding::astar(navmesh, entity_pos, first.1.clone()) {
                target_path = Some(path);
            }
        }

        // No egg target, just wander to random spot
        if target_path.is_none() {
            // No target, just wander aimlessly
            let mut rng = rand::thread_rng();
            let mut ok = false;
            let mut target = TilePos { x: 0, y: 0 };
            while !ok {
                target.x = rng.gen_range(0..map_size.x);
                target.y = rng.gen_range(0..map_size.y);
                ok = !navmesh.solid(target);
            }

            if let Some(path) = Pathfinding::astar(navmesh, entity_pos, target) {
                target_path = Some(path);
            }

        }

        if let Some(path) = target_path {
            commands.entity(entity.0).insert(path);
        }

    }

}


fn big_bee_fight(
    mut commands: Commands,
    mut ent: Query<(Entity, &mut BigBeeFight)>,
    time: Res<Time>
){
    for (ent, beefight) in ent.iter_mut() {
        let t = time.last_update().unwrap() - beefight.started;
        if t.as_secs() > 5 {
            commands.entity(beefight.bee1).remove::<BeeFight>();
            commands.entity(beefight.bee2).insert(BeeKilled);
            commands.entity(ent).despawn();
        }
    }
}

fn bee_dead(
    mut commands: Commands,
    mut ent: Query<(Entity, &Transform), With<BeeKilled>>,
    assets: Res<AssetCol>
) {
    for (ent, pos) in ent.iter_mut() {
        // TODO: needs to set tilemap, not just be a sprite
        commands.spawn((SpriteSheetBundle {
            texture_atlas: assets.tiles.clone(),
            transform: Transform::from_xyz(
                pos.translation.x.floor(),
                pos.translation.y.floor(),
                Layers::MIDGROUND),
            sprite: TextureAtlasSprite::new(37),
            ..default()
        }, OnGameScreen));
        commands.spawn((SpriteSheetBundle {
            texture_atlas: assets.tiles.clone(),
            transform: Transform::from_xyz(
                pos.translation.x.floor() + 40.0,
                pos.translation.y.floor(),
                Layers::MIDGROUND),
            sprite: TextureAtlasSprite::new(38),
            ..default()
        }, OnGameScreen));
        commands.entity(ent).despawn_recursive();
    }
}
