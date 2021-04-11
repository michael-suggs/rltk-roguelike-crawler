use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::collections::HashMap;

use rltk::RandomNumberGenerator;

use crate::{
    spawner, BuildData, InitialMapBuilder, Map, MapBuilder, Position, TileType,
    SHOW_MAPGEN_VISUALIZER,
};

use super::common::{paint, Digger, Symmetry};

#[derive(PartialEq, Clone, Copy)]
pub enum DLAAlgorithm {
    WalkInwards,
    WalkOutwards,
    CentralAttractor,
}

pub struct DLABuilder {
    algorithm: DLAAlgorithm,
    symmetry: Symmetry,
    brush_size: i32,
    floor_percent: f32,
}

impl InitialMapBuilder for DLABuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuildData) {
        self.build(rng, build_data);
    }
}

impl DLABuilder {
    pub fn new() -> Box<DLABuilder> {
        match RandomNumberGenerator::new().roll_dice(1, 5) {
            1 => DLABuilder::new_random(),
            2 => DLABuilder::new_walk_inwards(),
            3 => DLABuilder::new_walk_outwards(),
            4 => DLABuilder::new_central_attractor(),
            _ => DLABuilder::new_insectoid(),
        }
    }

    pub fn new_random() -> Box<DLABuilder> {
        let mut rng = rltk::RandomNumberGenerator::new();
        Box::new(DLABuilder {
            algorithm: rand::random(),
            symmetry: rand::random(),
            brush_size: rng.roll_dice(1, 3),
            floor_percent: 0.25,
        })
    }

    pub fn new_walk_inwards() -> Box<DLABuilder> {
        Box::new(DLABuilder {
            algorithm: DLAAlgorithm::WalkInwards,
            symmetry: Symmetry::None,
            brush_size: 1,
            floor_percent: 0.25,
        })
    }

    pub fn new_walk_outwards() -> Box<DLABuilder> {
        Box::new(DLABuilder {
            algorithm: DLAAlgorithm::WalkOutwards,
            symmetry: Symmetry::None,
            brush_size: 2,
            floor_percent: 0.25,
        })
    }

    pub fn new_central_attractor() -> Box<DLABuilder> {
        Box::new(DLABuilder {
            algorithm: DLAAlgorithm::WalkInwards,
            symmetry: Symmetry::None,
            brush_size: 2,
            floor_percent: 0.25,
        })
    }

    pub fn new_insectoid() -> Box<DLABuilder> {
        Box::new(DLABuilder {
            algorithm: DLAAlgorithm::WalkInwards,
            symmetry: Symmetry::Horizontal,
            brush_size: 2,
            floor_percent: 0.25,
        })
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuildData) {
        let start = Position::from(build_data.map.center());
        let start_idx = build_data.map.xy_idx(start.x, start.y);
        build_data.take_snapshot();
        DLABuilder::seed_start(&mut build_data.map, start_idx);

        // let total_tiles = self.map.tiles.len() as i32;
        let desired_floor_tiles = (self.floor_percent * build_data.map.tiles.len() as f32) as usize;

        match self.algorithm {
            DLAAlgorithm::WalkInwards => self.walk_inwards(desired_floor_tiles, rng, build_data),
            DLAAlgorithm::WalkOutwards => self.walk_outwards(desired_floor_tiles, rng, build_data),
            DLAAlgorithm::CentralAttractor => {
                self.central_attractor(desired_floor_tiles, rng, build_data)
            }
        }
    }

    fn seed_start(map: &mut Map, start_idx: usize) {
        map.tiles[start_idx] = TileType::Floor;
        map.tiles[start_idx - 1] = TileType::Floor;
        map.tiles[start_idx + 1] = TileType::Floor;
        map.tiles[start_idx - map.width as usize] = TileType::Floor;
        map.tiles[start_idx + map.width as usize] = TileType::Floor;
    }

    fn walk_inwards(
        &mut self,
        desired_floor_tiles: usize,
        rng: &mut RandomNumberGenerator,
        build_data: &mut BuildData,
    ) {
        let mut floor_tile_count = build_data.map.count_floor_tiles();
        while floor_tile_count < desired_floor_tiles {
            let mut drunk = TileDigger::new(
                rng.roll_dice(1, build_data.map.width - 3) + 1,
                rng.roll_dice(1, build_data.map.height - 3) + 1,
                TileType::Wall,
            );

            let (prev_x, prev_y) = drunk.stagger(&mut build_data.map, rng);
            paint(
                &mut build_data.map,
                self.symmetry,
                self.brush_size,
                prev_x,
                prev_y,
            );
            floor_tile_count = build_data.map.count_floor_tiles();
        }
    }

    fn walk_outwards(
        &mut self,
        desired_floor_tiles: usize,
        rng: &mut RandomNumberGenerator,
        build_data: &mut BuildData,
    ) {
        let mut floor_tile_count = build_data.map.count_floor_tiles();
        let mut drunk = TileDigger::new(
            build_data.start.unwrap().x,
            build_data.start.unwrap().y,
            TileType::Floor,
        );

        while floor_tile_count < desired_floor_tiles {
            drunk.stagger(&mut build_data.map, rng);
            floor_tile_count = build_data.map.count_floor_tiles();
        }

        paint(
            &mut build_data.map,
            self.symmetry,
            self.brush_size,
            drunk.x,
            drunk.y,
        );
    }

    fn central_attractor(
        &mut self,
        desired_floor_tiles: usize,
        rng: &mut RandomNumberGenerator,
        build_data: &mut BuildData,
    ) {
        let mut floor_tile_count = build_data.map.count_floor_tiles();
        while floor_tile_count < desired_floor_tiles {
            let mut digger = Position {
                x: rng.roll_dice(1, build_data.map.width - 3) + 1,
                y: rng.roll_dice(1, build_data.map.height - 3) + 1,
            };
            let mut prev = digger.clone();
            let mut digger_idx = build_data.map.xy_idx(digger.x, digger.y);

            let mut path = rltk::line2d(
                rltk::LineAlg::Bresenham,
                rltk::Point::new(digger.x, digger.y),
                rltk::Point::new(build_data.start.unwrap().x, build_data.start.unwrap().y),
            );

            while build_data.map.tiles[digger_idx] == TileType::Wall && !path.is_empty() {
                prev = digger;
                digger = Position {
                    x: path[0].x,
                    y: path[0].y,
                };
                path.remove(0);
                digger_idx = build_data.map.xy_idx(digger.x, digger.y);
            }

            paint(
                &mut build_data.map,
                self.symmetry,
                self.brush_size,
                prev.x,
                prev.y,
            );
            floor_tile_count = build_data.map.count_floor_tiles();
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
