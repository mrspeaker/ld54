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
                // Doesn't update if using custom font on desktop?!
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

}

fn update_ui(
    //inv: Res<Inventory>,
    mut ui_dirt: Query<&mut Text, With<UIDirtAmount>>,
    game_data: Res<GameData>
) {
    //if inv.is_changed() {
        //let dirts: u32 = inv.dirt;
    for mut text in &mut ui_dirt {
        text.sections[0].value = format!("{}", game_data.eggs_spawned);
    }
    //}
}
