use crate::terrain::{GAP_LEFT, TILE_SIZE};
use crate::{despawn_screen, prelude::*, GameState};
use bevy::math::swizzles::Vec3Swizzles;
use bevy::{input::mouse::MouseButtonInput, prelude::*, window::PrimaryWindow};
use rand::Rng;

use crate::Layers;

pub const RUMBLEBEE_SPEED: f32 = 20.0;

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), game_setup)
            .add_systems(
                Update,
                (
                    assign_waypoints,
                    walk_path,
                    move_bob,
                    mouse_button_events,
                    cursor_position,
                    animate_sprite,
                    bevy::window::close_on_esc,
                )
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(OnExit(GameState::InGame), despawn_screen::<OnGameScreen>);
    }
}

#[derive(Component)]
struct FollowPath {
    end: Vec2,
    done: bool,
}

#[derive(Component)]
struct Bob;

#[derive(Component)]
struct OnGameScreen;

#[derive(Component)]
struct RumbleBee;

#[derive(Resource, Deref, DerefMut)]
struct GameTimer(Timer);

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Resource)]
struct GameData {
    tiles: usize,
}

fn assign_waypoints(
    mut query: Query<(&mut FollowPath, &Transform)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window: &Window = window_query.get_single().unwrap();

    let mut rng = rand::thread_rng();
    for (mut follow_path, _transform) in &mut query {
        if follow_path.done {
            let x: f32 = rng.gen_range(0.0..=1.0) * (window.width() - GAP_LEFT) + GAP_LEFT;
            let y: f32 = rng.gen_range(0.0..=1.0) * (window.height()- TILE_SIZE) + TILE_SIZE;

            follow_path.end = Vec2::new(x, y);
            follow_path.done = false;
        }
    }
}

fn walk_path(time: Res<Time>, mut query: Query<(&mut FollowPath, &mut Transform)>) {
    let dt = time.delta_seconds();
    for (mut follow_path, mut transform) in &mut query {
        if !follow_path.done {
            let p = transform.translation.xy();
            let end = follow_path.end;
            let path = end - p;

            let path_length = path.length();
            let dir = path / path_length;
            let mut movement_dist = RUMBLEBEE_SPEED * dt;

            if movement_dist >= path_length {
                movement_dist = path_length;
                follow_path.done = true;
            }

            let new_pos = p + (dir * movement_dist);

            transform.translation.x = new_pos.x;
            transform.translation.y = new_pos.y;
        }
    }
}

fn move_bob(time: Res<Time>, mut pos: Query<(&mut Transform, With<Bob>)>) {
    for (mut transform, _bob) in &mut pos {
        transform.translation.y += ((time.elapsed_seconds() + transform.translation.x) * 4.0).sin() * 0.1;
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
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let window: &Window = window_query.get_single().unwrap();

    let texture_handle = asset_server.load("img/gabe-idle-run.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 7, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    // Use only the subset of sprites in the sheet that make up the run animation
    let animation_indices = AnimationIndices { first: 1, last: 6 };

    // Background image
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("img/bg.png"),
            transform: Transform::from_xyz(
                window.width() / 2.0,
                window.height() / 2.0,
                Layers::BACKGROUND)
                .with_scale(Vec3::new(1.7, 1.4, 0.0)),
            ..default()
        },
        OnGameScreen,
    ));

    // Make the beez
    let mut rng = rand::thread_rng();
    let num_beez = 6;
    for i in 0..num_beez {
        let bee_pos = Vec3::new(
            rng.gen_range(0.0..=1.0) * (window.width() - GAP_LEFT) + GAP_LEFT,
            rng.gen_range(0.0..=1.0) * (window.height() - TILE_SIZE) + TILE_SIZE,
            Layers::MIDGROUND);

        let texture = asset_server.load(if i < num_beez / 2 {"img/beep.png"} else { "img/beeb.png" });

        commands.spawn((
            RumbleBee,
            SpriteBundle {
                texture,
                transform: Transform::from_translation(bee_pos),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(50.0, 50.0)),
                    ..default()
                },
                ..default()
            },
            OnGameScreen,
            FollowPath {
                end: bee_pos.xy(),
                done: true,
            },
            Bob
        ));
    }

    /*commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite: TextureAtlasSprite::new(animation_indices.first),
            transform: Transform::from_xyz(window.width() / 2.0, 100.0, 0.1)
                .with_scale(Vec3::splat(6.0)),
            ..default()
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
));*/

    commands.spawn((SpriteBundle {
        sprite: Sprite {
            color: Color::hsl(120., 0.5, 0.2),
            custom_size: Some(Vec2::new(GAP_LEFT, window.height())),
            ..default()
        },
        transform: Transform::from_xyz(GAP_LEFT/2.0, window.height()/2.0, Layers::UI),
        ..default()
    }, OnGameScreen));

    commands.insert_resource(GameData { tiles: 1 });
}

fn move_with_keys(
    key_in: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<RumbleBee>>,
) {
    let mut dir = Vec3::ZERO;
    let mut transform = query.single_mut();

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
        transform.translation += dir * RUMBLEBEE_SPEED * time.delta_seconds();
    }
}

fn confine_to_window(
    mut ent_q: Query<(&Sprite, &mut Transform), With<RumbleBee>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
        let window: &Window = window_query.get_single().unwrap();

    for (sprite, mut transform) in &mut ent_q {
        let hw = sprite.custom_size.unwrap_or(Vec2::ONE).x / 2.0;
        let hh = sprite.custom_size.unwrap_or(Vec2::ONE).y / 2.0;
        let x1 = hw;
        let x2 = window.width() - hw;
        let y1 = hh;
        let y2 = window.height() - hh;
        let mut t: Vec3 = transform.translation;
        if t.x < x1 {
            t.x = x1;
        }
        if t.x > x2 {
            t.x = x2;
        }
        if t.y < y1 {
            t.y = y1;
        }
        if t.y > y2 {
            t.y = y2;
        }
        transform.translation = t;
    }
}

fn cursor_position(windows: Query<&Window, With<PrimaryWindow>>) {
    if let Some(_position) = windows.single().cursor_position() {
        // screen_print!("Cursor pos {:?}", position);
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
