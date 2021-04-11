use super::{common::draw_corridor, Map, MapBuilder, Position, Rect, TileType};
use crate::{spawner, BuildData, InitialMapBuilder, SHOW_MAPGEN_VISUALIZER};
use rltk::RandomNumberGenerator;
use specs::prelude::*;

const MIN_ROOM_SIZE: i32 = 8;

pub struct BspInteriorBuilder {
    rects: Vec<Rect>,
}

impl InitialMapBuilder for BspInteriorBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut crate::BuildData) {
        self.build(rng, build_data);
    }
}

impl BspInteriorBuilder {
    pub fn new(new_depth: i32) -> Box<BspInteriorBuilder> {
        Box::new(BspInteriorBuilder { rects: Vec::new() })
    }

    /// Creates a new BspInterior map.
    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuildData) {
        let mut rooms: Vec<Rect> = Vec::new();
        // If any rects are hanging around, clear them
        self.rects.clear();
        // Start with the whole map as a room
        self.rects.push(Rect::new(
            1,
            1,
            build_data.map.width - 2,
            build_data.map.height - 2,
        ));
        // Build subrects for our first room
        self.add_subrects(self.rects[0], rng);

        // Clone to avoid the almighty borrow checker
        self.rects.clone().iter().for_each(|r| {
            // Get a handy handle on the room's memory location
            let room = *r;
            // Add it to the list of rooms and carve it out of the map
            rooms.push(room);
            for y in room.y1..room.y2 {
                for x in room.x1..room.x2 {
                    let idx = build_data.map.xy_idx(x, y);
                    if idx > 0
                        && idx < ((build_data.map.width * build_data.map.height) - 1) as usize
                    {
                        build_data.map.tiles[idx] = TileType::Floor;
                    }
                }
            }
            // Take a snapshot for nifty generation graphics
            build_data.take_snapshot();
        });

        // Make some corridors
        for i in 0..rooms.len() - 1 {
            let room = rooms[i];
            let next = rooms[i + 1];
            let start_x = room.x1 + (rng.roll_dice(1, i32::abs(room.x1 - room.x2)) - 1);
            let start_y = room.y1 + (rng.roll_dice(1, i32::abs(room.y1 - room.y2)) - 1);
            let end_x = next.x1 + (rng.roll_dice(1, i32::abs(next.x1 - next.x2)) - 1);
            let end_y = next.y1 + (rng.roll_dice(1, i32::abs(next.y1 - next.y2)) - 1);
            draw_corridor(&mut build_data.map, start_x, start_y, end_x, end_y);
            build_data.take_snapshot();
        }
        build_data.rooms = Some(rooms);
    }

    /// Randomly splits a rectangular room either horizontally or vertically.
    fn add_subrects(&mut self, rect: Rect, rng: &mut RandomNumberGenerator) {
        // Take out the last rectangle so we can split it up.
        // On the first call, this takes out our entire-map rectangle.
        if !self.rects.is_empty() {
            self.rects.remove(self.rects.len() - 1);
        }

        // Useful handles on partition boundaries
        let half_width = rect.width() / 2;
        let half_height = rect.height() / 2;
        // Randomly choose a horizontal or vertical split
        let split = rng.roll_dice(1, 4);

        if split <= 2 {
            // Split horizontally
            // Build and add h1 (the left partition) to the rect list
            let h1 = Rect::new(rect.x1, rect.y1, half_width - 1, rect.height());
            self.rects.push(h1);
            // If room left to split h1, recursively split it again
            if half_width > MIN_ROOM_SIZE {
                self.add_subrects(h1, rng);
            }
            // Build and add h2 (the right partition) to the rect list
            let h2 = Rect::new(rect.x1 + half_width, rect.y1, half_width, rect.height());
            self.rects.push(h2);
            // If room left to split h2, recursively split it again
            if half_width > MIN_ROOM_SIZE {
                self.add_subrects(h2, rng);
            }
        } else {
            // Split vertically
            // Build and add v1 (the top partition) to the rect list
            let v1 = Rect::new(rect.x1, rect.y1, rect.width(), half_height - 1);
            self.rects.push(v1);
            // If room left to split v1, recursively split it again
            if half_height > MIN_ROOM_SIZE {
                self.add_subrects(v1, rng);
            }
            // Build and add v2 (the bottom partition) to the rect list
            let v2 = Rect::new(rect.x1, rect.y1 + half_height, rect.width(), half_height);
            self.rects.push(v2);
            // If room left to split v2, recursively split it again
            if half_height > MIN_ROOM_SIZE {
                self.add_subrects(v2, rng);
            }
        }
    }
}
