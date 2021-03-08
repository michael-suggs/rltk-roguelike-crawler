use super::{Map, Rect, TileType};
use std::{cmp::{max, min}, iter};

pub fn apply_room_to_map(map: &mut Map, room: &Rect) {
    (room.y1 + 1 ..= room.y2)
        .map(|y| iter::repeat(y).zip(room.x1 + 1 ..= room.x2))
        .flatten()
        .for_each(|(y,x)| {
            let idx = map.xy_idx(x, y);
            map.tiles[idx] = TileType::Floor
        });
}

pub fn apply_horizontal_tunnel(map: &mut Map, x1: i32, x2: i32, y: i32) {
    (min(x1, x2) ..= max(x1, x2))
        .for_each(|x| {
            let idx = map.xy_idx(x, y);
            if idx > 0 && idx < map.width as usize * map.height as usize {
                map.tiles[idx as usize] = TileType::Floor;
            }
        });
}

pub fn apply_vertical_tunnel(map: &mut Map, y1: i32, y2: i32, x: i32) {
    (min(y1, y2) ..= max(y1, y2))
        .for_each(|y| {
            let idx = map.xy_idx(x, y);
            if idx > 0 && idx < map.width as usize * map.height as usize {
                map.tiles[idx as usize] = TileType::Floor;
            }
        })
}
