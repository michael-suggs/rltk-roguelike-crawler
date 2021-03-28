use std::{collections::HashMap, iter::repeat};

use rltk::RandomNumberGenerator;
use specs::prelude::*;

use crate::{Map, MapBuilder, Position, SHOW_MAPGEN_VISUALIZER, TileType, spawner};

use super::common::{generate_voronoi_spawn_regions, remove_unreachable_areas_returning_most_distant};

pub struct CellularAutomataBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
}

impl MapBuilder for CellularAutomataBuilder {
    fn build_map(&mut self) {
        self.build();
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
        self.noise_areas.iter().for_each(|area| spawner::spawn_region(ecs, area.1, self.depth));
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

impl CellularAutomataBuilder {
    pub fn new(new_depth: i32) -> CellularAutomataBuilder {
        CellularAutomataBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        // Randomize the map.
        for y in 1..self.map.height - 1 {
            for x in 1..self.map.width - 1 {
                let roll = rng.roll_dice(1, 100);
                let idx = self.map.xy_idx(x, y);
                // Give a slight preference to wall tiles.
                if roll > 55 {
                    self.map.tiles[idx] = TileType::Floor;
                } else {
                    self.map.tiles[idx] = TileType::Wall;
                }
            }
        }

        // Apply cellular automata rules 15 times.
        for _ in 0..15 {
            // Copy map tiles, so we aren't overwriting the tiles we're counting.
            let mut newtiles = self.map.tiles.clone();
            // Iterate all map cells.
            for y in 1..self.map.height - 1 {
                for x in 1..self.map.width - 1 {
                    let idx = self.map.xy_idx(x, y);
                    let mut neighbors = 0;
                    // Indices of all neighboring tiles for `idx`.
                    let neighbor_indices: Vec<usize> = vec![
                        1, self.map.width as usize,
                        self.map.width as usize - 1,
                        self.map.width as usize + 1
                    ];
                    // Zip `idx` with neighbor vec and check how many neighbors are walls.
                    neighbor_indices
                        .iter()
                        .zip(repeat(idx))
                        .for_each(|(n, i)| {
                            if self.map.tiles[i - n] == TileType::Wall { neighbors += 1; }
                            if self.map.tiles[i + n] == TileType::Wall { neighbors += 1; }
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
            self.map.tiles = newtiles.clone();
            self.take_snapshot();
        }

        // Locate a place to start the player.
        self.locate_start();
        // Find a place to put the exit using Dijkstra's.
        self.locate_exit();

        self.noise_areas = generate_voronoi_spawn_regions(&self.map, &mut rng);

        // // Make a new noise generator to generate Cellular (Voronoi) noise.
        // let mut noise = rltk::FastNoise::seeded(
        //     rng.roll_dice(1, 65536) as u64);
        // noise.set_noise_type(rltk::NoiseType::Cellular);
        // // 0.08 is arbitrary, but seems to work nice.
        // noise.set_frequency(0.08);
        // // Uses L1 distance to favor enlongated shapes.
        // noise.set_cellular_distance_function(rltk::CellularDistanceFunction::Manhattan);

        // // Iterate the map
        // for (x, y) in self.map.iter_xy() {
        //     let idx = self.map.xy_idx(x, y);
        //     // Exclude wall tiles, focus on floors.
        //     if self.map.tiles[idx] == TileType::Floor {
        //         // Query for a noise value for the current coordinates, and scale to make useful.
        //         let cell_value = (noise.get_noise(x as f32, y as f32) * 10240.0) as i32;
        //         if self.noise_areas.contains_key(&cell_value) {
        //             self.noise_areas.get_mut(&cell_value).unwrap().push(idx);
        //         } else {
        //             self.noise_areas.insert(cell_value, vec![idx]);
        //         }
        //     }
        // }
    }

    /// Finds a starting location relatively close to the center of the map.
    fn locate_start(&mut self) {
        self.starting_position = Position::from(self.map.center());
        let mut start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        while self.map.tiles[start_idx] != TileType::Floor {
            self.starting_position.x -= 1;
            start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        }
    }

    /// Finds an exit reasonably far away from the player's starting location.
    fn locate_exit(&mut self) {
        // Make a vector of the player's start (since `DijkstraMap` implements multi-start).
        let start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        self.take_snapshot();
        // let map_starts: Vec<usize> = vec![start_idx];
        // // Make a `DijkstraMap` matching the map dimensions that runs for a max of 200 steps.
        // let dijkstra_map = rltk::DijkstraMap::new(self.map.width, self.map.height,
        //     &map_starts, &self.map, 200.0);

        // // `exit_tile` holds the tile index and distance to the proposed exit.
        // let mut exit_tile = (0, 0.0f32);
        // // Enumerate map tiles looking for an exit.
        // self.map.tiles
        //     .iter_mut()
        //     .enumerate()
        //     .for_each(|(i, tile)| {
        //         // Found a floor tile--get the distance between it and the start.
        //         if *tile == TileType::Floor {
        //             let dist_to_start = dijkstra_map.map[i];
        //             if dist_to_start == std::f32::MAX {
        //                 // Unreachable, so might as well make it a wall tile.
        //                 *tile = TileType::Wall;
        //             } else if dist_to_start > exit_tile.1 {
        //                 // If the new candidate tile is reachable and further that our previous
        //                 // best exit candidate, set it's index and distance to the new best.
        //                 exit_tile = (i, dist_to_start);
        //             }
        //         }
        //     });
        // // Visited all tiles--set the exit location and take a snapshot.
        // self.map.tiles[exit_tile.0] = TileType::DownStairs;

        let exit_tile = remove_unreachable_areas_returning_most_distant(
            &mut self.map, start_idx
        );
        self.take_snapshot();

        self.map.tiles[exit_tile] = TileType::DownStairs;
        self.take_snapshot();
    }
}
