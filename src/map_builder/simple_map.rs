use rltk::RandomNumberGenerator;
use specs::prelude::*;
use super::{common::*, Map, MapBuilder, Rect, TileType};

pub struct SimpleMapBuilder {}

impl MapBuilder for SimpleMapBuilder {
    fn build(new_depth: i32) -> Map {
        let mut map = Map::new(new_depth);
        SimpleMapBuilder::rooms_and_corridors(&mut map);
        map
    }
}

impl SimpleMapBuilder {
    /// Generates a new map with rooms connected via corridors.
    ///
    /// `MAX_ROOMS`: Maximum number of rooms to generate.
    /// `MIN_SIZE`: Smallest room size to generate.
    /// `MAX_SIZE`: Largest room size to generate.
    fn rooms_and_corridors(map: &mut Map) {
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, map.width - w - 1) - 1;
            let y = rng.roll_dice(1, map.height - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let ok: bool = map.rooms.iter().all(|r| !new_room.intersect(r));

            if ok {
                apply_room_to_map(map, &new_room);

                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms.last().unwrap().center();
                    if rng.range(0,2) == 1 {
                        apply_horizontal_tunnel(map, prev_x, new_x, prev_y);
                        apply_vertical_tunnel(map, prev_y, new_y, new_x);
                    } else {
                        apply_vertical_tunnel(map, prev_y, new_y, prev_x);
                        apply_horizontal_tunnel(map, prev_x, new_x, new_y);
                    }
                }

                map.rooms.push(new_room);
            }
        }

        let stairs_pos = map.rooms.last().unwrap().center();
        let stairs_idx = map.xy_idx(stairs_pos.0, stairs_pos.1);
        map.tiles[stairs_idx] = TileType::DownStairs;
    }
}
