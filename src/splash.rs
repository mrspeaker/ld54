use bevy::input::keyboard::KeyboardInput;
use bevy::{prelude::*, input::ButtonState};
use bevy::window::PrimaryWindow;

use crate::AssetCol;
use crate::{despawn_screen, GameState};

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Splash), splash_setup)
            .add_systems(Update, (countdown).run_if(in_state(GameState::Splash)))
            .add_systems(OnExit(GameState::Splash), despawn_screen::<OnSplashScreen>);
    }
}

#[derive(Component)]
struct OnSplashScreen;

#[derive(Resource, Deref, DerefMut)]
struct SplashTimer(Timer);

fn splash_setup(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    assets: Res<AssetCol>,
) {
    let window: &Window = window_query.get_single().unwrap();

    commands.insert_resource(SplashTimer(Timer::from_seconds(5.0, TimerMode::Once)));

    // Background image
    commands.spawn((
        SpriteBundle {
            texture: assets.bg.clone(),
            transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0)
                .with_scale(Vec3::new(1.7, 1.4, 0.0)),
            ..default()
        },
        OnSplashScreen,
    ));

    // Character sprite
    commands.spawn((
        SpriteBundle {
            texture: assets.splash.clone(),
            transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 1.0)
                .with_scale(Vec3::new(0.34, 0.3, 1.0)),
            ..default()
        },
        OnSplashScreen,
    ));
}

fn countdown(
    mut game_state: ResMut<NextState<GameState>>,
    mut key_evr: EventReader<KeyboardInput>,
    buttons: Res<Input<MouseButton>>,
    touches: Res<Touches>,
) {
    //if timer.tick(time.delta()).finished()
    let mut done = false;
    for ev in key_evr.iter() {
        match ev.state {
            ButtonState ::Released => {
                done = true;
            },
            _ => ()
        };
    }
    for _ in buttons.get_just_released() {
        done = true;
    }
    for finger in touches.iter() {
        if touches.just_pressed(finger.id()) {
            done = true;
        }
    }
    if done {
        game_state.set(GameState::InGame);
    }
}
