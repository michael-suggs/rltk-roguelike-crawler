use rltk::RandomNumberGenerator;
use specs::prelude::*;
use crate::{Position, SHOW_MAPGEN_VISUALIZER, spawner};

use super::{common::*, Map, MapBuilder, Rect, TileType};

pub struct SimpleMapBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    rooms: Vec<Rect>,
    history: Vec<Map>,
}

impl MapBuilder for SimpleMapBuilder {
    fn build_map(&mut self) {
        self.rooms_and_corridors();
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
        &self.rooms
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

impl SimpleMapBuilder {
    pub fn new(new_depth: i32) -> SimpleMapBuilder {
        SimpleMapBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            rooms: Vec::new(),
            history: Vec::new(),
        }
    }

    /// Generates a new map with rooms connected via corridors.
    ///
    /// `MAX_ROOMS`: Maximum number of rooms to generate.
    /// `MIN_SIZE`: Smallest room size to generate.
    /// `MAX_SIZE`: Largest room size to generate.
    fn rooms_and_corridors(&mut self) {
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, self.map.width - w - 1) - 1;
            let y = rng.roll_dice(1, self.map.height - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let ok: bool = self.rooms.iter().all(|r| !new_room.intersect(r));

            if ok {
                apply_room_to_map(&mut self.map, &new_room);
                self.take_snapshot();

                if !self.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = self.rooms.last().unwrap().center();
                    if rng.range(0,2) == 1 {
                        apply_horizontal_tunnel(&mut self.map, prev_x, new_x, prev_y);
                        apply_vertical_tunnel(&mut self.map, prev_y, new_y, new_x);
                    } else {
                        apply_vertical_tunnel(&mut self.map, prev_y, new_y, prev_x);
                        apply_horizontal_tunnel(&mut self.map, prev_x, new_x, new_y);
                    }
                }

                self.rooms.push(new_room);
                self.take_snapshot();
            }
        }

        let stairs_pos = self.rooms.last().unwrap().center();
        let stairs_idx = self.map.xy_idx(stairs_pos.0, stairs_pos.1);
        self.map.tiles[stairs_idx] = TileType::DownStairs;

        let start_pos = self.rooms[0].center();
        self.starting_position = Position { x: start_pos.0, y: start_pos.1 };
    }
}
