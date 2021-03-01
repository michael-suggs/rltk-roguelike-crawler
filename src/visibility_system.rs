use specs::prelude::*;
use super::{Map, Position, Viewshed};
use rltk::{field_of_view, Point};

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        // System should be passed Map for use (no map is a failure)
        ReadExpect<'a, Map>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (map, mut viewshed, pos) = data;

        for (viewshed, pos) in (&mut viewshed, &pos).join() {
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
        }
    }
}
