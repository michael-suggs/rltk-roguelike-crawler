use std::{collections::HashMap, iter::repeat};

use rltk::RandomNumberGenerator;
use specs::prelude::*;

use crate::{
    spawner, BuildData, InitialMapBuilder, Map, MetaMapBuilder, Position, TileType,
    SHOW_MAPGEN_VISUALIZER,
};

pub struct CellularAutomataBuilder {}

impl InitialMapBuilder for CellularAutomataBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuildData) {
        self.build(rng, build_data);
    }
}

impl MetaMapBuilder for CellularAutomataBuilder {
    fn build_map(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuildData) {
        self.apply_iteration(build_data);
    }
}

impl CellularAutomataBuilder {
    pub fn new() -> Box<CellularAutomataBuilder> {
        Box::new(CellularAutomataBuilder {})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuildData) {
        // Randomize the map.
        for y in 1..build_data.map.height - 1 {
            for x in 1..build_data.map.width - 1 {
                let roll = rng.roll_dice(1, 100);
                let idx = build_data.map.xy_idx(x, y);
                // Give a slight preference to wall tiles.
                if roll > 55 {
                    build_data.map.tiles[idx] = TileType::Floor;
                } else {
                    build_data.map.tiles[idx] = TileType::Wall;
                }
            }
        }

        // Apply cellular automata rules 15 times.
        for _ in 0..15 {
            self.apply_iteration(build_data);
        }
    }

    fn apply_iteration(&mut self, build_data: &mut BuildData) {
        // Copy map tiles, so we aren't overwriting the tiles we're counting.
        let mut newtiles = build_data.map.tiles.clone();
        // Iterate all map cells.
        for y in 1..build_data.map.height - 1 {
            for x in 1..build_data.map.width - 1 {
                let idx = build_data.map.xy_idx(x, y);
                let mut neighbors = 0;
                // Indices of all neighboring tiles for `idx`.
                let neighbor_indices: Vec<usize> = vec![
                    1,
                    build_data.map.width as usize,
                    build_data.map.width as usize - 1,
                    build_data.map.width as usize + 1,
                ];
                // Zip `idx` with neighbor vec and check how many neighbors are walls.
                neighbor_indices.iter().zip(repeat(idx)).for_each(|(n, i)| {
                    if build_data.map.tiles[i - n] == TileType::Wall {
                        neighbors += 1;
                    }
                    if build_data.map.tiles[i + n] == TileType::Wall {
                        neighbors += 1;
                    }
                });
                // 0 or more than 4 neighbors--make it a wall; otherwise, it's a floor tile.
                if neighbors > 4 || neighbors == 0 {
                    newtiles[idx] = TileType::Wall;
                } else {
                    newtiles[idx] = TileType::Floor;
                }
            }
        }
        // End of iteration; update the map tiles, take a snapshot, and continue.
        build_data.map.tiles = newtiles.clone();
        build_data.take_snapshot();
    }
}
