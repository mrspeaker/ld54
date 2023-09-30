#![warn(clippy::pedantic)]
#![allow(dead_code)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::needless_pass_by_value)]

pub mod game;
pub mod logo;
pub mod organism;
pub mod splash;

use std::sync::OnceLock;

use bevy::window::PrimaryWindow;
use bevy::{asset::HandleId, prelude::*};
use bevy_debug_text_overlay::{screen_print, OverlayPlugin};
use bevy::sprite::MaterialMesh2dBundle;

pub mod prelude {
    pub use bevy::prelude::*;
    pub use bevy_debug_text_overlay::screen_print;
    pub use macros::gene;

    pub use crate::{organism, FONT};
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
        .add_plugins((logo::LogoPlugin, splash::SplashPlugin, game::GamePlugin))
        .run();
}

fn setup(mut commands: Commands,     
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
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
    
    pub fn color_from_char(byte: u8) -> Color {
        let c = byte as char;
        match c {
            '#' => Color::GRAY,
            '.' => Color::BLUE,
            _ => panic!("Bad char in tilemap"),
        }
    }

    const TILE_SIZE: f32 = 20.0;
    
    //            1         2         3         4         5         6         7         
    //  01234567890123456789012345678901234567890123456789012345678901234567890123456789
    let tilemap = b"\
        ................................................................................\
        ................................................................................\
        ................................................................................\
        ................................................................................\
        ................................................................................\
        ########........................................................................\
        ##############..................................................................\
        .........#####..................................................................\
        ................................................................................\
        ................................................................................\
        .....................................................................###########\
        ......................................................................##########\
        ..........................................................#.....................\
        ..........................................................##....................\
        ################..........................................###....###############\
        ##################....................................##########################\
        #####################....###....######.............###.#########################\
        ################################################################################\
        ################################################################################\
        ################################################################################";

    let tile_x_count = 80;
    let tile_y_count = tilemap.len() / tile_x_count;

    for y in 0..tile_y_count {
        for x in 0..tile_x_count {
            
            let tile_centre_offset = Vec3::new(TILE_SIZE, TILE_SIZE, 0.0) / 2.0;
            let pos = Vec3::new(x as f32, y as f32, 0.0) * TILE_SIZE + tile_centre_offset;
            
            let tilemap_index = (((tile_y_count - 1) - y) * tile_x_count) + x;
            let color = color_from_char(tilemap[tilemap_index]);

            commands.spawn(MaterialMesh2dBundle {
                mesh: meshes
                    .add(shape::Quad::new(Vec2::new(TILE_SIZE, TILE_SIZE)).into())
                    .into(),
                material: materials.add(ColorMaterial::from(color)),
                transform: Transform::from_translation(pos),
                ..default()
            });
        }
    }
        
}

fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}

