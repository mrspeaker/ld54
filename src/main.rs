#![warn(clippy::pedantic)]
#![allow(dead_code)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::needless_pass_by_value)]

pub mod game;
pub mod logo;
pub mod organism;
pub mod pathfinding;
pub mod splash;
pub mod terrain;

use std::sync::OnceLock;

use bevy::window::PrimaryWindow;
use bevy::{asset::HandleId, prelude::*};
use bevy_debug_text_overlay::{screen_print, OverlayPlugin};

pub mod prelude {
    pub use bevy::prelude::*;
    pub use bevy_debug_text_overlay::screen_print;
    pub use macros::gene;

    pub use crate::{organism, terrain, FONT};
}

pub static FONT: OnceLock<HandleId> = OnceLock::new();

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
        .add_plugins((
            logo::LogoPlugin,
            splash::SplashPlugin,
            game::GamePlugin,
            terrain::TerrainPlugin
        ))
        .run();
}

fn setup(mut commands: Commands,     
    window_query: Query<&Window, With<PrimaryWindow>>,
    assets: Res<AssetServer>) {
    screen_print!(sec: 3.0, "Run main setup.");
    let window: &Window = window_query.get_single().unwrap();
    FONT.set(assets.load::<Font, _>("font/FredokaOne-Regular.ttf").id())
        .unwrap();

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

