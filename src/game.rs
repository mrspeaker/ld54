use std::ops::{Add, Sub};
use crate::pathfinding::{Pathfinding, Navmesh};
use crate::pointer::Pointer;
use crate::settings::{RUMBLEBEE_SPEED_START, DIG_REPEAT_IN_SECS, DIG_POWER, EGG_SPAWN_TIME_START};
use crate::terrain::{GAP_LEFT, px_to_tilepos, Health};
use crate::{despawn_screen, GameState, AssetCol};
use bevy::math::Vec3Swizzles;
use bevy::utils::Instant;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_kira_audio::prelude::*;

use crate::terrain::Tile;
use bevy_ecs_tilemap::prelude::*;


use crate::Layers;

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::InGame), game_setup)
            .add_event::<GotAnEgg>()
            .add_systems(
                Update,
                (
                    // dbg_draw_path,
                    check_exit,
                    move_bob,
                    follow_path,
                    check_if_stuck_in_tile,
                    smash_dirt_when_stuck,
                    animate_sprite,
                    update_sprite,
                    bevy::window::close_on_esc,
                    egg_listener
                )
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(OnExit(GameState::InGame), despawn_screen::<OnGameScreen>);
    }
}

#[derive(Component)]
pub struct FollowPath {
    pub end: Vec2,
    pub done: bool,
}

#[derive(Component)]
pub struct Stuck {
    tile: Entity,
    last_dig: Instant
}

/// A pair of navmeshes to represent an "alternative" view of the navmesh.
/// In our case that is the tilemap withouth DIRT tiles, to let an entity
/// find a path as if it wasn't obstructed by dirt.
#[derive(Component)]
pub struct NavmeshPair {
    pub main: Navmesh,
    pub alt: Navmesh,
}

#[derive(Component)]
pub struct Speed {
    pub speed: f32,
}

#[derive(Component)]
pub struct Displacement(pub Vec2);

#[derive(Component)]
pub struct Bob;

#[derive(Component)]
pub struct Wander;

#[derive(Component)]
pub struct Fight {
    opponent: Entity,
}

#[derive(Component)]
pub struct OnGameScreen;

#[derive(Component)]
pub struct AnimationIndices {
    pub frames: Vec<usize>,
    pub cur: usize
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

#[derive(Resource)]
pub struct GameData {
    pub eggs_spawned: usize,
    pub game_started: bool,
    pub game_over: bool,
    pub bee_base_speed: f32,
    pub egg_spawn_time: f32

}

#[derive(Event, Default)]
pub struct GotAnEgg;

fn egg_listener(
    mut events: EventReader<GotAnEgg>,
    mut game_data: ResMut<GameData>,
    assets: Res<AssetCol>,
    audio: Res<Audio>,
) {
    let got_egg = events.len() > 0;
    for _ in events.iter() {
        game_data.eggs_spawned += 1;
        game_data.game_started = true; // Can't get 0 total lol.
                                       // Not sure where "game_started" could be set.
    }

    if got_egg {
        audio.play(assets.blip.clone()).with_volume(0.8);
    }

}

fn move_bob(time: Res<Time>, mut pos: Query<(&mut Transform, Option<&Displacement>, With<Bob>)>) {
    let mut i = 0.;
    for (mut transform, displacment, _bob) in &mut pos {
        let mut do_bob = true;
        if let Some(displacement) = displacment {
            if (displacement.0.x).abs() < (displacement.0.y).abs() {
                do_bob = false;
            }
            i+=1.;
        }
        if do_bob {
           transform.translation.y +=
                    ((time.elapsed_seconds() + i) * 10.0).sin() * 0.4;
        }
    }
}

fn update_sprite(
    mut query: Query<(&mut Transform, Option<&Displacement>)>
) {
    for (mut transform, displacement) in query.iter_mut() {
        // Face the direction you are moving.
        if let Some(displacement) = displacement {
            if displacement.0.x != 0.0 {
                transform.scale.x = transform.scale.x.abs() * if displacement.0.x < 0.0 { -1.0 } else { 1.0 };
            }
        }
        // TODO: clamp position inside screen (stop Bob from pushing to outside tile)
    }
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &mut AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (mut indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            indices.cur = (indices.cur + 1) % indices.frames.len();
            sprite.index = indices.frames[indices.cur];
        }
    }
}

fn game_setup(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    assets: Res<AssetCol>,
    _audio: Res<Audio>
) {
    let window: &Window = window_query.get_single().unwrap();

    commands.insert_resource(GameData {
        eggs_spawned: 0,
        game_started: false,
        game_over: false,
        bee_base_speed: RUMBLEBEE_SPEED_START,
        egg_spawn_time: EGG_SPAWN_TIME_START
    });

    /*audio
        .play(assets.tune.clone())
        .loop_from(0.0)
        .fade_in(AudioTween::new(
            Duration::from_secs(2),
            AudioEasing::OutPowi(2),
        ))
        .with_volume(0.1);*/

    // Background image
    commands.spawn((
        SpriteBundle {
            texture: assets.bg.clone(),
            transform: Transform::from_xyz(
                window.width() / 2.0,
                window.height() / 2.0,
                Layers::BACKGROUND,
            )
            .with_scale(Vec3::new(1.7, 1.4, 0.0)),
            ..default()
        },
        OnGameScreen,
    ));

    // UI bit
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::hsla(120., 0.5, 0.2, 0.5),
                custom_size: Some(Vec2::new(GAP_LEFT, window.height())),
                ..default()
            },
            transform: Transform::from_xyz(GAP_LEFT / 2.0, window.height() / 2.0, Layers::UI),
            ..default()
        },
        OnGameScreen,
    ));

    // Exit button
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::hsl(20., 0.5, 0.1),
                custom_size: Some(Vec2::new(GAP_LEFT * 0.8, 50.0)),
                ..default()
            },
            transform: Transform::from_xyz(GAP_LEFT / 2.0, 40.0, Layers::UI),
            ..default()
        },
        OnGameScreen,
    ));

    commands.spawn((
        TextBundle::from_section(
            "exit",
            TextStyle {
                font: assets.font.clone(),
                font_size: 24.0,
                color: Color::WHITE,
            },
        ) // Set the alignment of the Text
        .with_text_alignment(TextAlignment::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(30.0),
            left: Val::Px(20.0),
            ..default()
        }),
        OnGameScreen,
    ));

}

/// Exit the game when exit button pressed
fn check_exit(
    pointer: Res<Pointer>,
    mut game_state: ResMut<NextState<GameState>>,
){
    if pointer.released &&
        pointer.pos.x < GAP_LEFT * 0.95 &&
        pointer.pos.y < 65.0 &&
        pointer.pos.y > 5.0
    {
        // transition to splash.
        game_state.set(GameState::Splash);
    }
}

pub fn follow_path(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<
            (Entity, &mut Pathfinding, &mut Transform, &Speed, Option<&mut Displacement>),
        (With<FollowPath>, Without<Stuck>)>,
    tilemap: Query<(
        &TilemapGridSize,
        &TilemapType,
    )>,
) {
    /// Distance to the target considered "at" the target.
    const TARGET_EPSILON: f32 = 5.0;
    let (grid_size, map_type) = tilemap.single();
    let delta_time = time.delta_seconds().min(0.3);
    if delta_time > 0.2 {
        info!(">>> dt");
    }
    for (entity, mut path, mut transform, speed, displacement) in &mut query {
        //TODO: get size from entity
        let target = path.current(grid_size, map_type).add(Vec2 { x: GAP_LEFT + 25., y: 25. });

        let delta =
            target.sub(transform.translation.xy()).normalize() * delta_time * speed.speed;

        if let Some(mut displacement) = displacement {
            displacement.0 = delta;
        }
        transform.translation += delta.extend(0.0);
        if transform.translation.xy().distance(target) < TARGET_EPSILON && !path.step() {
            commands.entity(entity).remove::<Pathfinding>();
        }
    }
}

/// Syncs the main and "no-dirt" nav meshes when tile map changes
pub fn update_navmesh_on_tile_change(
    mut tile_query: Query<(&Tile, &TilePos), Or<(Added<Tile>, Changed<Tile>)>>,
    mut navmesh: Query<&mut NavmeshPair>
) {
    for (tile, tile_pos) in &mut tile_query {
        let mut nmp = navmesh.get_single_mut().unwrap();
        nmp.main.set_solid(*tile_pos, match tile {
            // TODO: this logic is repeated several times!
            Tile::Air => false,
            Tile::Egg { .. } => false,
            _ => true
        });
        nmp.alt.set_solid(*tile_pos, match tile {
            // TODO: this logic is repeated several times!
            Tile::Air => false,
            Tile::Egg { .. } => false,
            Tile::Dirt { .. } => false,
            _ => true
        });
    }
}

/// Invalidates a path if a solid tile has been inserted along the way
pub fn remove_conflicting_paths_on_tile_change(
    mut commands: Commands,
    mut tile_query: Query<(&Tile, &TilePos), Changed<Tile>>,
    path: Query<(Entity, &mut Pathfinding), With<FollowPath>>,
) {
    for (tile, tile_pos) in &mut tile_query {
        if Tile::is_solid(*tile) {
            // Invalidate any crossing paths
            for (ent, path) in path.iter() {
                if path.path.contains(tile_pos) {
                    commands.entity(ent).remove::<Pathfinding>();
                }
            }

        }
    }
}

fn check_if_stuck_in_tile(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform), (With<Pathfinding>, Without<Stuck>)>,
    tilemap: Query<(
        &TilemapGridSize,
        &TileStorage,
    )>,
    tiles: Query<&Tile, With<Health>>,
    time: Res<Time>
){
    let (grid_size, storage) = tilemap.single();

    for (entity, transform) in &mut query {
        // Am I currently inside a solid block?
        let pos = transform.translation.xy().add(Vec2 { x: -GAP_LEFT, y: 0.0 });
        let tile_pos = px_to_tilepos(pos, grid_size);
        let tile_ent = storage.get(&tile_pos);
        let tile = tile_ent.and_then(|e| tiles.get(e).ok());

        if let Some(Tile::Dirt { .. }) = tile {
            commands.entity(entity)
                .insert(Stuck {
                    tile: tile_ent.unwrap(),
                    last_dig: time.last_update().unwrap() } )
                .remove::<Pathfinding>();
        }
    }
}

/// When entity is stuck in dirt, smash out after some time
fn smash_dirt_when_stuck(
    mut commands: Commands,
    mut ents: Query<(Entity, &mut Stuck)>,
    mut tiles: Query<(&mut Tile, &mut Health, &mut TileColor)>,
    time: Res<Time>
) {

    for (entity, mut stuck) in ents.iter_mut() {
        let t = time.last_update().unwrap();
        let dt = t - stuck.last_dig;
        if dt.as_secs_f32() < DIG_REPEAT_IN_SECS {
            continue;
        }
        stuck.last_dig = t;

        let mut tile_done = false;
        if let Ok((mut tile, mut health, mut color)) = tiles.get_mut(stuck.tile) {
            health.0 = health.0.saturating_sub(DIG_POWER); // Kill some dirt HP.
            let hp = health.0;
            if hp == 0 {
                *tile = Tile::Air;
                tile_done = true;
                color.0.set_a(100.);
            } else {
                color.0.set_a(hp as f32 / 100.);
            }
        } else {
            tile_done = true;
        }

        if tile_done {
            commands.entity(entity).remove::<Stuck>();
        }
    }
}
