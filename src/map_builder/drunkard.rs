use std::collections::HashMap;

use rltk::RandomNumberGenerator;
use specs::prelude::*;

use crate::{spawner, SHOW_MAPGEN_VISUALIZER};
use super::{MapBuilder, Map, TileType, Position};

pub struct DrunkardsWalkBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
}

impl MapBuilder for DrunkardsWalkBuilder {
    fn build_map(&mut self) {
        self.build();
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
        for area in self.noise_areas.iter() {
            spawner::spawn_region(ecs, area.1, self.depth);
        }
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
            snapshot
                .revealed_tiles
                .iter_mut()
                .for_each(|v| *v = true);
            self.history.push(snapshot);
        }
    }
}

impl DrunkardsWalkBuilder {
    pub fn new(new_depth: i32) -> DrunkardsWalkBuilder {
        DrunkardsWalkBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
        }
    }

    #[allow(clippy::map_entry)]
    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        self.starting_position = Position::from(self.map.center());
        let start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);

        let map_starts: Vec<usize> = vec![start_idx];
        let dijkstra = rltk::DijkstraMap::new(
            self.map.width,
            self.map.height,
            &map_starts,
            &self.map,
            200.0,
        );

        let mut exit_tile = (0, 0.0f32);
        self.map.tiles.iter_mut().enumerate().for_each(|(i, tile)| {
            if *tile == TileType::Floor {
                let dist_to_start = dijkstra.map[i];
                if dist_to_start == std::f32::MAX {
                    *tile = TileType::Wall;
                } else {
                    if dist_to_start > exit_tile.1 {
                        exit_tile.0 = i;
                        exit_tile.1 = dist_to_start;
                    }
                }
            }
        });

        self.take_snapshot();
        self.map.tiles[exit_tile.0] = TileType::DownStairs;
        self.take_snapshot();

        let mut noise = rltk::FastNoise::seeded(rng.roll_dice(1, 65536) as u64);
        noise.set_noise_type(rltk::NoiseType::Cellular);
        noise.set_frequency(0.08);
        noise.set_cellular_distance_function(rltk::CellularDistanceFunction::Manhattan);

        self.map.iter_xy().iter_mut()
            .for_each(|(x,y)| {
                let idx = self.map.xy_idx(*x, *y);
                if self.map.tiles[idx] == TileType::Floor {
                    let cell_value = (noise.get_noise(*x as f32, *y as f32) * 10240.0) as i32;
                    if self.noise_areas.contains_key(&cell_value) {
                        self.noise_areas.get_mut(&cell_value).unwrap().push(idx);
                    } else {
                        self.noise_areas.insert(cell_value, vec![idx]);
                    }
                }
            })
    }
}
