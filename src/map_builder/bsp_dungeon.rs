use super::common::apply_room_to_map;
use crate::{BuildData, InitialMapBuilder, Map, MapBuilder, Position, Rect, SHOW_MAPGEN_VISUALIZER, TileType, spawner};

pub struct BspDungeonBuilder {
    rects: Vec<Rect>,
}

impl InitialMapBuilder for BspDungeonBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut crate::BuildData) {
        self.build(rng, build_data);
    }
}

impl BspDungeonBuilder {
    pub fn new(new_depth: i32) -> Box<BspDungeonBuilder> {
        Box::new(BspDungeonBuilder {
            rects: Vec::new(),
        })
    }

    /// Creates a new BSP dungeon.
    fn build(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuildData) {
        let mut rooms: Vec<Rect> = Vec::new();
        // Clear any previously stored rectangles.
        self.rects.clear();
        // Add the entire map as the "first room" (with some added padding).
        self.rects
            .push(Rect::new(2, 2, build_data.map.width - 5, build_data.map.height - 5));
        // Divide the first room into four quadrants.
        self.add_subrects(self.rects[0]);

        // Partition to create new rooms no more than 240 times.
        let mut n_rooms = 0;
        while n_rooms < 240 {
            // Get a random rectangle from our rect vec.
            let rect = self.get_random_rect(rng);
            // Get a random rectangular room from inside the rect we just grabbed.
            let candidate = self.get_random_sub_rect(rect, rng);

            // If it's in the map boundaries and isn't overlapping any other rooms, add it.
            if self.is_possible(candidate, &build_data.map) {
                // Mark it on the map
                apply_room_to_map(&mut build_data.map, &candidate);
                // Add it to the rooms list
                rooms.push(candidate);
                // Divide the rectangle we just used (but not the candidate!)
                self.add_subrects(rect);
                build_data.take_snapshot();
            }

            n_rooms += 1;
        }

        rooms.sort_by(|a, b| a.x1.cmp(&b.x1));
        for i in 0..rooms.len() - 1 {
            let room = rooms[i];
            let next = rooms[i + 1];
            let start_x = room.x1 + (rng.roll_dice(1, i32::abs(room.x1 - room.x2)) - 1);
            let start_y = room.y1 + (rng.roll_dice(1, i32::abs(room.y1 - room.y2)) - 1);
            let end_x = next.x1 + (rng.roll_dice(1, i32::abs(next.x1 - next.x2)) - 1);
            let end_y = next.y1 + (rng.roll_dice(1, i32::abs(next.y1 - next.y2)) - 1);
            BspDungeonBuilder::draw_corridor(&mut build_data.map, start_x, start_y, end_x, end_y);
            build_data.take_snapshot();
        }
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
    fn get_random_rect(&mut self, rng: &mut rltk::RandomNumberGenerator) -> Rect {
        if self.rects.len() == 1 {
            return self.rects[0];
        }
        let idx: usize = (rng.roll_dice(1, self.rects.len() as i32) - 1) as usize;
        self.rects[idx]
    }

    /// Produces a random sub-rectangle inside another between a 3x3 and a 10x10.
    fn get_random_sub_rect(&self, rect: Rect, rng: &mut rltk::RandomNumberGenerator) -> Rect {
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
    fn is_possible(&self, rect: Rect, map: &Map) -> bool {
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
                if x > map.width - 2 || x < 1 || y > map.height - 2 || y < 1 {
                    return false;
                }

                // If the tile isn't a wall tile, we've overlapped into another room.
                let idx = map.xy_idx(x, y);
                if map.tiles[idx] != TileType::Wall {
                    return false;
                }
            }
        }
        // If none of the above, we can build it.
        return true;
    }

    fn draw_corridor(map: &mut Map, x1: i32, y1: i32, x2: i32, y2: i32) {
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
            let idx = map.xy_idx(x, y);
            map.tiles[idx] = TileType::Floor;
        }
    }
}
