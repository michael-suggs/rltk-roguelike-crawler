use specs::prelude::World;

use bsp_dungeon::BspDungeonBuilder;
use bsp_interior::BspInteriorBuilder;
use cellular_automata::CellularAutomataBuilder;
use dla::DLABuilder;
use drunkard::DrunkardsWalkBuilder;
use maze::MazeBuilder;
use simple_map::SimpleMapBuilder;

use super::Rect;
use super::{components::Position, map::*};

mod bsp_dungeon;
mod bsp_interior;
mod cellular_automata;
mod common;
mod dla;
mod drunkard;
mod maze;
mod simple_map;

/// Basic functionality all [`MapBuilder`] implementors must have.
pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&mut self, ecs: &mut World);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
}

/// Generates a new [`Map`] at a given depth using a random [`MapBuilder`].
pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    // let mut rng = rltk::RandomNumberGenerator::new();
    // match rng.roll_dice(1, 8) {
    //     1 => Box::new(BspDungeonBuilder::new(new_depth)),
    //     2 => Box::new(BspInteriorBuilder::new(new_depth)),
    //     3 => Box::new(CellularAutomataBuilder::new(new_depth)),
    //     4 => Box::new(DrunkardsWalkBuilder::open_area(new_depth)),
    //     5 => Box::new(DrunkardsWalkBuilder::open_halls(new_depth)),
    //     6 => Box::new(DrunkardsWalkBuilder::winding_passages(new_depth)),
    //     7 => Box::new(MazeBuilder::new(new_depth)),
    //     _ => Box::new(SimpleMapBuilder::new(new_depth)),
    // }
    Box::new(DLABuilder::new(new_depth))
}
