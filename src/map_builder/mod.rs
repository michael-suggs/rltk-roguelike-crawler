use specs::prelude::World;

use bsp_dungeon::BspDungeonBuilder;
use bsp_interior::BspInteriorBuilder;
use cellular_automata::CellularAutomataBuilder;
use drunkard::DrunkardsWalkBuilder;
use simple_map::SimpleMapBuilder;
use super::{
    components::Position,
    map::*
};
use super::Rect;

mod bsp_dungeon;
mod bsp_interior;
mod cellular_automata;
mod common;
mod drunkard;
mod simple_map;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&mut self, ecs: &mut World);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    let mut rng = rltk::RandomNumberGenerator::new();
    match rng.roll_dice(1, 4) {
        // 1 => Box::new(SimpleMapBuilder::new(new_depth)),
        // 2 => Box::new(BspDungeonBuilder::new(new_depth)),
        // 3 => Box::new(BspInteriorBuilder::new(new_depth)),
        // 4 => Box::new(CellularAutomataBuilder::new(new_depth)),
        _ => Box::new(DrunkardsWalkBuilder::new(new_depth)),
    }
}
