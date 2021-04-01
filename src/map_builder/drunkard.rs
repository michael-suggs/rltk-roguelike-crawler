use std::collections::HashMap;

use rltk::RandomNumberGenerator;
use specs::prelude::*;

use super::common::{
    generate_voronoi_spawn_regions, remove_unreachable_areas_returning_most_distant, DrunkDigger,
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
            let mut drunk = DrunkDigger::new(drunk_x, drunk_y, &mut rng);

            // This actually does the map generation, staggering the drunk around
            // the map and digging until its life expires.
            drunk.stagger_lifetime(&mut self.map, self.settings.drunken_lifetime);
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
