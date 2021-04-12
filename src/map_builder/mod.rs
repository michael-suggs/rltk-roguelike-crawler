#![allow(dead_code, unused_imports)]

use specs::prelude::World;

use area_based_gen::{AreaStartingPosition, VoronoiSpawning, XStart, YStart};
use bsp_dungeon::BspDungeonBuilder;
use bsp_interior::BspInteriorBuilder;
use cellular_automata::CellularAutomataBuilder;
use common::{CullUnreachable, DistantExit};
use dla::DLABuilder;
use drunkard::DrunkardsWalkBuilder;
use maze::MazeBuilder;
use prefab_builder::PrefabBuilder;
use room_based_gen::{RoomBasedSpawner, RoomBasedStairs, RoomBasedStartingPosition};
use simple_map::SimpleMapBuilder;
use voronoi::VoronoiBuilder;
use waveform_collapse::WaveformCollapseBuilder;

use crate::{spawner, SHOW_MAPGEN_VISUALIZER};

use super::Rect;
use super::{components::Position, map::*};

mod area_based_gen;
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

pub trait InitialMapBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuildData);
}

pub trait MetaMapBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuildData);
}

pub fn random_builder(new_depth: i32, rng: &mut rltk::RandomNumberGenerator) -> BuilderChain {
    BuilderChains::CellularAutomata.match_builder(new_depth)
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
            },
        }
    }

    /// Adds a starting map to the chain of builders.
    pub fn start_with(mut self, starter: Box<dyn InitialMapBuilder>) -> Self {
        match self.starter {
            None => self.starter = Some(starter),
            Some(_) => panic!("BuilderChain can only accept a single starting builder"),
        }
        self
    }

    pub fn with(mut self, metabuilder: Box<dyn MetaMapBuilder>) -> Self {
        self.builders.push(metabuilder);
        self
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

pub enum BuilderChains {
    SimpleMap,
    BspDungeon,
    BspInterior,
    CellularAutomata,
    DiffusionLimitedAggregation,
    DrunkardsWalk,
    Maze,
    Prefab,
    Voronoi,
}

impl BuilderChains {
    pub fn match_builder(&self, new_depth: i32) -> BuilderChain {
        match *self {
            BuilderChains::SimpleMap => BuilderChain::new(new_depth)
                .start_with(SimpleMapBuilder::new())
                .with(RoomBasedSpawner::new())
                .with(RoomBasedStartingPosition::new())
                .with(RoomBasedStairs::new()),
            BuilderChains::BspDungeon => BuilderChain::new(new_depth)
                .start_with(BspDungeonBuilder::new())
                .with(RoomBasedSpawner::new())
                .with(RoomBasedStartingPosition::new())
                .with(RoomBasedStairs::new()),
            BuilderChains::BspInterior => BuilderChain::new(new_depth)
                .start_with(BspInteriorBuilder::new())
                .with(RoomBasedSpawner::new())
                .with(RoomBasedStartingPosition::new())
                .with(RoomBasedStairs::new()),
            BuilderChains::CellularAutomata => BuilderChain::new(new_depth)
                .start_with(CellularAutomataBuilder::new())
                .with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER))
                .with(CullUnreachable::new())
                .with(VoronoiSpawning::new())
                .with(DistantExit::new()),
            BuilderChains::DrunkardsWalk => BuilderChain::new(new_depth)
                .start_with(DrunkardsWalkBuilder::random())
                .with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER))
                .with(CullUnreachable::new())
                .with(VoronoiSpawning::new())
                .with(DistantExit::new()),
            BuilderChains::DiffusionLimitedAggregation => BuilderChain::new(new_depth)
                .start_with(DLABuilder::new())
                .with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER))
                .with(CullUnreachable::new())
                .with(VoronoiSpawning::new())
                .with(DistantExit::new()),
            BuilderChains::Maze => BuilderChain::new(new_depth)
                .start_with(MazeBuilder::new())
                .with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER))
                .with(CullUnreachable::new())
                .with(VoronoiSpawning::new())
                .with(DistantExit::new()),
            BuilderChains::Voronoi => BuilderChain::new(new_depth)
                .start_with(VoronoiBuilder::new())
                .with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER))
                .with(CullUnreachable::new())
                .with(VoronoiSpawning::new())
                .with(DistantExit::new()),
            BuilderChains::Prefab => BuilderChain::new(new_depth)
                .start_with(VoronoiBuilder::pythagoras(64))
                .with(WaveformCollapseBuilder::new())
                .with(PrefabBuilder::room_vaults())
                .with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER))
                .with(CullUnreachable::new())
                .with(VoronoiSpawning::new())
                .with(PrefabBuilder::sectional(
                    prefab_builder::prefab_sections::UNDERGROUND_FORT,
                ))
                .with(DistantExit::new()),
            _ => panic!("BuilderChain yet implemented for specified builder!"),
        }
    }
}
