use std::ops::Sub;

use crate::pathfinding::{Navmesh, follow_path};
use crate::rumblebees::RumbleBee;
use crate::terrain::{GAP_LEFT, Plant};
use crate::{despawn_screen, prelude::*, GameState};
use bevy::{input::mouse::MouseButtonInput, prelude::*, window::PrimaryWindow};
use bevy_ecs_tilemap::tiles::TilePos;
use bevy::math::swizzles::Vec3Swizzles;
use rand::Rng;

use crate::Layers;

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::InGame), game_setup)
            .add_systems(
                Update,
                (
                    // dbg_draw_path,
                    move_bob,
                    follow_path,
                    find_target,
                    mouse_button_events,
                    animate_sprite,
                    update_sprite,
                    bevy::window::close_on_esc,
                )
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(OnExit(GameState::InGame), despawn_screen::<OnGameScreen>);
    }
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
pub struct OnGameScreen;

#[derive(Resource, Deref, DerefMut)]
struct GameTimer(Timer);

#[derive(Component)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

#[derive(Resource)]
struct GameData {
    tiles: usize,
}

/// Set the organisms pathfinding to go to the given tile.
fn find_target(
    mut commands: Commands,
    entity: Query<(Entity, &Transform, &RumbleBee), Without<Pathfinding>>,
    tilemap: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &Navmesh,
    )>,
    plants: Query<(&Plant, &TilePos)>,
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
            // info!("Entity outside map {:?} {} {}", &entity.1.translation.xy(), map_size.x as f32 * grid_size.x, map_size.y as f32 * grid_size.y);
            continue;
        };

        let targets = plants.iter().filter_map(|(plant, pos)| (plant.ptype == entity.2.faction).then_some(pos));
        let mut target = TilePos { x: 0, y: 0 }; // { x: target.x, y: target.y };
        /*if let Some(first) = &targets.next() {
            info!("ya {} {}", first.x, first.y);
            target.x = first.x;
            target.y = first.y;
        } else {*/
            let mut rng = rand::thread_rng();
            let mut ok = false;
            while !ok {
                target.x = rng.gen_range(0..map_size.x);
                target.y = rng.gen_range(0..map_size.y);
                ok = !navmesh.solid(target);
            }
        //}
        if let Some(path) = Pathfinding::astar(navmesh, entity_pos, target) {
            commands.entity(entity.0).insert(path);
        }

/*        for &target in plants.iter().filter_map(|(plant, pos)| (plant.ptype == entity.2.faction).then_some(pos)) {
            // Go to random spots, just for fun
            // (don't just stop at the only plant!)
            if true || entity_pos.x == target.x && entity_pos.y == target.y {
                let mut rng = rand::thread_rng();
                let mut ok = false;
                while !ok {
                    target.x = rng.gen_range(0..map_size.x);
                    target.y = rng.gen_range(0..map_size.y);
                    ok = !navmesh.solid(target);
                }
            }*/
        //}
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
        if let Some(displacement) = displacement {
            if displacement.0.x != 0.0 {
                transform.scale.x = if displacement.0.x < 0.0 { -1.0 } else { 1.0 };
            }
        }
    }
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = if sprite.index == indices.last {
                indices.first
            } else {
                sprite.index + 1
            };
        }
    }
}

fn game_setup(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window: &Window = window_query.get_single().unwrap();

    // Background image
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("img/bg.png"),
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

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::hsl(120., 0.5, 0.2),
                custom_size: Some(Vec2::new(GAP_LEFT, window.height())),
                ..default()
            },
            transform: Transform::from_xyz(GAP_LEFT / 2.0, window.height() / 2.0, Layers::UI),
            ..default()
        },
        OnGameScreen,
    ));

    commands.insert_resource(GameData { tiles: 1 });
}

#[derive(Component)]
struct ArrowKeys;

fn move_with_keys(
    key_in: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Speed), With<ArrowKeys>>,
) {
    let mut dir = Vec3::ZERO;
    for (mut transform, speed) in &mut query {
        if key_in.pressed(KeyCode::Right) {
            dir.x += 1.0;
        }
        if key_in.pressed(KeyCode::Left) {
            dir.x -= 1.0;
        }
        if key_in.pressed(KeyCode::Up) {
            dir.y += 1.0;
        }
        if key_in.pressed(KeyCode::Down) {
            dir.y -= 1.0;
        }
        if dir.length() > 0.0 {
            dir = dir.normalize();
            transform.translation += dir * speed.speed * time.delta_seconds();
        }
    }
}

fn mouse_button_events(
    mut commands: Commands,
    mut events: EventReader<MouseButtonInput>,
    assets: Res<AssetServer>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    use bevy::input::ButtonState;

    let pos = windows.single().cursor_position();

    for ev in &mut events {
        match ev.state {
            ButtonState::Pressed => {
                // screen_print!("Mouse button press: {:?}", ev.button);
            }
            ButtonState::Released => {
                if let Some(position) = pos {
                    organism::create_random_organsim(&mut commands, &assets, position);
                }
            }
        }
    }
}
