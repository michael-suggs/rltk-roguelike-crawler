use crate::{Map, Position, TileType};

pub fn build_patterns(
    map: &Map,
    chunk_size: i32,
    include_flipping: bool,
    dedupe: bool,
) -> Vec<BuildPattern> {
    let chunks_x = map.width / chunk_size;
    let chunks_y = map.height / chunk_size;
    let mut patterns: Vec<BuildPattern> = Vec::new();

    for cy in 0..chunks_y {
        for cx in 0..chunks_x {
            let mut pattern: BuildPattern = BuildPattern::new(
                Position { x: cx * chunk_size, y: cy * chunk_size },
                Position { x: (cx + 1) * chunk_size, y: (cy + 1) * chunk_size }
            );

            for y in pattern.start.y..pattern.end.y {
                for x in pattern.start.x..pattern.end.x {
                    let idx = map.xy_idx(x, y);
                    pattern.add_tile(map.tiles[idx]);
                }
            }

            patterns.push(pattern);
        }
    }

    patterns
}

pub struct BuildPattern {
    tiles: Vec<TileType>,
    start: Position,
    end: Position,
}

impl BuildPattern {
    pub fn new(start: Position, end: Position) -> BuildPattern {
        BuildPattern {
            tiles: Vec::new(),
            start,
            end
        }
    }

    pub fn add_tile(&mut self, tile: TileType) {
        self.tiles.push(tile);
    }

    pub fn flip_vertically(&mut self) -> BuildPattern {
        let flipped = BuildPattern::new(self.start, self.end);
        for y in self.start.y..self.end.y {
            for x in self.start.x..self.end.x {
                let idx = self.xy_idx(x, y);
                flipped.add_tile(tile)
            }
        }

        flipped
    }

    fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * crate::MAPWIDTH as usize) + x as usize
    }
}
