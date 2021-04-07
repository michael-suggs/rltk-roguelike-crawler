use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::collections::HashMap;

use rltk::RandomNumberGenerator;

use crate::{spawner, Map, MapBuilder, Position, TileType, SHOW_MAPGEN_VISUALIZER};

use super::common::{paint, Digger, Symmetry};

#[derive(PartialEq, Clone, Copy)]
pub enum DLAAlgorithm {
    WalkInwards,
    WalkOutwards,
    CentralAttractor,
}

pub struct DLABuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
    algorithm: DLAAlgorithm,
    symmetry: Symmetry,
    brush_size: i32,
    floor_percent: f32,
    spawn_list: Vec<(usize, String)>,
}

impl MapBuilder for DLABuilder {
    fn build_map(&mut self) {
        self.build();
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

    fn get_spawn_list(&self) -> &Vec<(usize, String)> {
        &self.spawn_list
    }
}

impl DLABuilder {
    pub fn new_random(new_depth: i32) -> DLABuilder {
        let mut rng = rltk::RandomNumberGenerator::new();
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
            spawn_list: Vec::new(),
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
            symmetry: Symmetry::None,
            brush_size: 1,
            floor_percent: 0.25,
            spawn_list: Vec::new(),
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
            symmetry: Symmetry::None,
            brush_size: 2,
            floor_percent: 0.25,
            spawn_list: Vec::new(),
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
            symmetry: Symmetry::None,
            brush_size: 2,
            floor_percent: 0.25,
            spawn_list: Vec::new(),
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
            symmetry: Symmetry::Horizontal,
            brush_size: 2,
            floor_percent: 0.25,
            spawn_list: Vec::new(),
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

        for area in self.noise_areas.iter().skip(1) {
            spawner::spawn_region(
                &self.map,
                &mut rng,
                area.1,
                self.depth,
                &mut self.spawn_list,
            );
        }
    }

    fn seed_start(&mut self, start_idx: usize) {
        self.map.tiles[start_idx] = TileType::Floor;
        self.map.tiles[start_idx - 1] = TileType::Floor;
        self.map.tiles[start_idx + 1] = TileType::Floor;
        self.map.tiles[start_idx - self.map.width as usize] = TileType::Floor;
        self.map.tiles[start_idx + self.map.width as usize] = TileType::Floor;
    }

    fn walk_inwards(&mut self, desired_floor_tiles: usize, rng: &mut RandomNumberGenerator) {
        let mut floor_tile_count = self.map.count_floor_tiles();
        while floor_tile_count < desired_floor_tiles {
            let mut drunk = TileDigger::new(
                rng.roll_dice(1, self.map.width - 3) + 1,
                rng.roll_dice(1, self.map.height - 3) + 1,
                TileType::Wall,
            );

            let (prev_x, prev_y) = drunk.stagger(&mut self.map, rng);
            paint(
                &mut self.map,
                self.symmetry,
                self.brush_size,
                prev_x,
                prev_y,
            );
            floor_tile_count = self.map.count_floor_tiles();
        }
    }

    fn walk_outwards(&mut self, desired_floor_tiles: usize, rng: &mut RandomNumberGenerator) {
        let mut floor_tile_count = self.map.count_floor_tiles();
        let mut drunk = TileDigger::new(
            self.starting_position.x,
            self.starting_position.y,
            TileType::Floor,
        );

        while floor_tile_count < desired_floor_tiles {
            drunk.stagger(&mut self.map, rng);
            floor_tile_count = self.map.count_floor_tiles();
        }

        paint(
            &mut self.map,
            self.symmetry,
            self.brush_size,
            drunk.x,
            drunk.y,
        );
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

            paint(
                &mut self.map,
                self.symmetry,
                self.brush_size,
                prev.x,
                prev.y,
            );
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

/// Digger that staggers around the map, creating open areas.
pub struct TileDigger {
    // Digger's current x position
    pub x: i32,
    // Diggers current y position
    pub y: i32,
    // Floor for [`walk_outwards()`], Wall for [`walk_inwards()`]
    tile_type: TileType,
    // Current (x, y) in the map index (1D array indexing from 2D index)
    idx: usize,
}

impl TileDigger {
    /// Creates and returns a new [`DrunkDigger`].
    pub fn new(x: i32, y: i32, tile_type: TileType) -> TileDigger {
        TileDigger {
            x: x,
            y: y,
            // life: life,
            tile_type: tile_type,
            idx: usize::default(),
        }
    }
}

impl Digger for TileDigger {
    fn get_position(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    fn get_position_mut(&mut self) -> (&mut i32, &mut i32) {
        (&mut self.x, &mut self.y)
    }

    fn set_position(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    /// Moves the digger around the map one tile at a time in a random direction
    /// until they've run out of life.
    ///
    /// Uses [`TileType::DownStairs`] as a marker to differentiate tiles dug by the
    /// digger (the [`TileType::DownStairs`] tiles) from tiles that were already
    /// floor tiles; keeps us from having to add another TileType enum variant,
    /// which could possibly break exhaustion on TileType match statements.
    /// These will be turned into floor tiles during the `build` loop.
    fn stagger(&mut self, map: &mut Map, rng: &mut rltk::RandomNumberGenerator) -> (i32, i32) {
        let mut prev_pos: (i32, i32) = (self.x, self.y);
        while map.tiles[self.idx] == self.tile_type {
            prev_pos = (self.x, self.y);
            self.stagger_direction(map, rng);
            self.idx = map.xy_idx(self.x, self.y);
        }
        prev_pos
    }
}
