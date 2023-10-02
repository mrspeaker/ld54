use crate::prelude::*;
use crate::GameState;

pub const DIRT_AMOUNT:u8 = 10; // umm, why did I make it u8?!

#[derive(Resource, Default, Debug)]
pub struct Inventory {
    pub dirt: u8,
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
    assets: Res<AssetServer>,
) {
    commands.insert_resource(Inventory { dirt: DIRT_AMOUNT });

    commands.spawn((
        TextBundle::from_section(
            DIRT_AMOUNT.to_string(),
            TextStyle {
                // Doesn't update if using custom font on desktop?!
                //font: assets.load("font/FredokaOne-Regular.ttf"),
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
    inv: Res<Inventory>,
    mut ui_dirt: Query<&mut Text, With<UIDirtAmount>>
) {
    let dirts: u8 = inv.dirt;
    for mut text in &mut ui_dirt {
        text.sections[0].value = format!("{}", dirts);
    }
}
