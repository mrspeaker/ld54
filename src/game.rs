use crate::{despawn_screen, GameState};
use bevy::{prelude::*, window::PrimaryWindow, input::mouse::MouseButtonInput};
use bevy_debug_text_overlay::screen_print;

pub const PLAYA_SPEED: f32 = 250.0;

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), game_setup)
            .add_systems(
                Update,
                (
                    move_with_keys,
                    mouse_button_events,
                    cursor_position,
                    confine_to_window,
                    animate_sprite)
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(OnExit(GameState::InGame), despawn_screen::<OnGameScreen>);
    }
}

#[derive(Component)]
struct OnGameScreen;

#[derive(Component)]
struct Playa;

#[derive(Resource, Deref, DerefMut)]
struct GameTimer(Timer);

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

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

    // Make the player
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("img/char.png"),
            transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0 + 50., 1.0),
            sprite: Sprite {
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..default()
            },
            ..default()
        },
        Playa,
        OnGameScreen,
    ));

    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite: TextureAtlasSprite::new(animation_indices.first),
            transform: Transform::from_xyz(window.width() / 2.0, 100.0, 1.0)
                .with_scale(Vec3::splat(6.0)),
            ..default()
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("img/bg.png"),
            transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0)
                .with_scale(Vec3::new(1.8, 1.62, 0.0)),
            ..default()
        },
        OnGameScreen,
    ));
}

fn move_with_keys(
    key_in: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Playa>>,
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
        transform.translation += dir * PLAYA_SPEED * time.delta_seconds();
    }
}

fn confine_to_window(
    mut playa_query: Query<(&Sprite, &mut Transform), With<Playa>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let (sprite, mut transform) = playa_query.single_mut();
    let window: &Window = window_query.get_single().unwrap();
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

fn cursor_position(
    q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    if let Some(position) = q_windows.single().cursor_position() {
        screen_print!("Cursor pos {:?}", position);
    }
}


fn mouse_button_events(
    mut mousebtn_evr: EventReader<MouseButtonInput>,
) {
    use bevy::input::ButtonState;

    for ev in mousebtn_evr.iter() {
        match ev.state {
            ButtonState::Pressed => {
                screen_print!("Mouse button press: {:?}", ev.button);
            }
            ButtonState::Released => {
                screen_print!("Mouse button release: {:?}", ev.button);
            }
        }
    }
}
