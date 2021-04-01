use std::collections::HashMap;

use rltk::RandomNumberGenerator;
use specs::prelude::*;

use super::common::{
    generate_voronoi_spawn_regions, remove_unreachable_areas_returning_most_distant,
};
use super::{Map, MapBuilder, Position, TileType};
use crate::{spawner, SHOW_MAPGEN_VISUALIZER};

/// Sets where drunkards will start when generating a drunkards' walk map.
#[derive(PartialEq, Copy, Clone)]
pub enum DrunkSpawnMode {
    StartingPoint,
    Random,
}

/// Controls how the drunkards will generate the map.
pub struct DrunkardSettings {
    pub spawn_mode: DrunkSpawnMode,
    pub drunken_lifetime: i32,
    pub floor_percent: f32,
}

/// Builder to construct a drunkards' walk map.
pub struct DrunkardsWalkBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
    settings: DrunkardSettings,
}

impl MapBuilder for DrunkardsWalkBuilder {
    /// Generates the map
    fn build_map(&mut self) {
        self.build();
    }

    /// Spawns enemies and items on the map
    fn spawn_entities(&mut self, ecs: &mut World) {
        for area in self.noise_areas.iter() {
            spawner::spawn_region(ecs, area.1, self.depth);
        }
    }

    /// Returns the map itself
    fn get_map(&self) -> Map {
        self.map.clone()
    }

    /// Returns the player's starting position on the map
    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    /// Gets a vector of all map generation stages for visualization
    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    /// Clones the map at a timestep during generation, for later visualization
    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            snapshot.revealed_tiles.iter_mut().for_each(|v| *v = true);
            self.history.push(snapshot);
        }
    }
}

impl DrunkardsWalkBuilder {
    /// Generates a new drunkards' walk map with passed settings
    pub fn new(new_depth: i32, settings: DrunkardSettings) -> DrunkardsWalkBuilder {
        DrunkardsWalkBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            settings,
        }
    }

    /// Generates a new drunkards' wak map focusing on a large, open area
    pub fn open_area(new_depth: i32) -> DrunkardsWalkBuilder {
        DrunkardsWalkBuilder::new(
            new_depth,
            DrunkardSettings {
                spawn_mode: DrunkSpawnMode::StartingPoint,
                drunken_lifetime: 400,
                floor_percent: 0.5,
            },
        )
    }

    /// Generates a new drunkards' walk map, focusing on having open halls
    pub fn open_halls(new_depth: i32) -> DrunkardsWalkBuilder {
        DrunkardsWalkBuilder::new(
            new_depth,
            DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                drunken_lifetime: 400,
                floor_percent: 0.5,
            },
        )
    }

    /// Generates a new drunkards' walk map, focusing on winding corridors
    pub fn winding_passages(new_depth: i32) -> DrunkardsWalkBuilder {
        DrunkardsWalkBuilder::new(
            new_depth,
            DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                drunken_lifetime: 100,
                floor_percent: 0.4,
            },
        )
    }

    /// Builds the drunkards' walk map, using settings from one of the above constructors
    #[allow(clippy::map_entry)]
    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        self.starting_position = Position::from(self.map.center());
        let start_idx = self
            .map
            .xy_idx(self.starting_position.x, self.starting_position.y);
        self.map.tiles[start_idx] = TileType::Floor;

        // Total number of tiles on the map
        let total_tiles = self.map.width * self.map.height;
        // Number of floor tiles we want on the generated map
        let desired_floor_tiles = (self.settings.floor_percent * total_tiles as f32) as usize;
        // Current number of floor tiles on the map
        let mut floor_tile_count = self
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
                    drunk_x = self.starting_position.x;
                    drunk_y = self.starting_position.y;
                }
                DrunkSpawnMode::Random => {
                    if digger_count == 0 {
                        // If this is the first drunkard, always start at the player's start
                        // to ensure the player has room to spawn and move.
                        drunk_x = self.starting_position.x;
                        drunk_y = self.starting_position.y;
                    } else {
                        // If not the first drunkard, start them somewhere on the map
                        drunk_x = rng.roll_dice(1, self.map.width - 3) + 1;
                        drunk_y = rng.roll_dice(1, self.map.height - 3) + 1;
                    }
                }
            }

            // Create a drunk to stagger around the map
            let mut drunk =
                DrunkDigger::new(drunk_x, drunk_y, self.settings.drunken_lifetime, &mut rng);

            // This actually does the map generation, staggering the drunk around
            // the map and digging until its life expires.
            drunk.stagger(&mut self.map);
            if drunk.did_something {
                self.take_snapshot();
                active_digger_count += 1;
            }

            digger_count += 1;
            // Set all the drunk's marked (dug) tiles to floor tiles
            self.map.tiles.iter_mut().for_each(|tile| {
                if *tile == TileType::DownStairs {
                    *tile = TileType::Floor;
                }
            });
            // Get the new floor tile count before finishing the loop
            floor_tile_count = self
                .map
                .tiles
                .iter()
                .filter(|t| **t == TileType::Floor)
                .count();
        }

        // Get rid of unreachable areas and get the furthest reachable tile
        let exit_tile = remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);
        self.take_snapshot();

        // Set the furthest reachable tile to be the stairs down to the next level
        self.map.tiles[exit_tile] = TileType::DownStairs;
        self.take_snapshot();

        // Generate map noise
        self.noise_areas = generate_voronoi_spawn_regions(&self.map, &mut rng);
    }
}

/// Digger that staggers around the map, creating open areas.
pub struct DrunkDigger<'a> {
    // Digger's current x position
    pub x: i32,
    // Diggers current y position
    pub y: i32,
    // If the digger has actually dug any wall tiles out
    pub did_something: bool,
    // How long the drunk will stagger for
    pub life: i32,
    rng: &'a mut RandomNumberGenerator,
    // Current (x, y) in the map index (1D array indexing from 2D index)
    idx: usize,
}

impl<'a> DrunkDigger<'a> {
    /// Creates and returns a new [`DrunkDigger`].
    pub fn new(x: i32, y: i32, life: i32, rng: &mut RandomNumberGenerator) -> DrunkDigger {
        DrunkDigger {
            x: x,
            y: y,
            did_something: false,
            life: life,
            rng: rng,
            idx: usize::default(),
        }
    }

    /// Moves the drunk around the map one tile at a time in a random direction
    /// until they've run out of life.
    ///
    /// Uses [`TileType::DownStairs`] as a marker to differentiate tiles dug by the
    /// digger (the [`TileType::DownStairs`] tiles) from tiles that were already
    /// floor tiles; keeps us from having to add another TileType enum variant,
    /// which could possibly break exhaustion on TileType match statements.
    /// These will be turned into floor tiles during the `build` loop.
    pub fn stagger(&mut self, map: &mut Map) {
        while self.life > 0 {
            self.idx = map.xy_idx(self.x, self.y);
            // If they've landed on a wall tile, dig it out
            if map.tiles[self.idx] == TileType::Wall {
                self.did_something = true;
            }
            // Mark the tiles dug by the digger
            map.tiles[self.idx] = TileType::DownStairs;
            // Get its position for the next iteration and reduce its remaining life
            self.stagger_direction(map);
            self.life -= 1;
        }
    }

    /// Randomly generates the digger's new position, and moves them to it.
    /// Moves one tile (at most) in one of the four cardinal directions.
    pub fn stagger_direction(&mut self, map: &Map) {
        // Roll dice to pick a direction to move, then update the digger's
        // position based on said roll. If movement would take the digger
        // outside the map bounds, do nothing instead.
        match self.rng.roll_dice(1, 4) {
            1 => {
                if self.x > 2 {
                    self.x -= 1
                }
            }
            2 => {
                if self.x < map.width - 2 {
                    self.x += 1;
                }
            }
            3 => {
                if self.y > 2 {
                    self.y -= 1;
                }
            }
            _ => {
                if self.y < map.height - 2 {
                    self.y += 1;
                }
            }
        };
    }
}
