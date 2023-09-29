mod game;
mod logo;
mod splash;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_debug_text_overlay::{screen_print, OverlayPlugin};

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Logo,
    Splash,
    InGame,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "LD54".into(),
                canvas: Some("#game".to_owned()),
                resolution: (500.0 * 2.0, 300.0 * 2.0).into(),
                fit_canvas_to_parent: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(OverlayPlugin {
            font_size: 14.0,
            ..default()
        })
        .add_state::<GameState>()
        .add_systems(Startup, setup)
        .add_plugins((logo::LogoPlugin, splash::SplashPlugin, game::GamePlugin))
        .run();
}

fn setup(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
    screen_print!(sec: 3.0, "Run main setup.");
    let window: &Window = window_query.get_single().unwrap();

    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
        ..default()
    });
}

fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
