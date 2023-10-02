use crate::prelude::*;
use crate::GameState;

#[derive(Resource, Default, Debug)]
pub struct Inventory {
    pub dirt: u8,
}

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), ui_setup);
    }
}

fn ui_setup(
    mut commands: Commands
) {
    commands.insert_resource(Inventory { dirt: 10 });
}
