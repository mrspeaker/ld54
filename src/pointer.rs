use crate::prelude::*;
use crate::terrain::Tile;
use bevy::input::touch::{Touches,TouchPhase};

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
            self.is_down = match self.tile {
                Tile::Rock { .. } => false,
                _ => true
            };

            // TODO: this is now responsible for clearing pressed,
            // but shouldn't be. Should be done once per frame (at the beginning)
            self.pressed = false;

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
    mouse: Res<Input<MouseButton>>,
    touches: Res<Touches>,
    mut touch_evr: EventReader<TouchInput>,
) {
    let mut touch_move = false;
    for ev in touch_evr.iter() {
        match ev.phase {
            TouchPhase::Moved => {
                touch_move = true;
            }
            _ => ()
        }
    }

    for finger in touches.iter() {
        if touches.just_pressed(finger.id()) {
            println!("A new touch with ID {} just began.", finger.id());
        }
        touch_move = true;

        debug!(
            "Finger {} is at position ({},{}), started from ({},{}).",
            finger.id(),
            finger.position().x,
            finger.position().y,
            finger.start_position().x,
            finger.start_position().y,
        );
        pointer.pos.x = finger.position().x;
        pointer.pos.y = finger.position().y;
    }

    pointer.pressed
        = mouse.just_pressed(MouseButton::Left)
        || touches.any_just_pressed();
    pointer.released = mouse.just_released(MouseButton::Left)
        || touches.any_just_released();
    pointer.is_down = mouse.pressed(MouseButton::Left) || touch_move;

    for cursor_moved in &mut cursor_moved_events {
        for (cam_t, cam) in camera_q.iter() {
            if let Some(pos) = cam.viewport_to_world_2d(cam_t, cursor_moved.position) {
                pointer.pos = pos;
            }
        }
    }
}
