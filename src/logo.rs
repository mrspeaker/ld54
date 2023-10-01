use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::{despawn_screen, GameState};

pub struct LogoPlugin;

impl Plugin for LogoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Logo), setup)
            .add_systems(Update, (countdown).run_if(in_state(GameState::Logo)))
            .add_systems(OnExit(GameState::Logo), despawn_screen::<OnLogoScreen>);
    }
}

#[derive(Component)]
struct OnLogoScreen;

#[derive(Resource, Deref, DerefMut)]
struct ScreenTimer(Timer);

fn setup(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window: &Window = window_query.get_single().unwrap();

    commands.insert_resource(ScreenTimer(Timer::from_seconds(2.0, TimerMode::Once)));

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("img/goodsoup.png"),
            transform: Transform::from_xyz(window.width() / 2.0, window.height() * 0.5, 1.0)
                .with_scale(Vec3::splat(0.5)),
            ..default()
        },
        OnLogoScreen,
    ));

    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            "good soup",
            TextStyle {
                font: asset_server.load("font/FredokaOne-Regular.ttf"),
                font_size: 80.0,
                color: Color::WHITE,
            },
        ) // Set the alignment of the Text
        .with_text_alignment(TextAlignment::Center)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(15.0),
            ..default()
        }),
        OnLogoScreen,
    ));
}

fn countdown(
    mut game_state: ResMut<NextState<GameState>>,
    time: Res<Time>,
    mut timer: ResMut<ScreenTimer>,
) {
    if timer.tick(time.delta()).finished() {
        game_state.set(GameState::Splash);
    }
}
