use specs::prelude::World;

use simple_map::SimpleMapBuilder;
use super::Rect;

pub use super::{
    components::Position,
    map::*,
};

mod common;
mod simple_map;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&mut self, ecs: &mut World);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    Box::new(SimpleMapBuilder::new(new_depth))
}
