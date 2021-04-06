use crate::{Map, TileType};

use super::constraints::Chunk;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct MapChunk {
    pub pattern: Vec<TileType>,
    pub exits: [Vec<bool>; 4],
    pub has_exits: bool,
    pub compatible_with: [Vec<usize>; 4],
}

pub fn tile_idx_in_chunks(chunk_size: i32, x: i32, y: i32) -> usize {
    ((y * chunk_size) + x) as usize
}

#[derive(Clone, Copy)]
pub enum Direction {
    North = 0,
    South = 1,
    West = 2,
    East = 3,
}

impl Direction {
    pub fn get_index(&self, chunk_size: i32, x: i32) -> usize {
        match self {
            Direction::North => tile_idx_in_chunks(chunk_size, x, 0),
            Direction::South => tile_idx_in_chunks(chunk_size, x, chunk_size - 1),
            Direction::West => tile_idx_in_chunks(chunk_size, 0, x),
            Direction::East => tile_idx_in_chunks(chunk_size, chunk_size - 1, x),
        }
    }

    pub fn map_index(&self, map: &Map, chunk: &Chunk, x: i32) -> usize {
        match self {
            Direction::North => map.xy_idx(chunk.start.x + x, chunk.start.y),
            Direction::South => map.xy_idx(chunk.start.x + x, chunk.start.y + chunk.size - 1),
            Direction::West => map.xy_idx(chunk.start.x, chunk.start.y + x),
            Direction::East => map.xy_idx(chunk.start.x + chunk.size - 1, chunk.start.y + x),
        }
    }

    pub fn opposite(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
        }
    }

    pub fn get_indices(chunk_size: i32, x: i32) -> Vec<usize> {
        Direction::iterator().map(|d| d.get_index(chunk_size, x)).collect::<Vec<usize>>()
    }

    pub fn iterator() -> impl Iterator<Item = Direction> {
        [Direction::North, Direction::South, Direction::East, Direction::West].iter().copied()
    }
}
