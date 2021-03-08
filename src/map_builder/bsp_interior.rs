use rltk::RandomNumberGenerator;
use specs::prelude::*;
use crate::{SHOW_MAPGEN_VISUALIZER, spawner};
use super::{Map, MapBuilder, Position, Rect, TileType};

const MIN_ROOM_SIZE: i32 = 8;

pub struct BspInteriorBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    rooms: Vec<Rect>,
    history: Vec<Map>,
    rects: Vec<Rect>,
}

impl MapBuilder for BspInteriorBuilder {
    fn build_map(&mut self) {
        self.build();
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
        self.rooms
            .iter()
            .skip(1)
            .for_each(|room| spawner::spawn_room(ecs, room, self.depth));
    }

    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            snapshot.revealed_tiles
                    .iter_mut()
                    .for_each(|v| *v = true);
            self.history.push(snapshot);
        }
    }
}

impl BspInteriorBuilder {
    pub fn new(new_depth: i32) -> BspInteriorBuilder {
        BspInteriorBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            rooms: Vec::<Rect>::new(),
            history: Vec::<Map>::new(),
            rects: Vec::<Rect>::new(),
        }
    }

    /// Creates a new BspInterior map.
    fn build(&mut self) {
        // New rng for partitioning
        let mut rng = RandomNumberGenerator::new();
        // If any rects are hanging around, clear them
        self.rects.clear();
        // Start with the whole map as a room
        self.rects.push(Rect::new(1, 1, self.map.width - 2, self.map.height - 2));
        // Build subrects for our first room
        self.add_subrects(self.rects[0], &mut rng);

        // Clone to avoid the almighty borrow checker
        self.rects.clone().iter().for_each(|r| {
            // Get a handy handle on the room's memory location
            let room = *r;
            // Add it to the list of rooms and carve it out of the map
            self.rooms.push(room);
            for y in room.y1 .. room.y2 {
                for x in room.x1 .. room.x2 {
                    let idx = self.map.xy_idx(x, y);
                    if idx > 0 && idx < ((self.map.width * self.map.height) - 1) as usize {
                        self.map.tiles[idx] = TileType::Floor;
                    }
                }
            }
            // Take a snapshot for nifty generation graphics
            self.take_snapshot();
        });
        // Start the player in the middle of the first room
        self.starting_position = Position::from(self.rooms[0].center());

        // Make some corridors
        for i in 0..self.rooms.len() - 1 {
            let room = self.rooms[i];
            let next = self.rooms[i + 1];
            let start_x = room.x1 + (rng.roll_dice(1, i32::abs(room.x1 - room.x2)) - 1);
            let start_y = room.y1 + (rng.roll_dice(1, i32::abs(room.y1 - room.y2)) - 1);
            let end_x = next.x1 + (rng.roll_dice(1, i32::abs(next.x1 - next.x2)) - 1);
            let end_y = next.y1 + (rng.roll_dice(1, i32::abs(next.y1 - next.y2)) - 1);
            self.draw_corridor(start_x, start_y, end_x, end_y);
            self.take_snapshot();
        }

        let stairs = self.rooms[self.rooms.len() - 1].center();
        let stairs_idx = self.map.xy_idx(stairs.0, stairs.1);
        self.map.tiles[stairs_idx] = TileType::DownStairs;
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
            let h1 = Rect::new( rect.x1, rect.y1, half_width - 1, rect.height());
            self.rects.push(h1);
            // If room left to split h1, recursively split it again
            if half_width > MIN_ROOM_SIZE { self.add_subrects(h1, rng); }
            // Build and add h2 (the right partition) to the rect list
            let h2 = Rect::new(rect.x1 + half_width, rect.y1, half_width, rect.height());
            self.rects.push(h2);
            // If room left to split h2, recursively split it again
            if half_width > MIN_ROOM_SIZE { self.add_subrects(h2, rng); }
        } else {
            // Split vertically
            // Build and add v1 (the top partition) to the rect list
            let v1 = Rect::new(rect.x1, rect.y1, rect.width(), half_height - 1);
            self.rects.push(v1);
            // If room left to split v1, recursively split it again
            if half_height > MIN_ROOM_SIZE { self.add_subrects(v1, rng); }
            // Build and add v2 (the bottom partition) to the rect list
            let v2 = Rect::new(rect.x1, rect.y1 + half_height, rect.width(), half_height);
            self.rects.push(v2);
            // If room left to split v2, recursively split it again
            if half_height > MIN_ROOM_SIZE { self.add_subrects(v2, rng); }
        }
    }

    /// Draw a corridor between two rooms.
    fn draw_corridor(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        let mut x = x1;
        let mut y = y1;

        while x != x2 || y != y2 {
            if x < x2 {
                x += 1;
            } else if x > x2 {
                x -= 1;
            } else if y < y2 {
                y += 1;
            } else if y > y2 {
                y -= 1;
            }
            let idx = self.map.xy_idx(x, y);
            self.map.tiles[idx] = TileType::Floor;
        }
    }
}
