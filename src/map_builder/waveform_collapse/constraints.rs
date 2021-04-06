use std::collections::HashSet;

use crate::{Map, Position, TileType};

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

pub fn render_pattern_to_map(map: &mut Map, pattern: &Vec<TileType>, chunk: Chunk) {
    // println!("\nNEW PATTERN\n");
    let mut i = 0usize;
    for tile_pos in chunk.into_iter() {
        let idx = map.xy_idx(tile_pos.x, tile_pos.y);
        // println!("({} => {}) tile_pos: ({}, {})", i, idx, tile_pos.x, tile_pos.y);
        map.tiles[idx] = pattern[i];
        map.visible_tiles[idx] = true;
        i += 1;
    }
}

#[derive(Clone, Copy)]
pub struct Chunk {
    size: i32,
    pub start: Position,
    pub end: Position,
}

impl Chunk {
    pub fn new(chunk_size: i32, x: i32, y: i32) -> Chunk {
        Chunk {
            size: chunk_size,
            start: Position { x: (x * chunk_size) + 1, y: (y * chunk_size) + 1 },
            end: Position { x: ((x + 1) * chunk_size) + 1, y: ((y + 1) * chunk_size) + 1 },
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
