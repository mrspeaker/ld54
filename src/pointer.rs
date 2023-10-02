use bevy::input::{mouse::MouseButtonInput, ButtonState};

use crate::prelude::*;
use crate::terrain::Tile;

#[derive(Resource, Debug)]
pub struct Pointer {
    pub pos: Vec2,
    pub is_down: bool,
    pub pressed: bool,
    pub released: bool,
    pub tile: Tile,
}
impl Pointer {
    pub fn set_active_item(&mut self, tile: Tile) {
        if self.pressed {
            self.tile = match tile {
                Tile::Air => Tile::Dirt { style: 1, topsoil: true } ,
                Tile::Rock { .. } => tile,
                _ => Tile::Air,
            };
            self.pressed = false;
            self.is_down = match self.tile {
                Tile::Rock { .. } => false,
                _ => true
            }
        }
    }
}

impl Default for Pointer {
    fn default() -> Self {
        Pointer {
            pos: Vec2::new(-1000.0, -1000.0),
            is_down: false,
            pressed: false,
            released: false,
            tile: Tile::Unknown,
        }
    }
}

pub fn update_pointer(
    camera_q: Query<(&GlobalTransform, &Camera)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut pointer: ResMut<Pointer>,
    mut events: EventReader<MouseButtonInput>,
) {
    for cursor_moved in &mut cursor_moved_events {
        for (cam_t, cam) in camera_q.iter() {
            if let Some(pos) = cam.viewport_to_world_2d(cam_t, cursor_moved.position) {
                pointer.pos = pos;
                // TODO: figure out how to set this reliably.
                // Currently calling pointer.pressed = false after handling in highlight-tile
                //pointer.pressed = false;
                //pointer.released = false;
                for ev in &mut events {
                    match ev.state {
                        ButtonState::Pressed => {
                            pointer.is_down = true;
                            pointer.pressed = true;
                        }
                        ButtonState::Released => {
                            pointer.is_down = false;
                            pointer.released = true;
                        }
                    }
                }
            }
        }
    }
}
