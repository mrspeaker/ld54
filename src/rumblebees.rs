use crate::game::{
    OnGameScreen, Speed, Bob, Displacement, AnimationTimer,
    AnimationIndices
};
use crate::AssetCol;
use crate::pathfinding::FollowPath;
use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;
use rand::seq::IteratorRandom;
use crate::terrain::{GAP_LEFT, Tile, Egg, Faction};
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
                    birth_a_bee,
                    set_unassigned_bees,
                    find_target,
                    bee_fight_collisions,
                    bee_egg_collisions,
                    bee_fight,
                    big_bee_fight,
                    bee_dead.after(find_target).after(bee_fight_collisions).after(bee_fight)
                ).run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(Component)]
pub struct RumbleBee {
    pub faction: terrain::Faction,
}

#[derive(Component)]
pub struct Beenitialized {
    pub pos: Option<Vec2>
}

#[derive(Component)]
pub struct BeeFight;

#[derive(Component)]
pub struct BeeKilled;


#[derive(Component)]
pub struct BigBeeFight {
    pub bee1: Entity,
    pub bee2: Entity,
    started: Instant
}

#[derive(Component)]
pub struct BeeBorn {
    pub pos: Option<Vec2>,
    pub faction: Faction
}

fn birth_a_bee(
    mut commands: Commands,
    assets: Res<AssetCol>,
    bees: Query<(Entity, &BeeBorn)>,
    parent: Query<Entity, With<BeeContainer>>
) {
    for (ent, spawn) in bees.iter() {

        commands.entity(ent).despawn();

        let pos:Vec2 = spawn.pos.unwrap_or_else(|| Vec2::new(0.0,0.0));

        let is_blue = if spawn.faction == Faction::Blue { true } else { false };
        let bee_sprite = SpriteSheetBundle {
            texture_atlas: assets.chars.clone(),
            transform: Transform::from_scale(Vec3::splat(50.0/80.0)),
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
            Beenitialized {
                pos: spawn.pos,
            },
            OnGameScreen,
            FollowPath {
                end: pos,
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
            AnimationTimer(Timer::from_seconds(1.0, TimerMode::Repeating)),
        )).id();

        let wings = commands.spawn((
            SpriteSheetBundle {
                texture_atlas: assets.chars.clone(),
                sprite: TextureAtlasSprite::new(6),
                transform: Transform::from_xyz(0.,2., 0.01),
                ..default()
            },
            AnimationIndices { frames: vec![6, 7, 8, 7], cur: 0 },
            AnimationTimer(Timer::from_seconds(0.04, TimerMode::Repeating)),
        )).id();

        // should be bee or bee_sprite?
        commands.entity(bee).push_children(&[wings, arm, eyes]);
        if let Ok(beez) = parent.get_single() {
            commands.entity(beez).push_children(&[bee]);
        }

    }
}

// Punch arm
#[derive(Component)]
pub struct Army;

#[derive(Component)]
pub struct BeeContainer;

fn rumblebee_setup(
    mut commands: Commands,
){
    commands.spawn((SpatialBundle { ..default() }, BeeContainer, OnGameScreen))
        .insert(Name::new("Beez"));

    // Make the beez
    let num_beez = 2;
    for i in 0..num_beez {
        // Spawn new bee spawn (added in birth_bee)
        commands.spawn(BeeBorn {
            pos: None,
            faction: if i < num_beez / 2 { Faction::Blue } else { Faction::Red }
        });
    }

}

fn set_unassigned_bees(
    mut commands: Commands,
    mut ent: Query<(Entity, &mut Transform, &Beenitialized), With<RumbleBee>>,
    tilemap: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &Navmesh,
    ), Without<RumbleBee>>,
){
    if !tilemap.is_empty() {
        let (map_size, grid_size, navmesh) = tilemap.single();
        let mut rng = rand::thread_rng();

        for (ent, mut transform, beeinit) in ent.iter_mut() {
            let pos:Vec2 = match beeinit.pos {
                Some(pos) => pos,
                None => {
                    let mut target = TilePos { x: 0, y: 0 };
                    let mut ok = false;
                    while !ok {
                        target.x = rng.gen_range(0..map_size.x);
                        target.y = rng.gen_range(0..map_size.y);
                        ok = !navmesh.solid(target);
                    }
                    Vec2 {
                        x: target.x as f32 * grid_size.x + 25.0 + GAP_LEFT,
                        y: target.y as f32 * grid_size.y + 25.0
                    }
                }
            };

            // Set pos and give bee it's z index
            transform.translation =
                pos.extend(Layers::MIDGROUND + rng.gen_range(0..100) as f32);

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
                x: egg_pos.x as f32 * grid_size.x + 25. + GAP_LEFT,
                y: egg_pos.y as f32 * grid_size.y + 25.,
                z: bee_pos.translation.z
            };

            if bee_pos.translation.distance(pos) < 20.0 && bee.faction == egg.faction {
                // Got a egg..
                commands.entity(egg_ent).remove::<Egg>();
                *egg_tile = Tile::Air;

                // Spawn new bee
                commands.spawn(BeeBorn {
                    pos: Some(pos.xy().clone()),
                    faction: egg.faction
                });

                // Turn stalks to dead.
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

    let num = entities.len();
    for i in 0..num {
        let (ent_a,bee_a,  pos_a) = &entities[i];

        for j in i + 1..num {
            let (ent_b,bee_b,  pos_b ) = &entities[j];
            if bee_a.faction == bee_b.faction {
                continue;
            }
            let a = pos_a.translation.xy();
            let b = pos_b.translation.xy();
            if a.distance(b) < 50.0 {
                // GET READY TO BRUMBLE!
                if let Some(mut e) = commands.get_entity(*ent_a) {
                    e.insert(BeeFight);
                }
                if let Some(mut e) = commands.get_entity(*ent_b) {
                    e.insert(BeeFight);
                }
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


        let targets = eggs.iter().filter_map(|(egg, pos)| {
            (egg.faction == entity.2.faction).then_some((egg, pos))
        });

        let mut target_path: Option<Pathfinding> = None;
        // have a target egg - go to it!
        if let Some(first) = targets.choose(&mut rand::thread_rng()) {
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
            if let Some(mut e) = commands.get_entity(entity.0) {
                e.insert(path);
            }
        }

    }

}


fn big_bee_fight(
    mut commands: Commands,
    mut ent: Query<(Entity, &mut BigBeeFight)>,
    time: Res<Time>,
    bees: Query<Entity, With<BeeFight>>
){
    for (fight, beefight) in ent.iter_mut() {
        let t = time.last_update().unwrap() - beefight.started;
        if t.as_secs() > 5 {
            if bees.iter().any(|entity| entity == beefight.bee1) {
                commands.entity(beefight.bee1).remove::<BeeFight>();
            } else { info!("nop b1") }
            if bees.iter().any(|entity| entity == beefight.bee2) {
                commands.entity(beefight.bee2).remove::<BeeFight>().insert(BeeKilled);
            } else { info!("nop b2") }
            commands.entity(fight).despawn();
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
