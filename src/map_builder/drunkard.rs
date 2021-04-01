use std::collections::HashMap;

use rltk::RandomNumberGenerator;
use specs::prelude::*;

use crate::{spawner, SHOW_MAPGEN_VISUALIZER};
use super::common::{
    remove_unreachable_areas_returning_most_distant,
    generate_voronoi_spawn_regions
};
use super::{MapBuilder, Map, TileType, Position};

#[derive(PartialEq, Copy, Clone)]
pub enum DrunkSpawnMode { StartingPoint, Random }

pub struct DrunkardSettings {
    pub spawn_mode: DrunkSpawnMode,
    pub drunken_lifetime: i32,
    pub floor_percent: f32,
}

pub struct DrunkardsWalkBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
    settings: DrunkardSettings,
}

impl MapBuilder for DrunkardsWalkBuilder {
    fn build_map(&mut self) {
        self.build();
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
        for area in self.noise_areas.iter() {
            spawner::spawn_region(ecs, area.1, self.depth);
        }
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
            snapshot
                .revealed_tiles
                .iter_mut()
                .for_each(|v| *v = true);
            self.history.push(snapshot);
        }
    }
}

impl DrunkardsWalkBuilder {
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

    pub fn open_area(new_depth: i32) -> DrunkardsWalkBuilder {
        DrunkardsWalkBuilder::new(
            new_depth,
            DrunkardSettings {
                spawn_mode: DrunkSpawnMode::StartingPoint,
                drunken_lifetime: 400,
                floor_percent: 0.5,
            }
        )
    }

    pub fn open_halls(new_depth: i32) -> DrunkardsWalkBuilder {
        DrunkardsWalkBuilder::new(
            new_depth,
            DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                drunken_lifetime: 400,
                floor_percent: 0.5,
            }
        )
    }

    pub fn winding_passages(new_depth: i32) -> DrunkardsWalkBuilder {
        DrunkardsWalkBuilder::new(
            new_depth,
            DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                drunken_lifetime: 100,
                floor_percent: 0.4,
            }
        )
    }

    #[allow(clippy::map_entry)]
    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        self.starting_position = Position::from(self.map.center());
        let start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        self.map.tiles[start_idx] = TileType::Floor;

        let total_tiles = self.map.width * self.map.height;
        let desired_floor_tiles = (self.settings.floor_percent * total_tiles as f32) as usize;
        let mut floor_tile_count =
            self.map.tiles.iter().filter(|t| **t == TileType::Floor).count();
        let mut digger_count = 0;
        let mut active_digger_count = 0;

        while floor_tile_count < desired_floor_tiles {
            let drunk_x: i32;
            let drunk_y: i32;

            match self.settings.spawn_mode {
                DrunkSpawnMode::StartingPoint => {
                    drunk_x = self.starting_position.x;
                    drunk_y = self.starting_position.y;
                },
                DrunkSpawnMode::Random => {
                    if digger_count == 0 {
                        drunk_x = self.starting_position.x;
                        drunk_y = self.starting_position.y;
                    } else {
                        drunk_x = rng.roll_dice(1, self.map.width - 3) + 1;
                        drunk_y = rng.roll_dice(1, self.map.height - 3) + 1;
                    }
                },
            }

            let mut drunk = DrunkDigger::new(
                drunk_x, drunk_y, self.settings.drunken_lifetime, &mut rng
            );

            drunk.stagger(&mut self.map);
            if drunk.did_something {
                self.take_snapshot();
                active_digger_count += 1;
            }

            digger_count += 1;
            self.map.tiles.iter_mut().for_each(|tile| {
                if *tile == TileType::DownStairs {
                    *tile = TileType::Floor;
                }
            });
            floor_tile_count = self.map.tiles.iter().filter(|t| **t == TileType::Floor).count();
        }

        let exit_tile = remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);
        self.take_snapshot();

        self.map.tiles[exit_tile] = TileType::DownStairs;
        self.take_snapshot();

        self.noise_areas = generate_voronoi_spawn_regions(&self.map, &mut rng);
    }
}

pub struct DrunkDigger<'a> {
    pub x: i32,
    pub y: i32,
    pub did_something: bool,
    pub life: i32,
    rng: &'a mut RandomNumberGenerator,
    idx: usize,
}

impl<'a> DrunkDigger<'a> {
    pub fn new(x: i32, y: i32, life: i32, rng: &mut RandomNumberGenerator) -> DrunkDigger {
        DrunkDigger {
            x: x,
            y: y,
            did_something: false,
            life: life,
            rng: rng,
            idx: usize::default()
        }
    }

    pub fn stagger(&mut self, map: &mut Map) {
        while self.life > 0 {
            self.idx = map.xy_idx(self.x, self.y);
            if map.tiles[self.idx] == TileType::Wall {
                self.did_something = true;
            }
            map.tiles[self.idx] = TileType::DownStairs;

            self.stagger_direction(map);
            self.life -= 1;
        };
    }

    pub fn stagger_direction(&mut self, map: &Map) {
        match self.rng.roll_dice(1, 4) {
            1 => if self.x > 2 { self.x -= 1 },
            2 => if self.x < map.width - 2 { self.x += 1; },
            3 => if self.y > 2 { self.y -= 1; },
            _ => if self.y < map.height - 2 { self.y += 1; },
        };

        self.life -= 1;
    }
}
