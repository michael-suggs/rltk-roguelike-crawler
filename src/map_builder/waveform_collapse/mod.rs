use std::collections::HashMap;

use constraints::build_patterns;

use crate::{spawner, Map, MapBuilder, Position, TileType, SHOW_MAPGEN_VISUALIZER};

use self::constraints::{Chunk, render_pattern_to_map};

use super::common::{
    generate_voronoi_spawn_regions, remove_unreachable_areas_returning_most_distant,
};

mod constraints;
mod image_loader;

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
                &rltk::rex::XpFile::from_resource("../resources/wfc-demo1.xp").unwrap(),
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
        self.render_tile_gallery(&patterns, CHUNK_SIZE);

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

    fn render_tile_gallery(&mut self, patterns: &Vec<Vec<TileType>>, chunk_size: i32) {
        self.map = Map::new(0);
        let mut ctr = 0;
        let mut x = 0;
        let mut y = 0;
        let chunks_x = self.map.width / chunk_size;
        let chunks_y = self.map.height / chunk_size;

        while ctr < patterns.len() {
            let chunk = Chunk::new(chunk_size, x, y);
            println!("{} : New chunk at ({}, {})/({}, {}) => {:?} -> {:?}", ctr, x, y, chunks_x, chunks_y, chunk.start, chunk.end);
            render_pattern_to_map(&mut self.map, &patterns[ctr], chunk);
            self.take_snapshot();

            x += 1;
            if x >= chunks_x {
                // Move to the next row
                x = 0;
                y += 1;

                if y >= chunks_y {
                    // Move to the next page
                    self.take_snapshot();
                    self.map = Map::new(0);

                    x = 0;
                    y = 0;
                }
            }
            ctr += 1;
        }
        self.take_snapshot();

    }
}
