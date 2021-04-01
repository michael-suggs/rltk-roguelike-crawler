use super::{BlocksTile, Map, Position};
use specs::prelude::*;

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, position, blockers, entities) = data;

        // Sets up blocking for the terrain and blocking entities.
        map.populate_blocked();
        map.clear_content_index();
        for (ent, pos) in (&entities, &position).join() {
            let idx = map.xy_idx(pos.x, pos.y);

            // If blocking, update the block list.
            let _p: Option<&BlocksTile> = blockers.get(ent);
            if let Some(_p) = _p {
                map.blocked[idx] = true;
            }

            // Push entity to appropriate index slot.
            map.tile_content[idx].push(ent);
        }
    }
}
