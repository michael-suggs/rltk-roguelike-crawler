use std::collections::HashMap;

use common::MapChunk;
use constraints::{build_patterns, patterns_to_constraints, render_pattern_to_map, Chunk};
use image_loader::load_rex_map;
use rltk::RandomNumberGenerator;
use solver::Solver;

use crate::{spawner, BuildData, Map, MetaMapBuilder, Position, TileType, SHOW_MAPGEN_VISUALIZER};

mod common;
mod constraints;
mod image_loader;
mod solver;

#[derive(PartialEq, Clone, Copy)]
pub enum WaveformMode {
    TestMap,
    Derived,
}

pub struct WaveformCollapseBuilder {}

impl MetaMapBuilder for WaveformCollapseBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuildData) {
        self.build(rng, build_data);
    }
}

impl WaveformCollapseBuilder {
    pub fn new() -> Box<WaveformCollapseBuilder> {
        Box::new(WaveformCollapseBuilder {})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuildData) {
        const CHUNK_SIZE: i32 = 7;

        let patterns = build_patterns(&build_data.map, CHUNK_SIZE, true, true);
        let constraints = patterns_to_constraints(patterns, CHUNK_SIZE);
        self.render_tile_gallery(&constraints, CHUNK_SIZE, build_data);

        build_data.map = Map::new(build_data.map.depth);
        loop {
            let mut solver = Solver::new(constraints.clone(), CHUNK_SIZE, &build_data.map);
            while !solver.iteration(&mut build_data.map, rng) {
                build_data.take_snapshot();
            }
            build_data.take_snapshot();
            if solver.possible {
                break;
            }
            build_data.spawn_list.clear();
        }
    }

    fn render_tile_gallery(
        &mut self,
        constraints: &Vec<MapChunk>,
        chunk_size: i32,
        build_data: &mut BuildData,
    ) {
        build_data.map = Map::new(0);
        let mut ctr = 0;
        let mut x = 1;
        let mut y = 1;

        while ctr < constraints.len() {
            let chunk = Chunk::presized(chunk_size, Position { x, y });
            render_pattern_to_map(&mut build_data.map, &constraints[ctr], chunk);
            build_data.take_snapshot();

            x += chunk_size + 1;
            if x + chunk_size > build_data.map.width {
                // Move to the next row
                x = 1;
                y += chunk_size + 1;

                if y + chunk_size > build_data.map.height {
                    // Move to the next page
                    build_data.take_snapshot();
                    build_data.map = Map::new(0);

                    x = 1;
                    y = 1;
                }
            }
            ctr += 1;
        }
        build_data.take_snapshot();
    }
}
