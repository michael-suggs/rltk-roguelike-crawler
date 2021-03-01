use specs::prelude::*;
use super::{BlocksTile, Map, Position};

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, position, blockers) = data;

        // Sets up blocking for the terrain and blocking entities.
        map.populate_blocked();
        for (pos, _blocks) in (&position, &blockers).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            map.blocked[idx] = true;
        }
    }
}
