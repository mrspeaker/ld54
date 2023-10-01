use std::mem::MaybeUninit;

use crate::prelude::*;
use bevy_ecs_tilemap::prelude::*;

/// A an entity that can or cannot be navigated through while pathfinding.
#[derive(Component, Clone, Copy)]
pub struct Navigatable(bool);

#[derive(Debug, Component)]
pub struct Pathfinding(Vec<TilePos>);
impl Pathfinding {
    #[must_use]
    pub fn astar(
        storage: &TileStorage,
        entities: Query<&Navigatable>,
        from: TilePos,
        to: TilePos,
    ) -> Option<Self> {
        let succssors = |node: &TilePos| {
            let mut s = Successors::new();
            // Filter out non-navigatable nodes
            #[allow(clippy::identity_op)]
            [
                TilePos {
                    x: node.x - 1,
                    y: node.y - 1,
                },
                TilePos {
                    x: node.x + 0,
                    y: node.y - 1,
                },
                TilePos {
                    x: node.x + 1,
                    y: node.y - 1,
                },
                TilePos {
                    x: node.x - 1,
                    y: node.y + 0,
                },
                TilePos {
                    x: node.x + 1,
                    y: node.y + 0,
                },
                TilePos {
                    x: node.x - 1,
                    y: node.y + 1,
                },
                TilePos {
                    x: node.x + 0,
                    y: node.y + 1,
                },
                TilePos {
                    x: node.x + 1,
                    y: node.y + 1,
                },
            ]
            .iter()
            .filter(|pos| {
                let Some(entity) = storage.get(pos) else {
                    return false;
                };
                entities.get(entity).map_or(false, |nav| nav.0)
            })
            .for_each(|n| s.push(*n));
            s
        };
        let heuristic = |from: &TilePos| (from.x.abs_diff(to.x) + from.y.abs_diff(to.y)) / 3;
        let success = |node: &_| to.eq(node);
        Some(Self(
            ::pathfinding::directed::astar::astar(&from, succssors, heuristic, success)?.0,
        ))
    }
}

struct Successors {
    nodes: [MaybeUninit<TilePos>; 8],
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
        assert!(self.len < 8);
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
