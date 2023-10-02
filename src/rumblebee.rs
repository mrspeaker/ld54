use crate::game::OnGameScreen;
use crate::pathfinding::FollowPath;
use crate::terrain::{GAP_LEFT, TILE_SIZE};
use crate::{prelude::*, GameState};
use bevy::math::swizzles::Vec3Swizzles;
use bevy::{prelude::*, window::PrimaryWindow};
use rand::Rng;

use crate::Layers;

pub const RUMBLEBEE_SPEED: f32 = 50.0;

pub struct RumblebeePlugin;
impl Plugin for RumblebeePlugin {

    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::InGame), rumblebee_setup);
    }

}

#[derive(Component)]
pub struct RumbleBee {
    pub faction: terrain::Faction,
}

fn rumblebee_setup(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
){
    let window: &Window = window_query.get_single().unwrap();

    // Make the beez
    let mut rng = rand::thread_rng();
    let num_beez = 6;
    for i in 0..num_beez {
        let bee_pos = Vec3::new(
            rng.gen_range(0.0..=1.0) * (window.width() - GAP_LEFT) + GAP_LEFT,
            rng.gen_range(0.0..=1.0) * (window.height() - TILE_SIZE) + TILE_SIZE,
            Layers::MIDGROUND,
        );

        let texture = asset_server.load(if i < num_beez / 2 {
            "img/beep.png"
        } else {
            "img/beeb.png"
        });

        commands.spawn((
            RumbleBee {
                faction: terrain::Faction::random(),
            },
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
        ));


        /*
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,

    let texture_handle = asset_server.load("img/gabe-idle-run.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 7, 1, None, None);

    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    // Use only the subset of sprites in the sheet that make up the run animation
    let animation_indices = AnimationIndices { first: 1, last: 6 };


        commands.spawn((
            SpriteSheetBundle {
                texture_atlas: texture_atlas_handle,
                sprite: TextureAtlasSprite::new(animation_indices.first),
                transform: Transform::from_xyz(window.width() / 2.0, 100.0, 0.1)
                    .with_scale(Vec3::splat(6.0)),
                ..default()
            },
            animation_indices,
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        ));
        */


    }

}
