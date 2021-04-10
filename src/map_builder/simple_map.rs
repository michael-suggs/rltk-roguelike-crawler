use super::{common::*, Map, MapBuilder, Rect, TileType};
use crate::{BuildData, InitialMapBuilder, Position, SHOW_MAPGEN_VISUALIZER, spawner};
use rltk::RandomNumberGenerator;
use specs::prelude::*;

// pub struct SimpleMapBuilder {
//     map: Map,
//     starting_position: Position,
//     depth: i32,
//     rooms: Vec<Rect>,
//     history: Vec<Map>,
//     spawn_list: Vec<(usize, String)>,
// }

// impl MapBuilder for SimpleMapBuilder {
//     fn build_map(&mut self) {
//         self.rooms_and_corridors();
//     }

//     fn get_spawn_list(&self) -> &Vec<(usize, String)> {
//         &self.spawn_list
//     }

//     fn get_map(&self) -> Map {
//         self.map.clone()
//     }

//     fn get_starting_position(&self) -> Position {
//         self.starting_position.clone()
//     }

//     fn get_snapshot_history(&self) -> Vec<Map> {
//         self.history.clone()
//     }

//     fn take_snapshot(&mut self) {
//         if SHOW_MAPGEN_VISUALIZER {
//             let mut snapshot = self.map.clone();
//             snapshot.revealed_tiles.iter_mut().for_each(|v| *v = true);
//             self.history.push(snapshot);
//         }
//     }
// }

pub struct SimpleMapBuilder {}

impl InitialMapBuilder for SimpleMapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuildData) {
        self.rooms_and_corridors(rng, build_data);
    }
}

impl SimpleMapBuilder {
    pub fn new() -> Box<SimpleMapBuilder> {
        Box::new(SimpleMapBuilder {})
    }

    /// Generates a new map with rooms connected via corridors.
    ///
    /// `MAX_ROOMS`: Maximum number of rooms to generate.
    /// `MIN_SIZE`: Smallest room size to generate.
    /// `MAX_SIZE`: Largest room size to generate.
    fn rooms_and_corridors(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuildData) {
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;
        let mut rooms: Vec<Rect> = Vec::new();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, build_data.map.width - w - 1) - 1;
            let y = rng.roll_dice(1, build_data.map.height - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let ok: bool = rooms.iter().all(|r| !new_room.intersect(r));

            if ok {
                apply_room_to_map(&mut build_data.map, &new_room);
                build_data.take_snapshot();

                if !rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = rooms.last().unwrap().center();
                    if rng.range(0, 2) == 1 {
                        apply_horizontal_tunnel(&mut build_data.map, prev_x, new_x, prev_y);
                        apply_vertical_tunnel(&mut build_data.map, prev_y, new_y, new_x);
                    } else {
                        apply_vertical_tunnel(&mut build_data.map, prev_y, new_y, prev_x);
                        apply_horizontal_tunnel(&mut build_data.map, prev_x, new_x, new_y);
                    }
                }

                rooms.push(new_room);
                build_data.take_snapshot();
            }
        }
        build_data.rooms = Some(rooms);

        // let stairs_pos = rooms.last().unwrap().center();
        // let stairs_idx = build_data.map.xy_idx(stairs_pos.0, stairs_pos.1);
        // build_data.map.tiles[stairs_idx] = TileType::DownStairs;

        // let start_pos = rooms[0].center();
        // build_data.starting_position = Position {
        //     x: start_pos.0,
        //     y: start_pos.1,
        // };

        // for room in self.rooms.iter().skip(1) {
        //     spawner::spawn_room(&self.map, &mut rng, room, self.depth, &mut self.spawn_list);
        // }
    }
}
