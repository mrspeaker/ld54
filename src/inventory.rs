use crate::AssetCol;
use crate::game::GameData;
use crate::game::OnGameScreen;
use crate::prelude::*;
use crate::GameState;

pub const DIRT_AMOUNT:u32 = 64;

#[derive(Resource, Default, Debug)]
pub struct Inventory {
    pub dirt: u32,
}

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::InGame), ui_setup)
            .add_systems(
                Update,
                update_ui.run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(Component)]
struct UIDirtAmount;

#[derive(Component)]
struct UIGameOver;

fn ui_setup(
    mut commands: Commands,
    assets: Res<AssetCol>
) {
    commands.insert_resource(Inventory { dirt: DIRT_AMOUNT });

    commands.spawn((
        OnGameScreen,
        TextBundle::from_section(
            DIRT_AMOUNT.to_string(),
            TextStyle {
                font: assets.font.clone(),
                font_size: 50.0,
                color: Color::WHITE,
                ..default()
            },
        )
            .with_text_alignment(TextAlignment::Center)
            .with_style(Style {
                position_type: PositionType::Absolute,
                left: Val::Px(15.0),
                top: Val::Px(15.0),
                ..default()
            }),
        UIDirtAmount));

    commands.spawn((
        OnGameScreen,
        TextBundle::from_section(
            " ", //TODO: figure out how to toggle visibility, not text!
            TextStyle {
                font: assets.font.clone(),
                font_size: 100.0,
                color: Color::WHITE,
                ..default()
            },
        )
            .with_text_alignment(TextAlignment::Center)
            .with_style(Style {
                position_type: PositionType::Absolute,
                left: Val::Px(250.0),
                top: Val::Px(250.0),
                ..default()
            }),
        UIGameOver));

}

fn update_ui(
    mut ui_dirt: Query<&mut Text, (With<UIDirtAmount>, Without<UIGameOver>)>,
    mut ui_game_over: Query<&mut Text, With<UIGameOver>>,
    game_data: Res<GameData>
) {
    for mut text in &mut ui_dirt {
        text.sections[0].value = format!("{}", game_data.eggs_spawned);
    }

    // TODO: should just toggle visiblity
    if !game_data.game_started {
        for mut text in &mut ui_game_over {
            text.sections[0].value = format!("{}", "");
        }
    }
    if game_data.game_over {
        for mut text in &mut ui_game_over {
            text.sections[0].value = format!("{}", "GAME OVER");
        }
    }
}
