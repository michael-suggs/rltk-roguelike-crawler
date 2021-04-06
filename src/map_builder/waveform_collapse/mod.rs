use std::collections::HashMap;

use common::MapChunk;
use constraints::{build_patterns, patterns_to_constraints, render_pattern_to_map, Chunk};
use solver::Solver;

use crate::{spawner, Map, MapBuilder, Position, TileType, SHOW_MAPGEN_VISUALIZER};

use super::common::{
    generate_voronoi_spawn_regions, remove_unreachable_areas_returning_most_distant,
};

mod common;
mod constraints;
mod image_loader;
mod solver;

pub struct WaveformCollapseBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
}

impl MapBuilder for WaveformCollapseBuilder {
    fn build_map(&mut self) {
        self.build();
    }

    fn spawn_entities(&mut self, ecs: &mut specs::World) {
        self.noise_areas
            .iter()
            .for_each(|area| spawner::spawn_region(ecs, area.1, self.depth));
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
            let mut snapshot: Map = self.map.clone();
            snapshot.revealed_tiles.iter_mut().for_each(|v| *v = true);
            self.history.push(snapshot);
        }
    }
}

impl WaveformCollapseBuilder {
    pub fn new(new_depth: i32) -> WaveformCollapseBuilder {
        WaveformCollapseBuilder {
            map: image_loader::load_rex_map(
                new_depth,
                &rltk::rex::XpFile::from_resource("../resources/wfc-demo2.xp").unwrap(),
            ),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
        }
    }

    fn build(&mut self) {
        const CHUNK_SIZE: i32 = 7;
        let mut rng = rltk::RandomNumberGenerator::new();

        let patterns = build_patterns(&self.map, CHUNK_SIZE, true, true);
        let constraints = patterns_to_constraints(patterns, CHUNK_SIZE);
        self.render_tile_gallery(&constraints, CHUNK_SIZE);

        self.map = Map::new(self.depth);
        loop {
            let mut solver = Solver::new(constraints.clone(), CHUNK_SIZE, &self.map);
            while !solver.iteration(&mut self.map, &mut rng) {
                self.take_snapshot();
            }
            self.take_snapshot();
            if solver.possible {
                break;
            }
        }

        self.starting_position = Position::from(self.map.center());
        let mut start_idx = self
            .map
            .xy_idx(self.starting_position.x, self.starting_position.y);
        // while self.map.tiles[start_idx] != TileType::Floor {
        //     self.starting_position.x -= 1;
        //     start_idx = self
        //         .map
        //         .xy_idx(self.starting_position.x, self.starting_position.y);
        // }
        self.take_snapshot();

        let exit_tile = remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);
        self.map.tiles[exit_tile] = TileType::DownStairs;
        self.take_snapshot();

        self.noise_areas = generate_voronoi_spawn_regions(&self.map, &mut rng);
    }

    fn render_tile_gallery(&mut self, constraints: &Vec<MapChunk>, chunk_size: i32) {
        self.map = Map::new(0);
        let mut ctr = 0;
        let mut x = 1;
        let mut y = 1;

        while ctr < constraints.len() {
            let chunk = Chunk::presized(chunk_size, Position { x, y });
            render_pattern_to_map(&mut self.map, &constraints[ctr], chunk);
            self.take_snapshot();

            x += chunk_size + 1;
            if x + chunk_size > self.map.width {
                // Move to the next row
                x = 1;
                y += chunk_size + 1;

                if y + chunk_size > self.map.height {
                    // Move to the next page
                    self.take_snapshot();
                    self.map = Map::new(0);

                    x = 1;
                    y = 1;
                }
            }
            ctr += 1;
        }
        self.take_snapshot();
    }
}
