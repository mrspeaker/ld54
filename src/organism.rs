use rand::Rng;

use crate::{prelude::*, AssetCol};

pub mod gene;

pub fn create_random_organsim(
    commands: &mut Commands,
    assets: Res<AssetCol>,
    position: Vec2)
{
    //let font = assets.get_handle(*FONT.get().unwrap());
    commands
        .spawn(Organism::random(position))
        .with_children(|b| Organism::random_parts(b, assets.font.clone(), (100.0, 100.0)));
}

#[derive(Debug, Bundle)]
pub struct Organism {
    visibility: Visibility,
    computed_visibility: ComputedVisibility,
    local: Transform,
    global: GlobalTransform,
}
impl Organism {
    #[must_use]
    pub fn random(Vec2 { x, y }: Vec2) -> Self {
        Self {
            visibility: Visibility::Visible,
            computed_visibility: ComputedVisibility::default(),
            local: Transform::from_xyz(x, y, 1.0),
            global: GlobalTransform::default(),
        }
    }
    pub fn random_parts(b: &mut ChildBuilder, font: Handle<Font>, (x, y): (f32, f32)) {
        b.spawn(Text2dBundle {
            text: Text::from_section(
                random_name(),
                TextStyle {
                    font,
                    font_size: 60.0,
                    color: Color::GREEN,
                },
            ),
            transform: Transform::from_xyz(x, y, 1.0),
            ..Default::default()
        });
        //NameTag::new(font, random_name(), position));
        b.spawn(Torso);
    }
}

pub struct NameTag;
impl NameTag {}

/// A body part.
#[derive(Debug, Component)]
pub struct Torso;
impl Torso {}

#[must_use]
pub fn random_name() -> String {
    const NO_PREFIX_CHANCE: usize = 4;
    const PREFIX: &[&str] = &["Mc", "El"];
    const NO_SUFFIX_CHANCE: usize = 4;
    const SUFFIX: &[&str] = &["son", "y", "athy"];
    const FIRST: &[&str] = &["John", "Daniel", "Tom"];
    const LAST: &[&str] = &["Daniels", "Patrik", "Col"];
    let mut rng = rand::thread_rng();
    let mut name = String::with_capacity(32);
    if let Some(prefix) = PREFIX.get(rng.gen_range(0..PREFIX.len() + NO_PREFIX_CHANCE)) {
        name += prefix;
    }
    name += FIRST[rng.gen_range(0..FIRST.len())];
    if let Some(suffix) = SUFFIX.get(rng.gen_range(0..SUFFIX.len() + NO_SUFFIX_CHANCE)) {
        name += suffix;
    }
    name.push(' ');
    if let Some(prefix) = PREFIX.get(rng.gen_range(0..PREFIX.len() + NO_PREFIX_CHANCE)) {
        name += prefix;
    }
    name += LAST[rng.gen_range(0..LAST.len())];
    if let Some(suffix) = SUFFIX.get(rng.gen_range(0..SUFFIX.len() + NO_SUFFIX_CHANCE)) {
        name += suffix;
    }

    name
}
