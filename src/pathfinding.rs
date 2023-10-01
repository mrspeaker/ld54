use crate::prelude::*;
use pathfinding;
use bevy_ecs_tilemap::prelude::*;

pub struct Pathfinding(Vec<TilePos>);
impl Pathfinding {
    pub fn astar(storage: &TileStorage, from: TilePos) -> Self {
        //pathfinding::directed::astar(from, )
        todo!()
    }
}


