use std::{mem::MaybeUninit, fmt::Debug, ops::{Sub, Add}};
use bevy::math::swizzles::Vec3Swizzles;

use crate::{prelude::*, game::{Speed,Displacement}, terrain::{GAP_LEFT, Tile}};
use bevy_ecs_tilemap::prelude::*;

#[derive(Component)]
pub struct FollowPath {
    pub end: Vec2,
    pub done: bool,
}

/// A an entity that can or cannot be navigated through while pathfinding.
#[derive(Component)]
pub struct Navmesh {
    tiles: Box<[bool]>,
    width: u32,
    height: u32,
}
impl Navmesh {
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        let mut data = Vec::new();
        data.resize((width * height) as usize, false);
        Self {
            tiles: data.into_boxed_slice(),
            width,
            height,
        }
    }

    /// Tiles out of bounds are considered solid.
    #[must_use]
    pub fn solid(&self, pos: TilePos) -> bool {
        if pos.x >= self.width || pos.y >= self.height {
            return true;
        }
        self.tiles[(self.height - 1 - pos.y) as usize * self.width as usize + pos.x as usize]
    }
    pub fn set_solid(&mut self, pos: TilePos, solid: bool) {
        self.tiles[(self.height - 1 - pos.y) as usize * self.width as usize + pos.x as usize] = solid;
    }
    #[must_use]
    fn empty_neighbours(&self, pos: TilePos) -> Successors {
        fn try_add(nav: &Navmesh, s: &mut Successors, pos: TilePos) {
            if !nav.solid(pos) {
                s.push(pos);
            }
        }
        let mut s = Successors::new();
        if let Some(y) = pos.y.checked_sub(1) {
            try_add(self, &mut s, TilePos { x: pos.x, y });
        }
        if let Some(x) = pos.x.checked_sub(1) {
            try_add(self, &mut s, TilePos { x, y: pos.y });
        }
        try_add(self, &mut s, TilePos { x: pos.x + 1, y: pos.y });
        try_add(self, &mut s, TilePos { x: pos.x, y: pos.y + 1 });
        s
    }
}

#[derive(Debug, Component)]
pub struct Pathfinding {
    pub path: Vec<TilePos>,
    pub at: usize,
}
impl Pathfinding {
    #[must_use]
    pub fn astar(
        navmesh: &Navmesh,
        from: TilePos,
        to: TilePos,
    ) -> Option<Self> {
        let succssors = |pos: &TilePos| navmesh.empty_neighbours(*pos);
        let heuristic = |from: &TilePos| (from.x.abs_diff(to.x) + from.y.abs_diff(to.y)) / 3;
        let success = |node: &_| to.eq(node);
        Some(Self {
            path: ::pathfinding::directed::astar::astar(&from, succssors, heuristic, success)?.0,
            at: 0
        })
    }
    /// Get the normalised direction vector to travel to the next node.
    #[must_use]
    pub fn current(&self, grid_size: &TilemapGridSize, map_type: &TilemapType) -> Vec2 {
        self.path[self.at].center_in_world(grid_size, map_type)
    }
    /// Increment the node that the pathing is at.
    /// Returns true if there was another node.
    pub fn step(&mut self) -> bool {
        if self.at + 1 >= self.path.len() {
            return false;
        }
        self.at += 1;
        true
    }
}

struct Successors {
    nodes: [MaybeUninit<TilePos>; 4],
    len: usize,
    index: usize,
}
impl Successors {
    pub fn new() -> Self {
        Self {
            // Safety: Assuming init in to another MaybeUninit type.
            nodes: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
            index: 0,
        }
    }
    /// # Panics
    /// Panics if the [`Successors`] list is full.
    pub fn push(&mut self, item: TilePos) {
        assert!(self.len < 4);
        self.nodes[self.len].write(item);
        self.len += 1;
    }
    /// Removes all items from the list.
    pub fn clear(&mut self) {
        self.nodes[..self.len]
            .iter_mut()
            .for_each(|i| unsafe { MaybeUninit::assume_init_drop(i) });
        self.len = 0;
        self.index = 0;
    }
}
impl Drop for Successors {
    fn drop(&mut self) {
        self.clear();
    }
}
impl std::ops::Deref for Successors {
    type Target = [TilePos];
    fn deref(&self) -> &Self::Target {
        unsafe {
            core::slice::from_raw_parts(
                self.nodes.as_ptr().cast::<TilePos>(),
                self.len
            )
        }
    }
}
impl Iterator for Successors {
    type Item = (TilePos, u32);
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            return None;
        }
        let i = unsafe { self.nodes[self.index].assume_init_read() };
        self.index += 1;
        Some((i, 1))
    }
}

pub fn follow_path(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Pathfinding, &mut Transform, &Speed, Option<&mut Displacement>), With<FollowPath>>,
    tilemap: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &TileStorage,
        &Navmesh,
    )>,
) {
    /// Distance to the target considered "at" the target.
    const TARGET_EPSILON: f32 = 5.0;
    let (_map_size, grid_size, map_type, _storage, _navmesh) = tilemap.single();
    let delta_time = time.delta_seconds();
    for (entity, mut path, mut transform, speed, displacement) in &mut query {
        //TODO: get size from entity
        let target = path.current(grid_size, map_type).add(Vec2 { x: GAP_LEFT + 25., y: 25. });

        let delta =
            target.sub(transform.translation.xy()).normalize() * delta_time * speed.speed;

        if let Some(mut displacement) = displacement {
            displacement.0 = delta;
        }
        transform.translation += delta.extend(0.0);
        if transform.translation.xy().distance(target) < TARGET_EPSILON && !path.step() {
            commands.entity(entity).remove::<Pathfinding>();
        }
    }
}

pub fn update_navmesh_on_tile_change(
    mut tile_query: Query<(&Tile, &TilePos), Or<(Added<Tile>, Changed<Tile>)>>,
    mut navmesh: Query<&mut Navmesh>
) {
    for (tile, tile_pos) in &mut tile_query {
        navmesh.get_single_mut().unwrap().set_solid(*tile_pos, match tile {
            Tile::Air => false,
            _ => true
        });
    }
}

pub fn remove_conflicting_paths_on_tile_change(
    mut commands: Commands,
    mut tile_query: Query<(&Tile, &TilePos), Changed<Tile>>,
    path: Query<(Entity, &mut Pathfinding), With<FollowPath>>,
) {
    for (tile, tile_pos) in &mut tile_query {
        // TODO: why *not* solid?! This runs after update_tile... so should be solid?
        if !Tile::is_solid(*tile) {
            // We drawing, invalidate any crossing paths
            for (ent, path) in path.iter() {
                if path.path.contains(tile_pos) {
                    commands.entity(ent).remove::<Pathfinding>();
                }
            }

        }
    }
}
