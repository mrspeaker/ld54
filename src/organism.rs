use crate::prelude::*;

pub mod gene;

pub fn create_random_organsim(mut commands: Commands) {
    commands.spawn(Organism::random());
}

#[derive(Debug, Component)]
pub struct Organism {
    /// A name for the player to empathise with.
    name: String,
}
impl Organism {
    pub fn random() -> Self {
        Self {
            name: "todo".into(),
        }
    }
}

/// A body part.
pub struct Part {
    region: Region,
    // sprite: Sprite
}
pub enum Region {
    Torso,
    Head,
}
