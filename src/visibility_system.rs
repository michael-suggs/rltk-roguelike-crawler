use specs::prelude::*;
use super::{Map, Player, Position, Viewshed};
use rltk::{field_of_view, Point};

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        // System should be passed Map for use (no map is a failure)
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Player>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, entities, mut viewshed, pos, player) = data;

        for (ent, viewshed, pos) in (&entities, &mut viewshed, &pos).join() {
            // If player has been moved, update the viewshed.
            if viewshed.dirty {
                viewshed.dirty = false;
                // Start each loop by clearing the list of visible tiles.
                viewshed.visible_tiles.clear();
                // Get visible tiles for the current entity at position `pos` using
                // its visibility range from its viewshed.
                viewshed.visible_tiles = field_of_view(
                    Point::new(pos.x, pos.y),
                    viewshed.range,
                    &*map
                );
                // Deletes entries that don't meet the specified criteria; that is,
                // confines the visible tiles to only those within the map bounds.
                viewshed.visible_tiles.retain(
                    |p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height
                );

                // If this is the player, reveal the tiles they can see.
                let _p: Option<&Player> = player.get(ent);
                if let Some(_p) = _p {
                    // Set all to non-visible to start.
                    for t in map.visible_tiles.iter_mut() { *t = false };
                    // Update tiles in our currently visible range.
                    for vis in viewshed.visible_tiles.iter() {
                        let idx = map.xy_idx(vis.x, vis.y);
                        map.revealed_tiles[idx] = true;
                        map.visible_tiles[idx] = true;
                    }
                }
            }
        }
    }
}
