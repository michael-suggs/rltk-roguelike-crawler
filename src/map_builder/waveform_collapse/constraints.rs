use std::collections::HashSet;

use crate::{Map, Position, TileType};

use super::common::{Direction, MapChunk, tile_idx_in_chunks};

pub fn build_patterns(
    map: &Map,
    chunk_size: i32,
    include_flipping: bool,
    dedupe: bool,
) -> Vec<Vec<TileType>> {
    let chunks_x = map.width / chunk_size;
    let chunks_y = map.height / chunk_size;
    let mut patterns: Vec<Vec<TileType>> = Vec::new();

    for cy in 0..chunks_y {
        for cx in 0..chunks_x {
            let mut pattern: BuildPattern = BuildPattern::new(Chunk::new(chunk_size, cx, cy));
            pattern.add_tiles(map);
            patterns.push(pattern.pattern());

            if include_flipping {
                patterns.push(pattern.flip_horizontal(map).pattern());
                patterns.push(pattern.flip_vertical(map).pattern());
                patterns.push(pattern.flip_both(map).pattern());
            }

        }
    }

    if dedupe {
        rltk::console::log(format!("Pre-dedupe: {} patterns", patterns.len()));
        let set: HashSet<Vec<TileType>> = patterns.drain(..).collect();
        patterns.extend(set.into_iter());
        rltk::console::log(format!("Post-dedupe: {} patterns", patterns.len()))
    }

    patterns
}

pub fn render_pattern_to_map(map: &mut Map, map_chunk: &MapChunk, chunk: Chunk) {
    // println!("\nNEW PATTERN\n");
    let mut i = 0usize;
    for tile_pos in chunk.into_iter() {
        let idx = map.xy_idx(tile_pos.x + 1, tile_pos.y + 1);
        // println!("({} => {}) tile_pos: ({}, {})", i, idx, tile_pos.x, tile_pos.y);
        map.tiles[idx] = map_chunk.pattern[i];
        map.visible_tiles[idx] = true;
        i += 1;
    }

    for exit_direction in Direction::iterator() {
        for (x, exit) in map_chunk.exits[exit_direction as usize].iter().enumerate() {
            if *exit {
                let map_idx = exit_direction.map_index(map, &chunk, x as i32);
                map.tiles[map_idx] = TileType::DownStairs;
            }
        }
    }
}

pub fn patterns_to_constraints(patterns: Vec<Vec<TileType>>, chunk_size: i32) -> Vec<MapChunk> {
    let mut constraints: Vec<MapChunk> = Vec::new();
    for p in patterns {
        let mut new_chunk = MapChunk {
            pattern: p,
            exits: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            has_exits: true,
            compatible_with: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
        };
        for exit in new_chunk.exits.iter_mut() {
            (0..chunk_size).for_each(|_| (*exit).push(false));
        }

        let mut n_exits = 0;
        for x in 0..chunk_size {
            for (i, tile_idx) in Direction::get_indices(chunk_size, x).into_iter().enumerate() {
                if new_chunk.pattern[tile_idx] == TileType::Floor {
                    new_chunk.exits[i][x as usize] = true;
                    n_exits += 1;
                }
            }
        }
        new_chunk.has_exits = n_exits == 0;
        constraints.push(new_chunk);
    }

    let cloned_constraints = constraints.clone();
    for constraint in constraints.iter_mut() {
        for (j, potential) in cloned_constraints.iter().enumerate() {
            if !constraint.has_exits || !potential.has_exits {
                for compatible in constraint.compatible_with.iter_mut() {
                    compatible.push(j);
                }
            } else {
                for (direction, exit_list) in Direction::iterator().zip(constraint.exits.iter_mut()) {
                    let opposite = direction.opposite();

                    let mut it_fits = false;
                    let mut has_any = false;
                    for (slot, can_enter) in exit_list.iter().enumerate() {
                        if *can_enter {
                            has_any = true;
                            if potential.exits[opposite as usize][slot] {
                                it_fits = true;
                            }
                        }
                    }

                    if it_fits { constraint.compatible_with[direction as usize].push(j) }

                    if !has_any {
                        constraint.compatible_with.iter_mut().for_each(|c| c.push(j));
                    }
                }
            }
        }
    }

    constraints
}

#[derive(Clone, Copy)]
pub struct Chunk {
    pub size: i32,
    pub start: Position,
    pub end: Position,
}

impl Chunk {
    pub fn new(size: i32, x: i32, y: i32) -> Chunk {
        Chunk {
            size,
            start: Position { x: x * size, y: y * size },
            end: Position { x: (x + 1) * size, y: (y + 1) * size },
        }
    }

    pub fn presized(size: i32, start: Position) -> Chunk {
        Chunk {
            size,
            start,
            end: Position { x: start.x + size, y: start.y + size },
        }
    }
}

impl IntoIterator for Chunk {
    type Item = Position;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        (self.start.y..self.end.y)
            .flat_map(|y| std::iter::repeat(y).zip(self.start.x..self.end.x))
            .map(|(y, x)| Position { x, y })
            .collect::<Vec<Position>>()
            .into_iter()
    }
}

struct BuildPattern {
    tiles: Vec<TileType>,
    chunk: Chunk,
}

impl BuildPattern {
    fn new(chunk: Chunk) -> BuildPattern {
        BuildPattern {
            tiles: Vec::new(),
            chunk
        }
    }

    fn pattern(&self) -> Vec<TileType> {
        self.tiles.clone()
    }

    fn add_tile(&mut self, tile: TileType) {
        self.tiles.push(tile);
    }

    fn add_tiles(&mut self, map: &Map) {
        for pos in self.chunk.into_iter() {
            let idx = map.xy_idx(pos.x, pos.y);
            self.add_tile(map.tiles[idx]);
        }
    }

    fn flip_horizontal(&self, map: &Map) -> BuildPattern {
        let mut flipped = BuildPattern::new(self.chunk);
        for pos in flipped.chunk.into_iter() {
            let idx = map.xy_idx(self.chunk.end.x - (pos.x + 1), pos.y);
            flipped.add_tile(map.tiles[idx]);
        }
        flipped
    }

    fn flip_vertical(&self, map: &Map) -> BuildPattern {
        let mut flipped = BuildPattern::new(self.chunk);
        for pos in flipped.chunk.into_iter() {
            let idx = map.xy_idx(pos.x, self.chunk.end.y - (pos.y + 1));
            flipped.add_tile(map.tiles[idx]);
        }
        flipped
    }

    pub fn flip_both(&self, map: &Map) -> BuildPattern {
        let mut flipped = BuildPattern::new(self.chunk);
        for pos in flipped.chunk.into_iter() {
            let idx = map.xy_idx(self.chunk.end.x - (pos.x + 1), self.chunk.end.y - (pos.y + 1));
            flipped.add_tile(map.tiles[idx]);
        }
        flipped
    }
}
