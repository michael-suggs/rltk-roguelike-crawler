#![allow(dead_code, unused_imports)]

use specs::prelude::World;

use bsp_dungeon::BspDungeonBuilder;
use bsp_interior::BspInteriorBuilder;
use cellular_automata::CellularAutomataBuilder;
use dla::DLABuilder;
use drunkard::DrunkardsWalkBuilder;
use maze::MazeBuilder;
use prefab_builder::PrefabBuilder;
use simple_map::SimpleMapBuilder;
use voronoi::VoronoiBuilder;
use waveform_collapse::WaveformCollapseBuilder;

use crate::spawner;

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

/// Generates a new [`Map`] at a given depth using a random [`MapBuilder`].
pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    // Box::new(WaveformCollapseBuilder::test_map(new_depth))
    // let mut rng = rltk::RandomNumberGenerator::new();
    // match rng.roll_dice(1, 18) {
    //     1 => Box::new(BspDungeonBuilder::new(new_depth)),
    //     2 => Box::new(BspInteriorBuilder::new(new_depth)),
    //     3 => Box::new(CellularAutomataBuilder::new(new_depth)),
    //     4 => Box::new(DrunkardsWalkBuilder::open_area(new_depth)),
    //     5 => Box::new(DrunkardsWalkBuilder::open_halls(new_depth)),
    //     6 => Box::new(DrunkardsWalkBuilder::winding_passages(new_depth)),
    //     7 => Box::new(DrunkardsWalkBuilder::fat_passages(new_depth)),
    //     8 => Box::new(DrunkardsWalkBuilder::fearful_symmetry(new_depth)),
    //     9 => Box::new(MazeBuilder::new(new_depth)),
    //     10 => Box::new(DLABuilder::new_walk_inwards(new_depth)),
    //     11 => Box::new(DLABuilder::new_walk_outwards(new_depth)),
    //     12 => Box::new(DLABuilder::new_central_attractor(new_depth)),
    //     13 => Box::new(DLABuilder::new_insectoid(new_depth)),
    //     14 => Box::new(DLABuilder::new_random(new_depth)),
    //     15 => Box::new(VoronoiBuilder::pythagoras(new_depth)),
    //     16 => Box::new(VoronoiBuilder::manhattan(new_depth)),
    //     17 => Box::new(VoronoiBuilder::chebyshev(new_depth)),
    //     _ => Box::new(SimpleMapBuilder::new(new_depth)),
    // }
    // Box::new(VoronoiBuilder::chebyshev(new_depth))
    Box::new(PrefabBuilder::new(
        new_depth,
        Some(Box::new(CellularAutomataBuilder::new(new_depth))),
    ))
}
