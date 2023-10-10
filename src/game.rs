use std::ops::Sub;

use crate::pathfinding::{Navmesh, follow_path, Pathfinding};
use crate::rumblebees::{RumbleBee, BeeFight, Beenitialized, Army};
use crate::terrain::{GAP_LEFT, Egg};
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
                    mouse_button_events,
                    move_bob,
                    find_target,
                    follow_path,
                    bee_fight_collisions,
                    bee_egg_collisions,
                    bee_fight,
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
    pub frames: Vec<usize>,
    pub cur: usize
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
    entity: Query<
            (Entity, &Transform, &RumbleBee),
        (Without<Pathfinding>, Without<BeeFight>)>,
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
            // info!("Entity outside map {:?} {} {}", &entity.1.translation.xy(), map_size.x as f32 * grid_size.x, map_size.y as f32 * grid_size.y);
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
                //commands.entity(entity.0).insert(path);
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
            ButtonState::Released => {
                if let Some(position) = pos {
                    organism::create_random_organsim(&mut commands, &assets, position);
                }
            }
            _ => ()
        }
    }
}

fn bee_egg_collisions(
    mut commands: Commands,
    beez: Query<(Entity, &RumbleBee, &Transform)>,
    eggs: Query<(Entity, &Egg, &TilePos)>,
    tilemap: Query<&TilemapGridSize>
){
    let grid_size = tilemap.single();

    for (_bee_ent, _bee, bee_pos) in beez.iter() {
        for (egg_ent, _egg, egg_pos) in eggs.iter() {
            let pos = Vec3 {
                x: egg_pos.x as f32 * grid_size.x,
                y: egg_pos.y as f32 * grid_size.y,
                z: bee_pos.translation.z
            };

            if bee_pos.translation.distance(pos) < 50.0 {
                info!("hit a beee egg");
                commands.entity(egg_ent).remove::<Egg>();
            }
        }
    }
}


fn bee_fight_collisions(
    mut commands: Commands,
    beez: Query<(Entity, &RumbleBee, &Transform), (Without<BeeFight>, Without<Beenitialized>)>
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
            }
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
