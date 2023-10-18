#![warn(clippy::pedantic)]
#![allow(dead_code)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::needless_pass_by_value)]

extern crate console_error_panic_hook;

mod debug;
pub mod game;
pub mod logo;
pub mod organism;
pub mod pathfinding;
pub mod splash;
pub mod terrain;
pub mod pointer;
pub mod inventory;
pub mod rumblebees;
pub mod settings;

use std::sync::OnceLock;

use bevy::window::{Cursor, CursorIcon, PrimaryWindow};
use bevy::{asset::HandleId, prelude::*};
use bevy_debug_text_overlay::{screen_print, OverlayPlugin};
use bevy_kira_audio::prelude::*;
use bevy_asset_loader::prelude::*;
//use debug::DebugPlugin;

pub mod prelude {
    pub use bevy::prelude::*;
    pub use bevy_debug_text_overlay::screen_print;
    pub use bevy_ecs_tilemap::prelude::*;
    pub use macros::gene;

    pub use crate::{organism, pathfinding::{Navmesh, Pathfinding}, terrain, FONT};
}

pub static FONT: OnceLock<HandleId> = OnceLock::new();

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Loading,
    Logo,
    Splash,
    InGame,
}

pub struct Layers;
impl Layers {
    pub const MOST_BACK: f32 = 0.0;
    pub const BACKGROUND: f32 = 25.0;
    pub const MIDGROUND: f32 = 50.;
    pub const FOREGROUND: f32 = 75.0;
    pub const MOST_FRONT: f32 = 100.0;
    pub const UI: f32 = 150.0;
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Rumblebees".into(),
                    canvas: Some("#game".to_owned()),
                    resolution: (500.0 * 2.0, 300.0 * 2.0).into(),
                    fit_canvas_to_parent: false,
                    cursor: Cursor {
                        icon: CursorIcon::Crosshair,
                        visible: true,
                        ..default()
                    },
                    ..default()
                }),
                ..default()
            }),
            //.set(ImagePlugin::default_nearest()),
            AudioPlugin,
        ))
        .add_loading_state(LoadingState::new(GameState::Loading).continue_to_state(GameState::Splash))
        .add_collection_to_loading_state::<_, AssetCol>(GameState::Loading)
        .add_plugins(OverlayPlugin {
            font_size: 14.0,
            ..default()
        })
        //.add_plugins(DebugPlugin)
        .add_state::<GameState>()
        .add_systems(Startup, setup)
        .add_plugins((
            logo::LogoPlugin,
            splash::SplashPlugin,
            game::GamePlugin,
            terrain::TerrainPlugin,
            inventory::UIPlugin,
            rumblebees::RumblebeePlugin
        ))
        .run();
}

#[derive(AssetCollection, Resource)]
pub struct AssetCol {
    #[asset(path = "img/bg.png")]
    bg: Handle<Image>,
    #[asset(path = "img/rumblebees-splash.png")]
    splash: Handle<Image>,

    #[asset(texture_atlas(tile_size_x = 80.0, tile_size_y = 80.0, columns = 6, rows = 2))]
    #[asset(path = "img/chars.png")]
    pub chars: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 102.0, tile_size_y = 80.0, columns = 2, rows = 1))]
    #[asset(path = "img/arms.png")]
    pub arms: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 40.0, tile_size_y = 40.0, columns = 16, rows = 5))]
    #[asset(path = "img/tiles.png")]
    pub tiles: Handle<TextureAtlas>,

    #[asset(path = "sounds/blip.ogg")]
    blip: Handle<AudioSource>,
    #[asset(path = "sounds/test.ogg")]
    tune: Handle<AudioSource>,
    #[asset(path = "font/FredokaOne-Regular.ttf")]
    font: Handle<Font>,
}

fn setup(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    //assets: Res<AssetCol>
) {
    screen_print!(sec: 3.0, "Run main setup.");
    let window: &Window = window_query.get_single().unwrap();
    /*FONT.set(assets.font.id())
        .unwrap();*/

    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
        ..default()
    });

    console_error_panic_hook::set_once();
}

fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
