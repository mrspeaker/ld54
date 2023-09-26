use bevy::{
    prelude::*,
    window::PrimaryWindow
};
use crate::{ despawn_screen, GameState };

pub const PLAYA_SPEED:f32 = 250.0;

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::InGame), game_setup)
            .add_systems(Update, (
                move_with_keys,
                confine_to_window
            ).run_if(in_state(GameState::InGame)))
            .add_systems(OnExit(GameState::InGame), despawn_screen::<OnGameScreen>);
    }
}

#[derive(Component)]
struct OnGameScreen;

#[derive(Component)]
struct Playa;

#[derive(Resource, Deref, DerefMut)]
struct GameTimer(Timer);

fn game_setup(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,    
    asset_server: Res<AssetServer>       
) {
    let window: &Window = window_query.get_single().unwrap();
    
    // Make the player
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("img/char.png"),
            transform: Transform::from_xyz(
                window.width() / 2.0,
                window.height() / 2.0 + 50.,
                1.0
            ),
            sprite: Sprite {
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..default()
            },
            ..default()
        },
        Playa,
        OnGameScreen));

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("img/bg.png"),
            transform: Transform::from_xyz(
                window.width() / 2.0,
                window.height() / 2.0,
                0.0
            ).with_scale(Vec3::new(1.8, 1.62, 0.0)),
            ..default()
        },
        OnGameScreen
    ));
}

fn move_with_keys(
    key_in: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Playa>>
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
){
    let (sprite, mut transform) = playa_query.single_mut() ;
    let window: &Window = window_query.get_single().unwrap();
    let hw = sprite.custom_size.unwrap_or(Vec2::ONE).x / 2.0;
    let hh = sprite.custom_size.unwrap_or(Vec2::ONE).y / 2.0;
    let x1 = hw;
    let x2 = window.width() - hw;
    let y1 = hh;
    let y2 = window.height() - hh;
    let mut t: Vec3 = transform.translation;
    if t.x < x1 { t.x = x1 }
    if t.x > x2 { t.x = x2 }
    if t.y < y1 { t.y = y1 }
    if t.y > y2 { t.y = y2 }
    transform.translation = t;
}
