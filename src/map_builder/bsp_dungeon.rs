use rltk::RandomNumberGenerator;

use super::common::apply_room_to_map;
use crate::{spawner, Map, MapBuilder, Position, Rect, TileType, SHOW_MAPGEN_VISUALIZER};

pub struct BspDungeonBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    rooms: Vec<Rect>,
    history: Vec<Map>,
    rects: Vec<Rect>,
}

impl MapBuilder for BspDungeonBuilder {
    fn build_map(&mut self) {
        self.build();
    }

    fn spawn_entities(&mut self, ecs: &mut specs::World) {
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
            snapshot.revealed_tiles.iter_mut().for_each(|v| *v = true);
            self.history.push(snapshot);
        }
    }
}

impl BspDungeonBuilder {
    pub fn new(new_depth: i32) -> BspDungeonBuilder {
        BspDungeonBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            rooms: Vec::<Rect>::new(),
            history: Vec::<Map>::new(),
            rects: Vec::<Rect>::new(),
        }
    }

    /// Creates a new BSP dungeon.
    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();
        // Clear any previously stored rectangles.
        self.rects.clear();
        // Add the entire map as the "first room" (with some added padding).
        self.rects
            .push(Rect::new(2, 2, self.map.width - 5, self.map.height - 5));
        // Divide the first room into four quadrants.
        self.add_subrects(self.rects[0]);

        // Partition to create new rooms no more than 240 times.
        let mut n_rooms = 0;
        while n_rooms < 240 {
            // Get a random rectangle from our rect vec.
            let rect = self.get_random_rect(&mut rng);
            // Get a random rectangular room from inside the rect we just grabbed.
            let candidate = self.get_random_sub_rect(rect, &mut rng);

            // If it's in the map boundaries and isn't overlapping any other rooms, add it.
            if self.is_possible(candidate) {
                // Mark it on the map
                apply_room_to_map(&mut self.map, &candidate);
                // Add it to the rooms list
                self.rooms.push(candidate);
                // Divide the rectangle we just used (but not the candidate!)
                self.add_subrects(rect);
                self.take_snapshot();
            }

            n_rooms += 1;
        }
        self.starting_position = Position::from(self.rooms[0].center());

        self.rooms.sort_by(|a, b| a.x1.cmp(&b.x1));
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

    /// Splits a rectangle into four quadrants.
    fn add_subrects(&mut self, rect: Rect) {
        // Each quadrant lies between the boarder and half_width and half_height.
        let half_width = i32::max(rect.width() / 2, 1);
        let half_height = i32::max(rect.width() / 2, 1);

        // Add all four quadrants to our rect vec.
        self.rects
            .push(Rect::new(rect.x1, rect.y1, half_width, half_height));
        self.rects.push(Rect::new(
            rect.x1,
            rect.y1 + half_height,
            half_width,
            half_height,
        ));
        self.rects.push(Rect::new(
            rect.x1 + half_width,
            rect.y1,
            half_width,
            half_height,
        ));
        self.rects.push(Rect::new(
            rect.x1 + half_width,
            rect.y1 + half_height,
            half_width,
            half_height,
        ));
    }

    /// Get a random rectangle from the generated rectangles so far.
    fn get_random_rect(&mut self, rng: &mut RandomNumberGenerator) -> Rect {
        if self.rects.len() == 1 {
            return self.rects[0];
        }
        let idx: usize = (rng.roll_dice(1, self.rects.len() as i32) - 1) as usize;
        self.rects[idx]
    }

    /// Produces a random sub-rectangle inside another between a 3x3 and a 10x10.
    fn get_random_sub_rect(&self, rect: Rect, rng: &mut RandomNumberGenerator) -> Rect {
        let mut result = rect;
        let width = i32::max(3, rng.roll_dice(1, i32::min(rect.width(), 10)) - 1) + 1;
        let height = i32::max(3, rng.roll_dice(1, i32::min(rect.height(), 10)) - 1) + 1;

        result.x1 += rng.roll_dice(1, 6) - 1;
        result.y1 += rng.roll_dice(1, 6) - 1;
        result.x2 = result.x1 + width;
        result.y2 = result.y1 + height;

        result
    }

    /// Inspect a rectangle to see if it conforms to the map, and thus can be used as a room.
    fn is_possible(&self, rect: Rect) -> bool {
        // Copy the rect and expand 2 in all directions (to prevent room overlap).
        let mut expanded = rect;
        expanded.x1 -= 2;
        expanded.x2 += 2;
        expanded.y1 -= 2;
        expanded.y2 += 2;

        // Go through all coordinates in the rect.
        for y in expanded.y1..=expanded.y2 {
            for x in expanded.x1..=expanded.x2 {
                // Can't build if it's outside the map boundaries!
                if x > self.map.width - 2 || x < 1 || y > self.map.height - 2 || y < 1 {
                    return false;
                }

                // If the tile isn't a wall tile, we've overlapped into another room.
                let idx = self.map.xy_idx(x, y);
                if self.map.tiles[idx] != TileType::Wall {
                    return false;
                }
            }
        }
        // If none of the above, we can build it.
        return true;
    }

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
