#![allow(dead_code, unused_imports)]

use specs::prelude::World;

use bsp_dungeon::BspDungeonBuilder;
use bsp_interior::BspInteriorBuilder;
use cellular_automata::CellularAutomataBuilder;
use dla::DLABuilder;
use drunkard::DrunkardsWalkBuilder;
use maze::MazeBuilder;
use prefab_builder::PrefabBuilder;
use room_based_gen::{RoomBasedSpawner, RoomBasedStairs, RoomBasedStartingPosition};
use simple_map::SimpleMapBuilder;
use voronoi::VoronoiBuilder;
use waveform_collapse::WaveformCollapseBuilder;

use crate::{SHOW_MAPGEN_VISUALIZER, spawner};

use super::Rect;
use super::{components::Position, map::*};

mod bsp_dungeon;
mod bsp_interior;
mod cellular_automata;
mod common;
mod dla;
mod drunkard;
mod maze;
mod prefab_builder;
mod room_based_gen;
mod simple_map;
mod voronoi;
mod waveform_collapse;

/// Basic functionality all [`MapBuilder`] implementors must have.
pub trait MapBuilder {
    fn build_map(&mut self);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
    fn get_spawn_list(&self) -> &Vec<(usize, String)>;
    fn spawn_entities(&mut self, ecs: &mut World) {
        for entity in self.get_spawn_list().iter() {
            spawner::spawn_entity(ecs, &(&entity.0, &entity.1));
        }
    }
}

// /// Generates a new [`Map`] at a given depth using a random [`MapBuilder`].
// pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
//     // Box::new(WaveformCollapseBuilder::test_map(new_depth))
//     let mut rng = rltk::RandomNumberGenerator::new();
//     match rng.roll_dice(1, 18) {
//         1 => Box::new(BspDungeonBuilder::new(new_depth)),
//         2 => Box::new(BspInteriorBuilder::new(new_depth)),
//         3 => Box::new(CellularAutomataBuilder::new(new_depth)),
//         4 => Box::new(DrunkardsWalkBuilder::open_area(new_depth)),
//         5 => Box::new(DrunkardsWalkBuilder::open_halls(new_depth)),
//         6 => Box::new(DrunkardsWalkBuilder::winding_passages(new_depth)),
//         7 => Box::new(DrunkardsWalkBuilder::fat_passages(new_depth)),
//         8 => Box::new(DrunkardsWalkBuilder::fearful_symmetry(new_depth)),
//         9 => Box::new(MazeBuilder::new(new_depth)),
//         10 => Box::new(DLABuilder::new_walk_inwards(new_depth)),
//         11 => Box::new(DLABuilder::new_walk_outwards(new_depth)),
//         12 => Box::new(DLABuilder::new_central_attractor(new_depth)),
//         13 => Box::new(DLABuilder::new_insectoid(new_depth)),
//         14 => Box::new(DLABuilder::new_random(new_depth)),
//         15 => Box::new(VoronoiBuilder::pythagoras(new_depth)),
//         16 => Box::new(VoronoiBuilder::manhattan(new_depth)),
//         17 => Box::new(VoronoiBuilder::chebyshev(new_depth)),
//         _ => Box::new(SimpleMapBuilder::new(new_depth)),
//     }
// }

pub fn random_builder(new_depth: i32, rng: &mut rltk::RandomNumberGenerator) -> BuilderChain {
    let mut builder = BuilderChain::new(new_depth);
    builder.start_with(SimpleMapBuilder::new());
    builder.with(RoomBasedSpawner::new());
    builder.with(RoomBasedStartingPosition::new());
    builder.with(RoomBasedStairs::new());
    builder
}

pub struct BuildData {
    pub spawn_list: Vec<(usize, String)>,
    pub map: Map,
    pub start: Option<Position>,
    pub rooms: Option<Vec<Rect>>,
    pub history: Vec<Map>,
}

impl BuildData {
    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            snapshot.revealed_tiles.iter_mut().for_each(|v| *v = true);
            self.history.push(snapshot);
        }
    }
}

pub struct BuilderChain {
    starter: Option<Box<dyn InitialMapBuilder>>,
    builders: Vec<Box<dyn MetaMapBuilder>>,
    pub build_data: BuildData,
}

impl BuilderChain {
    pub fn new(new_depth: i32) -> BuilderChain {
        BuilderChain {
            starter: None,
            builders: Vec::new(),
            build_data: BuildData {
                spawn_list: Vec::new(),
                map: Map::new(new_depth),
                start: None,
                rooms: None,
                history: Vec::new(),
            }
        }
    }

    /// Adds a starting map to the chain of builders.
    pub fn start_with(&mut self, starter: Box<dyn InitialMapBuilder>) {
        match self.starter {
            None => self.starter = Some(starter),
            Some(_) => panic!("BuilderChain can only accept a single starting builder"),
        }
    }

    pub fn with(&mut self, metabuilder: Box<dyn MetaMapBuilder>) {
        self.builders.push(metabuilder);
    }

    pub fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator) {
        match &mut self.starter {
            None => panic!("Cannot run a BuilderChain"),
            Some(starter) => starter.build_map(rng, &mut self.build_data),
        }

        for metabuilder in self.builders.iter_mut() {
            metabuilder.build_map(rng, &mut self.build_data);
        }
    }

    pub fn spawn_entities(&mut self, ecs: &mut World) {
        for entity in self.build_data.spawn_list.iter() {
            spawner::spawn_entity(ecs, &(&entity.0, &entity.1));
        }
    }
}

pub trait InitialMapBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuildData);
}

pub trait MetaMapBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuildData);
}
