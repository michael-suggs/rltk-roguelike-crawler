use std::collections::HashMap;

use rltk::RandomNumberGenerator;

use crate::{spawner, Map, MapBuilder, Position, TileType, SHOW_MAPGEN_VISUALIZER};

use super::common::DrunkDigger;

#[derive(PartialEq, Clone, Copy)]
pub enum DLAAlgorithm {
    WalkInwards,
    WalkOutwards,
    CentralAttractor,
}

#[derive(PartialEq, Clone, Copy)]
pub enum DLASymmetry {
    None,
    Horizontal,
    Vertical,
    Both,
}

pub struct DLABuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
    algorithm: DLAAlgorithm,
    symmetry: DLASymmetry,
    brush_size: i32,
    floor_percent: f32,
}

impl MapBuilder for DLABuilder {
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
            let mut snapshot = self.map.clone();
            snapshot.revealed_tiles.iter_mut().for_each(|v| *v = true);
            self.history.push(snapshot);
        }
    }
}

impl DLABuilder {
    pub fn new(new_depth: i32) -> DLABuilder {
        DLABuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            algorithm: DLAAlgorithm::WalkInwards,
            symmetry: DLASymmetry::None,
            brush_size: 1,
            floor_percent: 0.25,
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        self.starting_position = Position::from(self.map.center());
        let start_idx = self
            .map
            .xy_idx(self.starting_position.x, self.starting_position.y);
        self.take_snapshot();
        self.seed_start(start_idx);

        // let total_tiles = self.map.tiles.len() as i32;
        let desired_floor_tiles = (self.floor_percent * self.map.tiles.len() as f32) as usize;

        match self.algorithm {
            DLAAlgorithm::WalkInwards => self.walk_inwards(desired_floor_tiles, &mut rng),
            DLAAlgorithm::WalkOutwards => self.walk_outwards(desired_floor_tiles, &mut rng),
            DLAAlgorithm::CentralAttractor => self.central_attractor(desired_floor_tiles, &mut rng),
        }
    }

    fn seed_start(&mut self, start_idx: usize) {
        self.map.tiles[start_idx] = TileType::Floor;
        self.map.tiles[start_idx - 1] = TileType::Floor;
        self.map.tiles[start_idx + 1] = TileType::Floor;
        self.map.tiles[start_idx - self.map.width as usize] = TileType::Floor;
        self.map.tiles[start_idx + self.map.width as usize] = TileType::Floor;
    }

    fn paint(&mut self, x: i32, y: i32) {
        let idx = self.map.xy_idx(x, y);
        self.map.tiles[idx] = TileType::Floor;
    }

    fn walk_inwards(&mut self, desired_floor_tiles: usize, rng: &mut RandomNumberGenerator) {
        let mut floor_tile_count = self.map.count_floor_tiles();
        while floor_tile_count < desired_floor_tiles {
            let mut drunk = DrunkDigger::new(
                rng.roll_dice(1, self.map.width - 3) + 1,
                rng.roll_dice(1, self.map.height - 3) + 1,
                rng,
            );

            let (prev_x, prev_y) = drunk.stagger_tiles(&mut self.map, TileType::Wall);
            self.paint(prev_x, prev_y);
            floor_tile_count = self.map.count_floor_tiles();
        }
    }

    fn walk_outwards(&mut self, desired_floor_tiles: usize, rng: &mut RandomNumberGenerator) {
        let mut floor_tile_count = self.map.count_floor_tiles();
        let mut drunk = DrunkDigger::new(self.starting_position.x, self.starting_position.y, rng);

        while floor_tile_count < desired_floor_tiles {
            drunk.stagger_tiles(&mut self.map, TileType::Floor);
            floor_tile_count = self.map.count_floor_tiles();
        }

        self.paint(drunk.x, drunk.y);
    }

    fn central_attractor(&mut self, desired_floor_tiles: usize, rng: &mut RandomNumberGenerator) {
        let mut floor_tile_count = self.map.count_floor_tiles();
        while floor_tile_count < desired_floor_tiles {
            let mut digger = Position {
                x: rng.roll_dice(1, self.map.width - 3) + 1,
                y: rng.roll_dice(1, self.map.height - 3) + 1,
            };
            let mut prev = digger.clone();
            let mut digger_idx = self.map.xy_idx(digger.x, digger.y);

            let mut path = rltk::line2d(
                rltk::LineAlg::Bresenham,
                rltk::Point::new(digger.x, digger.y),
                rltk::Point::new(self.starting_position.x, self.starting_position.y),
            );

            while self.map.tiles[digger_idx] == TileType::Wall && !path.is_empty() {
                prev = digger;
                digger = Position {
                    x: path[0].x,
                    y: path[0].y,
                };
                path.remove(0);
                digger_idx = self.map.xy_idx(digger.x, digger.y);
            }

            self.paint(prev.x, prev.y);
            floor_tile_count = self.map.count_floor_tiles();
        }
    }
}
