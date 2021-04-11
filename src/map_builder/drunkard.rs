use std::collections::HashMap;

use rltk::RandomNumberGenerator;
use specs::prelude::*;

use super::common::{
    generate_voronoi_spawn_regions, paint, remove_unreachable_areas_returning_most_distant, Digger,
    Symmetry,
};
use super::{Map, MapBuilder, Position, TileType};
use crate::{BuildData, InitialMapBuilder, SHOW_MAPGEN_VISUALIZER, spawner};

/// Sets where drunkards will start when generating a drunkards' walk map.
#[derive(PartialEq, Copy, Clone)]
pub enum DrunkSpawnMode {
    StartingPoint,
    Random,
}

/// Controls how the drunkards will generate the map.
#[derive(Copy, Clone)]
pub struct DrunkardSettings {
    pub spawn_mode: DrunkSpawnMode,
    pub lifespan: i32,
    pub floor_percent: f32,
    pub brush_size: i32,
    pub symmetry: Symmetry,
}

/// Builder to construct a drunkards' walk map.
pub struct DrunkardsWalkBuilder {
    settings: DrunkardSettings,
}

impl InitialMapBuilder for DrunkardsWalkBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuildData) {
        self.build(rng, build_data);
    }
}

impl DrunkardsWalkBuilder {
    /// Generates a new drunkards' walk map with passed settings
    pub fn new(settings: DrunkardSettings) -> Box<DrunkardsWalkBuilder> {
        Box::new(DrunkardsWalkBuilder { settings })
    }

    pub fn random() -> Box<DrunkardsWalkBuilder> {
        match RandomNumberGenerator::new().roll_dice(1, 5) {
            1 => DrunkardsWalkBuilder::open_area(),
            2 => DrunkardsWalkBuilder::open_halls(),
            3 => DrunkardsWalkBuilder::winding_passages(),
            4 => DrunkardsWalkBuilder::fat_passages(),
            _ => DrunkardsWalkBuilder::fearful_symmetry(),
        }
    }

    /// Generates a new drunkards' wak map focusing on a large, open area
    pub fn open_area() -> Box<DrunkardsWalkBuilder> {
        DrunkardsWalkBuilder::new(
            DrunkardSettings {
                spawn_mode: DrunkSpawnMode::StartingPoint,
                lifespan: 400,
                floor_percent: 0.5,
                brush_size: 1,
                symmetry: Symmetry::None,
            },
        )
    }

    /// Generates a new drunkards' walk map, focusing on having open halls
    pub fn open_halls() -> Box<DrunkardsWalkBuilder> {
        DrunkardsWalkBuilder::new(
            DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                lifespan: 400,
                floor_percent: 0.5,
                brush_size: 1,
                symmetry: Symmetry::None,
            },
        )
    }

    /// Generates a new drunkards' walk map, focusing on winding corridors
    pub fn winding_passages() -> Box<DrunkardsWalkBuilder> {
        DrunkardsWalkBuilder::new(
            DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                lifespan: 100,
                floor_percent: 0.4,
                brush_size: 1,
                symmetry: Symmetry::None,
            },
        )
    }

    /// Generates a map with double-sized corridors--gives a cave-like map.
    pub fn fat_passages() -> Box<DrunkardsWalkBuilder> {
        DrunkardsWalkBuilder::new(
            DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                lifespan: 100,
                floor_percent: 0.4,
                brush_size: 2,
                symmetry: Symmetry::None,
            },
        )
    }

    /// Generates a winding-passages map with symmetry in both directions.
    pub fn fearful_symmetry() -> Box<DrunkardsWalkBuilder> {
        DrunkardsWalkBuilder::new(
            DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                lifespan: 100,
                floor_percent: 0.4,
                brush_size: 1,
                symmetry: Symmetry::Both,
            },
        )
    }

    /// Builds the drunkards' walk map, using settings from one of the above constructors
    #[allow(clippy::map_entry)]
    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuildData) {
        let start = Position::from(build_data.map.center());
        let start_idx = build_data.map.xy_idx(start.x, start.y);
        build_data.map.tiles[start_idx] = TileType::Floor;

        // Total number of tiles on the map
        let total_tiles = build_data.map.width * build_data.map.height;
        // Number of floor tiles we want on the generated map
        let desired_floor_tiles = (self.settings.floor_percent * total_tiles as f32) as usize;
        // Current number of floor tiles on the map
        let mut floor_tile_count = build_data
            .map
            .tiles
            .iter()
            .filter(|t| **t == TileType::Floor)
            .count();
        // Number of diggers we've gone through during generation so far
        let mut digger_count = 0;
        // Number of diggers that have done something during generation
        let mut active_digger_count = 0;

        // Let drunkards dig until we've reached the desired number of floor tiles
        while floor_tile_count < desired_floor_tiles {
            let drunk_x: i32;
            let drunk_y: i32;

            // Get `drunk_x` and `drunk_y` based on the passed spawn mode
            match self.settings.spawn_mode {
                DrunkSpawnMode::StartingPoint => {
                    // Start at the player's starting position
                    drunk_x = start.x;
                    drunk_y = start.y;
                }
                DrunkSpawnMode::Random => {
                    if digger_count == 0 {
                        // If this is the first drunkard, always start at the player's start
                        // to ensure the player has room to spawn and move.
                        drunk_x = start.x;
                        drunk_y = start.y;
                    } else {
                        // If not the first drunkard, start them somewhere on the map
                        drunk_x = rng.roll_dice(1, build_data.map.width - 3) + 1;
                        drunk_y = rng.roll_dice(1, build_data.map.height - 3) + 1;
                    }
                }
            }

            // Create a drunk to stagger around the map
            let mut drunk = DrunkDigger::new(drunk_x, drunk_y, self.settings);

            // This actually does the map generation, staggering the drunk around
            // the map and digging until its life expires.
            drunk.stagger(&mut build_data.map, rng);
            if drunk.did_something {
                build_data.take_snapshot();
                active_digger_count += 1;
            }

            digger_count += 1;
            // Set all the drunk's marked (dug) tiles to floor tiles
            build_data.map.tiles.iter_mut().for_each(|tile| {
                if *tile == TileType::DownStairs {
                    *tile = TileType::Floor;
                }
            });
            // Get the new floor tile count before finishing the loop
            floor_tile_count = build_data
                .map
                .tiles
                .iter()
                .filter(|t| **t == TileType::Floor)
                .count();
        }
    }
}

/// Digger that staggers around the map, creating open areas.
pub struct DrunkDigger {
    // Digger's current x position
    pub x: i32,
    // Diggers current y position
    pub y: i32,
    // Current (x, y) in the map index (1D array indexing from 2D index)
    idx: usize,
    // If the digger has actually dug any wall tiles out
    pub did_something: bool,
    settings: DrunkardSettings,
}

impl Digger for DrunkDigger {
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

    /// Moves the drunk around the map one tile at a time in a random direction
    /// until they've run out of life.
    ///
    /// Uses [`TileType::DownStairs`] as a marker to differentiate tiles dug by the
    /// digger (the [`TileType::DownStairs`] tiles) from tiles that were already
    /// floor tiles; keeps us from having to add another TileType enum variant,
    /// which could possibly break exhaustion on TileType match statements.
    /// These will be turned into floor tiles during the `build` loop.
    fn stagger(&mut self, map: &mut Map, rng: &mut RandomNumberGenerator) -> (i32, i32) {
        let mut prev_position: (i32, i32) = self.get_position();
        while self.settings.lifespan > 0 {
            self.idx = map.xy_idx(self.x, self.y);
            // If they've landed on a wall tile, dig it out
            if map.tiles[self.idx] == TileType::Wall {
                self.did_something = true;
            }
            paint(
                map,
                self.settings.symmetry,
                self.settings.brush_size,
                self.x,
                self.y,
            );
            // Mark the tiles dug by the digger
            map.tiles[self.idx] = TileType::DownStairs;
            prev_position = self.get_position();
            // Get its position for the next iteration and reduce its remaining life
            self.stagger_direction(map, rng);
            self.settings.lifespan -= 1;
        }
        prev_position
    }
}

impl DrunkDigger {
    /// Creates and returns a new [`DrunkDigger`].
    pub fn new(x: i32, y: i32, settings: DrunkardSettings) -> DrunkDigger {
        DrunkDigger {
            x: x,
            y: y,
            idx: usize::default(),
            did_something: false,
            settings: settings,
        }
    }
}
