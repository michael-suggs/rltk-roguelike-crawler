use specs::prelude::World;

use bsp_dungeon::BspDungeonBuilder;
use simple_map::SimpleMapBuilder;
use super::{
    components::Position,
    map::*
};
use super::Rect;

mod common;
mod bsp_dungeon;
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
    match rng.roll_dice(1, 2) {
        1 => Box::new(BspDungeonBuilder::new(new_depth)),
        _ => Box::new(SimpleMapBuilder::new(new_depth)),
    }
}
