use crate::pathfinding::follow_path;
use crate::pointer::Pointer;
use crate::terrain::GAP_LEFT;
use crate::{despawn_screen, GameState, AssetCol};
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_kira_audio::prelude::*;

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
                    check_exit,
                    move_bob,
                    follow_path,
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
pub struct Wander;

#[derive(Component)]
pub struct Fight {
    opponent: Entity,
}

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
                transform.scale.x = transform.scale.x.abs() * if displacement.0.x < 0.0 { -1.0 } else { 1.0 };
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
    assets: Res<AssetCol>,
    _audio: Res<Audio>
) {
    let window: &Window = window_query.get_single().unwrap();

    /*audio
        .play(assets.tune.clone())
        .loop_from(0.0)
        .fade_in(AudioTween::new(
            Duration::from_secs(2),
            AudioEasing::OutPowi(2),
        ))
        .with_volume(0.1);*/

    // Background image
    commands.spawn((
        SpriteBundle {
            texture: assets.bg.clone(),
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

    // UI bit
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::hsla(120., 0.5, 0.2, 0.5),
                custom_size: Some(Vec2::new(GAP_LEFT, window.height())),
                ..default()
            },
            transform: Transform::from_xyz(GAP_LEFT / 2.0, window.height() / 2.0, Layers::UI),
            ..default()
        },
        OnGameScreen,
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::hsl(20., 0.5, 0.1),
                custom_size: Some(Vec2::new(GAP_LEFT * 0.8, 50.0)),
                ..default()
            },
            transform: Transform::from_xyz(GAP_LEFT / 2.0, 40.0, Layers::UI),
            ..default()
        },
        OnGameScreen,
    ));

    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            "exit",
            TextStyle {
                font: assets.font.clone(),
                font_size: 24.0,
                color: Color::WHITE,
            },
        ) // Set the alignment of the Text
        .with_text_alignment(TextAlignment::Center)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(30.0),
            left: Val::Px(20.0),
            ..default()
        }),
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


fn check_exit(
    pointer: Res<Pointer>,
    mut game_state: ResMut<NextState<GameState>>,
){
    if pointer.released && pointer.pos.x < GAP_LEFT * 0.95 {
        // transition to splash.
        game_state.set(GameState::Splash);
    }
}
