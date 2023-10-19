use crate::game::{
    OnGameScreen, Speed, Bob, Displacement,
    AnimationTimer, AnimationIndices, GotAnEgg, GameData, NavmeshPair, FollowPath, GameOver
};
use crate::AssetCol;
use crate::settings::{RUMBLEBEE_SPEED_MAX, RUMBLEBEE_PER_EGG_SPEEDUP_PERC, RUMBLEBEE_SPEED_START, RUMBLEBEE_SPEED_VARIANCE};
use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;
use rand::seq::IteratorRandom;
use crate::terrain::{GAP_LEFT, Tile, Egg, Faction, tilepos_to_px, find_empty_tile, px_to_tilepos};
use crate::{prelude::*, GameState};
use bevy::math::swizzles::Vec3Swizzles;
use bevy::prelude::*;
use bevy::utils::Instant;
use rand::Rng;
use std::ops::Sub;
use std::time::Duration;

use crate::Layers;

/*
Systems:
1. setup.
2. birth_a_bee. <BeeBorn>,
3. find_target. <Rumblebee> Without<Pathfinding, BeeFighter>
4. egg_collisions. <Rumblebee>
5. fight_collisions. <RumbeBee> Without<BeeFighter>
6. became_a_fighter. <RumbleBee> Added<BeeFighter>
7. bee_fight <BeeFight>. beez With<BeeFighter>
8. bee_dead
  .after(find_target)
  .after(bee_fight) Added<BeeKilled>

Setup:
1. spawns some <BeeBorn> entities.

Update:
2. birth_a_bee. <BeeBorn> <Navmesh>
   - despawn <BeeBorn>
   - spawn <RumbleBee>
     - if has pos, set it else find blank tile.
     - set pos + z index.

3. find_target. <Rumblebee> Without<Pathfinding, BeeFighter>
   - add <Pathfinding>

4. egg_collisions. <Rumblebee>
   - spawn <BeeBorn> on hit egg

5. fight_collisions. <RumbeBee> Without<BeeFighter>
   - for each pair, if collision:
     - add <BeeFighter> to each
     - spawn <BeeFight bee1 bee2>

6. became_a_fighter. <RumbleBee> Added<BeeFighter>
   - remove <Pathfinding>

7. bee_fight. <BeeFight>, beez With<BeeFighter>
   - after 5 secs:
     despawn <BeeFight>
     bee1: remove <BeeFighter>
     bee2: add <BeeKilled>

8. bee_dead. Added<BeeKilled>
   - despawn recursive.
*/

pub struct RumblebeePlugin;
impl Plugin for RumblebeePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::InGame), rumblebee_setup)
            .add_systems(
                Update,
                (
                    do_nothing_for_a_bit,
                    birth_a_bee,
                    find_target,
                    egg_collisions,
                    fight_collisions,
                    bee_fight,
                    became_a_fighter,
                    bee_dead,
                ).run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(Component)]
pub struct RumbleBee {
    pub faction: terrain::Faction,
}

#[derive(Component)]
pub struct Inactive {
    pub timer: Timer
}

// Container for grouping in Debug plugin
#[derive(Component)]
struct BeeContainer;

#[derive(Component)]
struct BeeBorn {
    pos: Option<Vec2>,
    faction: Faction,
}

// Punch arm
#[derive(Component)]
pub struct ArmAnim;

#[derive(Component)]
struct BeeFight {
    bee1: Entity,
    bee2: Entity,
    started: Instant
}

#[derive(Component)]
struct BeeFighter;

#[derive(Component)]
struct BeeKilled;

fn do_nothing_for_a_bit(
    mut commands: Commands,
    mut beez: Query<(Entity, &mut Inactive)>,
    time: Res<Time>
) {

    for (ent, mut inactive) in beez.iter_mut() {
        inactive.timer.tick(time.delta());
        if inactive.timer.finished() {
            commands.entity(ent).remove::<Inactive>();
        }
    }
}

fn rumblebee_setup(
    mut commands: Commands,
){
    commands.spawn((SpatialBundle { ..default() }, BeeContainer, OnGameScreen))
        .insert(Name::new("Beez"));

    // Make the beez
    commands.spawn(BeeBorn {
        pos: Some(Vec2 { x: 580.0, y: 300.0 }),
        faction: Faction::Blue
    });
    commands.spawn(BeeBorn {
        pos: Some(Vec2 { x: 460.0, y: 300.0 }),
        faction: Faction::Red
    });


}

fn birth_a_bee(
    mut commands: Commands,
    assets: Res<AssetCol>,
    bees: Query<(Entity, &BeeBorn)>,
    parent: Query<Entity, With<BeeContainer>>,
    tilemap: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &NavmeshPair,
    ), Without<RumbleBee>>,
    mut game_data: ResMut<GameData>
) {
    let (map_size, grid_size, navmesh) = tilemap.single();
    let mut rng = rand::thread_rng();

    for (ent, spawn) in bees.iter() {

        commands.entity(ent).despawn(); // Remove the BeeBorn entity

        // Speed up a bit every bee
        game_data.bee_base_speed =
            game_data.bee_base_speed +
            ((RUMBLEBEE_SPEED_MAX - RUMBLEBEE_SPEED_START)
             * RUMBLEBEE_PER_EGG_SPEEDUP_PERC)
            .min(RUMBLEBEE_SPEED_MAX); // Cap it to max

        screen_print!(sec: 5.0, "bee base speed: {:?}", game_data.bee_base_speed);

        let pos_given = spawn.pos
            .or_else(|| find_empty_tile(&navmesh.main, map_size)
                     .map(|pos| tilepos_to_px (&pos, grid_size)));

        if pos_given.is_none() {
            // Couldn't find a place!
            continue;
        }
        let pos = pos_given.unwrap().extend({
            // Set pos and give bee it's z index
            Layers::MIDGROUND + rng.gen_range(0..100) as f32
        });

        let is_blue = if spawn.faction == Faction::Blue { true } else { false };
        let bee_sprite = SpriteSheetBundle {
            texture_atlas: assets.chars.clone(),
            transform: Transform::from_translation(pos).with_scale(Vec3::splat(50.0/80.0)),
            sprite: TextureAtlasSprite::new(if is_blue {0} else {1}),
            ..default()
        };

        let bee = commands.spawn((
            bee_sprite,
            RumbleBee {
                faction: match is_blue {
                    true => Faction::Blue,
                    false => Faction::Red
                }
            },
            Inactive { timer: Timer::new(Duration::from_secs(3), TimerMode::Once) },
            OnGameScreen,
            FollowPath {
                end: pos.xy(),
                done: true,
            },
            Speed { speed: rng.gen_range(
                game_data.bee_base_speed * (1.0 - RUMBLEBEE_SPEED_VARIANCE) ..
                    game_data.bee_base_speed * (1.0 + RUMBLEBEE_SPEED_VARIANCE)) },
            Bob,
            Displacement(Vec2 { x: 0., y: 0. }),
        )).id();

        let arm = commands.spawn((
            SpriteSheetBundle {
                texture_atlas: assets.arms.clone(),
                transform: Transform::from_xyz(0.,0., 0.01),
                ..default()
            },
            ArmAnim,
            AnimationIndices { frames: vec![0], cur: 0 },
            AnimationTimer(
                Timer::from_seconds(0.1, TimerMode::Repeating)
            ),
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

/// Set the bee's pathfinding to go to a target tile
fn find_target(
    mut commands: Commands,
    entity: Query<(Entity, &Transform, &RumbleBee),
                  (Without<Inactive>, Without<Pathfinding>, Without<BeeFighter>)>,
    tilemap: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &NavmeshPair,
    )>,
    eggs: Query<(&Egg, &TilePos)>,
    mut game_data: ResMut<GameData>
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
            info!("Entity outside map {:?} {} {}", &entity.1.translation.xy(), map_size.x as f32 * grid_size.x, map_size.y as f32 * grid_size.y);
            continue;
        };

        let targets = eggs.iter().filter_map(|(egg, pos)| {
            (match_faction(entity.2.faction, egg.faction)).then_some((egg, pos))
        });

        let mut target_path: Option<Pathfinding> = None;

        /*
        TODO: Choose an egg to target: just random, should be "closest"
        1. for each target, look for a path. get the shortest.
        2. if no target, check again for non-dirt path, choose shortest.
        3. if none, wander.
        */

        if let Some(first) = targets.choose(&mut rand::thread_rng()) {
            if let Some(path) = Pathfinding::astar(&navmesh.main, entity_pos, first.1.clone()) {
                target_path = Some(path);
            } else {
                // Find the neearest dirt tile that is blocking path to egg
                if let Some(mut path) = Pathfinding::astar(&navmesh.alt, entity_pos, first.1.clone()) {
                    // Get path to dirt
                    let mut p: Vec<TilePos> = vec![];
                    while path.step() {
                        let pos = path.path[path.at];
                        p.push(pos);
                        if navmesh.main.solid(pos) {
                            // hit dirt
                            break;
                        }
                    }
                    if p.len() > 0 {
                        target_path = Some(Pathfinding { path: p, at: 0 });
                    }
                }
            }
        }

        // No egg target, just wander to random spot
        if target_path.is_none() {
            // No target, just wander aimlessly
            let mut rng = rand::thread_rng();
            let mut ok = false;
            let mut retries = 20;
            while !ok {
                if let Some(t)  = find_empty_tile(&navmesh.main, map_size) {
                    if let Some(path) = Pathfinding::astar(&navmesh.main, entity_pos, t) {
                        target_path = Some(path);
                        ok = true;
                    } else {
                        // Found a free spot, but can't get to it.
                        retries -= 1;
                        if retries < 0 {
                            // Not sure...
                            //info!("No where to go!");
                            ok = true;
                        }
                    }
                } else {
                    // No paths left. Game over?
                    game_data.game_over = true;
                    commands.spawn(GameOver);
                }
            }
            if retries < 0 {
                // Didn't find a path
                continue;
            }


            let mut target = TilePos { x: 0, y: 0 };
            while !ok {
                target.x = rng.gen_range(0..map_size.x);
                target.y = rng.gen_range(0..map_size.y);
                if !&navmesh.main.solid(target) {
                    if let Some(path) = Pathfinding::astar(&navmesh.main, entity_pos, target) {
                        target_path = Some(path);
                        ok = true;
                    }
                }
            }
        }

        if let Some(path) = target_path {
            commands.entity(entity.0).insert(path);
        }

    }
}

fn match_faction (a: Faction, b: Faction) -> bool {
    a == b || b == Faction::Green
}

fn egg_collisions(
    mut commands: Commands,
    beez: Query<(Entity, &RumbleBee, &Transform)>,
    mut eggs: Query<(Entity, &Egg, &mut Tile, &TilePos)>,
    tilemap: Query<(&TileStorage, &TilemapSize, &TilemapGridSize)>,
    mut tile_query: Query<&mut Tile, Without<Egg>>,
    mut got_egg_event: EventWriter<GotAnEgg>,
    game_data: Res<GameData>
){
    let (tile_storage, map_size, grid_size) = tilemap.single();

    for (_bee_ent, bee, bee_pos) in beez.iter() {
        for (egg_ent, egg, mut egg_tile, egg_pos) in eggs.iter_mut() {
            let pos = Vec3 {
                x: egg_pos.x as f32 * grid_size.x + 25. + GAP_LEFT,
                y: egg_pos.y as f32 * grid_size.y + 25.,
                z: bee_pos.translation.z
            };

            if match_faction(bee.faction, egg.faction) &&
                bee_pos.translation.distance(pos) < 20.0
            {
                // Got a egg..
                commands.entity(egg_ent).remove::<Egg>();

                if game_data.game_over {
                    return;
                }

                got_egg_event.send_default();


                // Egg turns into a un-passable stalk.
                // Should it? Maybe just Air to make it easier
                *egg_tile = Tile::Air;
                //*egg_tile = Tile::Stalk { style: 2 };

                // Spawn new bee
                commands.spawn(BeeBorn {
                    pos: Some(pos.xy().clone()),
                    faction: bee.faction
                });


                let mut next_pos = egg_pos.clone();
                let mut is_stalk = true;
                let mut is_top = true;

                // Walk down the stalk, turning the tree dead
                while is_stalk {
                    let down_pos = Neighbors::get_square_neighboring_positions(&next_pos, map_size, false).south;
                    match down_pos {
                        Some(down_pos) => {
                            if let Some(plant_ent) = tile_storage.get(&down_pos) {
                                if let Ok(mut tile) = tile_query.get_mut(plant_ent) {
                                    match &mut *tile {
                                        Tile::Stalk { style } => {
                                            if *style == 0 {
                                                *style = if is_top { 2 } else { 1 };
                                                is_top = false;
                                            }
                                            next_pos = down_pos;
                                        },
                                        _ => {
                                            is_stalk = false;
                                            // Is it only a 1-high tree?
                                            if is_top {
                                                // Yep, make it a dead stalk (instead of air)
                                                // good rule? Otherwise making 1-high would
                                                // be dominant strat
                                                *egg_tile = Tile::Stalk { style: 2 };
                                            }

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

fn fight_collisions(
    mut commands: Commands,
    beez: Query<(Entity, &RumbleBee, &Transform), (Without<Inactive>, Without<BeeFighter>)>,
    time: Res<Time>
){
    for [
        (ent_a, bee_a, pos_a),
        (ent_b, bee_b, pos_b)
    ] in beez.iter_combinations() {
        if bee_a.faction == bee_b.faction {
            continue;
        }
        let a = pos_a.translation.xy();
        let b = pos_b.translation.xy();
        if a.distance(b) < 50.0 {
            // GET READY TO BRUMBLE!
            commands.entity(ent_a).insert(BeeFighter);
            commands.entity(ent_b).insert(BeeFighter);
            commands.spawn(BeeFight {
                bee1: ent_a,
                bee2: ent_b,
                started: time.last_update().unwrap()
            });
        }
    }
}

fn became_a_fighter(
    mut commands: Commands,
    beez: Query<(Entity, &RumbleBee, &Transform, &Children), Added<BeeFighter>>,
    mut arms: Query<&mut AnimationIndices, With<ArmAnim>>,
){
    // Bee just got in a fight.
    for (ent, _bee, _transform, children) in beez.iter() {
        commands
            .entity(ent)
            .remove::<Pathfinding>();

        for child in children.iter() {
            if let Ok(mut arm) = arms.get_mut(*child) {
                arm.frames = vec![0, 1];
                arm.cur = 0;
            }
        }

    }
}

fn bee_fight(
    mut commands: Commands,
    mut bee_fight: Query<(Entity, &mut BeeFight)>,
    bees: Query<(Entity, &Children), With<BeeFighter>>,
    mut arms: Query<&mut AnimationIndices, With<ArmAnim>>,
    time: Res<Time>,
){
    for (fight_ent, beefight) in bee_fight.iter_mut() {
        let t = time.last_update().unwrap() - beefight.started;
        if t.as_secs() < 5 {
            continue;
        }
        commands.entity(fight_ent).despawn();

        for (bee, kids) in bees.iter() {
            if bee == beefight.bee1 {
                commands
                    .entity(bee)
                    .remove::<BeeFighter>();

                for &child in kids.iter() {
                    if let Ok(mut arm) = arms.get_mut(child) {
                        // Back to no anim
                        arm.frames = vec![0];
                        arm.cur = 0;
                    }
                }
            }
            if bee == beefight.bee2 {
                commands.entity(beefight.bee2).insert(BeeKilled);
            }
        }

    }
}

fn bee_dead(
    mut commands: Commands,
    mut ent: Query<(Entity, &RumbleBee, &Transform), Added<BeeKilled>>,
    all_beez: Query<&RumbleBee, Without<BeeKilled>>,
    //assets: Res<AssetCol>,
    tilemap: Query<(&TileStorage, &TilemapGridSize)>,
    mut tile_query: Query<&mut Tile, Without<Egg>>,
    mut game_data: ResMut<GameData>
) {
    let (tile_storage, grid_size) = tilemap.single();

    for (ent, bee, pos) in ent.iter_mut() {
        commands.entity(ent).despawn_recursive();

        // Get tile pos.
        let tp = px_to_tilepos(pos.translation.xy().sub(Vec2 { x: GAP_LEFT, y: 0.0 }), grid_size);

        if let Some(tile_ent) = tile_storage.get(&tp) {
            if let Ok(mut tile) = tile_query.get_mut(tile_ent) {
                *tile = Tile::Poo { style: if bee.faction == Faction::Blue { 1 } else { 0 } };
            }
        }

        /*commands.spawn((SpriteSheetBundle {
            texture_atlas: assets.tiles.clone(),
            transform: Transform::from_xyz(
                tpx.x,
                tpx.y,
                Layers::MIDGROUND - 1.0),
            sprite: TextureAtlasSprite::new(37),
            ..default()
        }, OnGameScreen));

        // Add some bones
        // TODO: needs to set tilemap, not just be a sprite
        let size = 40.0;
        commands.spawn((SpriteSheetBundle {
            texture_atlas: assets.tiles.clone(),
            transform: Transform::from_xyz(
                pos.translation.x.floor(),
                pos.translation.y.floor(),
                Layers::MIDGROUND - 1.0).with_scale(Vec3 { x: 0.4, y: 0.8, z: 1.0 }),
            sprite: TextureAtlasSprite::new(37),
            ..default()
        }, OnGameScreen));
        commands.spawn((SpriteSheetBundle {
            texture_atlas: assets.tiles.clone(),
            transform: Transform::from_xyz(
                pos.translation.x.floor() + size * 0.4,
                pos.translation.y.floor(),
                Layers::MIDGROUND - 1.0).with_scale(Vec3 { x: 0.4, y: 0.8, z: 1.0 }),
            sprite: TextureAtlasSprite::new(38),
            ..default()
         }, OnGameScreen));*/
    }

    // Is it game over?
    let mut blue = 0;
    let mut red = 0;
    for bee in all_beez.iter() {
        if bee.faction == Faction::Blue {
            blue += 1;
        }
        if bee.faction == Faction::Red {
            red += 1;
        }
    }
    if (blue == 0 || red == 0) &&
        game_data.game_started &&
        !game_data.game_over
    {
        info!("whattt game oer");
        // Game over!
        game_data.game_over = true;
        commands.spawn(GameOver);
    }
}
