use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
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
    pub fn new_random(new_depth: i32) -> DLABuilder {
        let rng = rltk::RandomNumberGenerator::new();
        DLABuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            algorithm: rand::random(),
            symmetry: rand::random(),
            brush_size: rng.roll_dice(1, 3),
            floor_percent: 0.25,
        }
    }

    pub fn new_walk_inwards(new_depth: i32) -> DLABuilder {
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

    pub fn new_walk_outwards(new_depth: i32) -> DLABuilder {
        DLABuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            algorithm: DLAAlgorithm::WalkOutwards,
            symmetry: DLASymmetry::None,
            brush_size: 2,
            floor_percent: 0.25,
        }
    }

    pub fn new_central_attractor(new_depth: i32) -> DLABuilder {
        DLABuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            algorithm: DLAAlgorithm::WalkInwards,
            symmetry: DLASymmetry::None,
            brush_size: 2,
            floor_percent: 0.25,
        }
    }

    pub fn new_insectoid(new_depth: i32) -> DLABuilder {
        DLABuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            algorithm: DLAAlgorithm::WalkInwards,
            symmetry: DLASymmetry::Horizontal,
            brush_size: 2,
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
        let center = Position::from(self.map.center());
        let idx = self.map.xy_idx(x, y);
        self.map.tiles[idx] = TileType::Floor;

        // Match on symmetry type
        match self.symmetry {
            // No symmetry--just paint
            DLASymmetry::None => self.apply_paint(x, y),
            DLASymmetry::Horizontal => {
                if x == center.x {
                    // If on the tile, paint it
                    self.apply_paint(x, y);
                } else {
                    // Else, apply paint symmetrically in the x-direction
                    // based on distance from it
                    let d_x = i32::abs(center.x - x);
                    self.apply_paint(center.x + d_x, y);
                    self.apply_paint(center.x - d_x, y);
                }
            }
            DLASymmetry::Vertical => {
                if y == center.y {
                    // If on the tile, paint it
                    self.apply_paint(x, y);
                } else {
                    // Else, apply paint symmetrically in the y-direction
                    // based on distance from it
                    let d_y = i32::abs(center.y - y);
                    self.apply_paint(x, center.y + d_y);
                    self.apply_paint(x, center.y + d_y);
                }
            }
            DLASymmetry::Both => {
                // Break center down into parts to appease the borrow checker
                let (center_x, center_y) = center.into();
                if (x, y) == (center_x, center_y) {
                    // If on the tile, paint it
                    self.apply_paint(x, y);
                } else {
                    // Apply symmetric paint horizontally about the tile
                    let d_x = i32::abs(center_x - x);
                    self.apply_paint(center_x + d_x, y);
                    self.apply_paint(center_x - d_x, y);
                    // Apply symmetric paint vertically about the tile
                    let d_y = i32::abs(center_y - y);
                    self.apply_paint(x, center_y + d_y);
                    self.apply_paint(x, center_y - d_y);
                }
            }
        }
    }

    /// Applies paint to a tile based on brush size.
    fn apply_paint(&mut self, x: i32, y: i32) {
        if self.brush_size == 1 {
            // Single-tile brush--paint just that floor tile
            let idx = self.map.xy_idx(x, y);
            self.map.tiles[idx] = TileType::Floor;
        } else {
            // Else, loop through brush size
            let half_brush = self.brush_size / 2;
            for brush_y in y - half_brush .. y + half_brush {
                for brush_x in x - half_brush .. x + half_brush {
                    // Make sure the `half_brush` index is in bounds
                    if self.map.in_bounds(brush_x, 0, brush_y, 0) {
                        // Paint at each `half_brush` index
                        let idx = self.map.xy_idx(brush_x, brush_y);
                        self.map.tiles[idx] = TileType::Floor;
                    }
                }
            }
        }
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

impl Distribution<DLAAlgorithm> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> DLAAlgorithm {
        match rng.gen_range(0..=2) {
            0 => DLAAlgorithm::WalkInwards,
            1 => DLAAlgorithm::WalkOutwards,
            _ => DLAAlgorithm::CentralAttractor,
        }
    }
}

impl Distribution<DLASymmetry> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> DLASymmetry {
        match rng.gen_range(0..=3) {
            0 => DLASymmetry::None,
            1 => DLASymmetry::Horizontal,
            2 => DLASymmetry::Vertical,
            _ => DLASymmetry::Both,
        }
    }
}
