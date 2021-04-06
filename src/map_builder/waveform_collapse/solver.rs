use std::{
    borrow::{Borrow, BorrowMut},
    collections::HashSet,
};

use crate::Map;

use super::common::{Direction, MapChunk};

pub struct Solver {
    constraints: Vec<MapChunk>,
    chunk_size: i32,
    chunks: Vec<Option<usize>>,
    chunks_x: usize,
    chunks_y: usize,
    remaining: Vec<(usize, i32)>,
    pub possible: bool,
}

impl Solver {
    pub fn new(constraints: Vec<MapChunk>, chunk_size: i32, map: &Map) -> Solver {
        let chunks_x = (map.width / chunk_size) as usize;
        let chunks_y = (map.height / chunk_size) as usize;
        let mut remaining: Vec<(usize, i32)> = Vec::new();
        (0..(chunks_x * chunks_y)).for_each(|i| remaining.push((i, 0)));

        Solver {
            constraints,
            chunk_size,
            chunks: vec![None; chunks_x * chunks_y],
            chunks_x,
            chunks_y,
            remaining,
            possible: true,
        }
    }

    pub fn iteration(&mut self, map: &mut Map, rng: &mut rltk::RandomNumberGenerator) -> bool {
        if self.remaining.is_empty() {
            return true;
        }

        let mut neighbors_exist = false;
        let mut remaining_copy = self.remaining.clone();
        for r in remaining_copy.iter_mut() {
            let idx = r.0;
            let chunk_x = idx % self.chunks_x;
            let chunk_y = idx / self.chunks_x;
            let neighbor_count = self.count_neighbors(chunk_x, chunk_y, None);
            if neighbor_count > 0 {
                neighbors_exist = true;
            }
            *r = (r.0, neighbor_count);
        }

        remaining_copy.sort_by(|a, b| b.1.cmp(&a.1));
        self.remaining = remaining_copy;

        let r_idx = if !neighbors_exist {
            (rng.roll_dice(1, self.remaining.len() as i32) - 1) as usize
        } else {
            0usize
        };

        let chunk_idx = self.remaining.remove(r_idx).0;
        let chunk_x = chunk_idx % self.chunks_x;
        let chunk_y = chunk_idx / self.chunks_x;
        let mut options: Vec<Vec<usize>> = Vec::new();
        let neighbors = self.count_neighbors(chunk_x, chunk_y, Some(&mut options));

        if neighbors == 0 {
            let new_chunk_idx = (rng.roll_dice(1, self.constraints.len() as i32) - 1) as usize;
            self.chunks[chunk_idx] = Some(new_chunk_idx);
            self.apply_constraints_to_map(map, chunk_x, chunk_y, new_chunk_idx);
        } else {
            let mut to_check: HashSet<usize> = HashSet::new();
            for option in options.iter() {
                option.iter().for_each(|i| {
                    to_check.insert(*i);
                });
            }

            let mut possible_options: Vec<usize> = Vec::new();
            for new_chunk_idx in to_check.iter() {
                let mut possible = true;
                for option in options.iter() {
                    if !option.contains(new_chunk_idx) {
                        possible = false;
                    }
                }
                if possible {
                    possible_options.push(*new_chunk_idx);
                }
            }

            if possible_options.is_empty() {
                rltk::console::log("Impossible!");
                self.possible = false;
                return true;
            } else {
                let new_chunk_idx = match possible_options.len() {
                    1 => 0,
                    _ => rng.roll_dice(1, possible_options.len() as i32) - 1,
                } as usize;
                self.chunks[chunk_idx] = Some(possible_options[new_chunk_idx]);
                self.apply_constraints_to_map(map, chunk_x, chunk_y, possible_options[new_chunk_idx]);
            }
        }

        false
    }

    fn chunk_idx(&self, x: usize, y: usize) -> usize {
        ((y * self.chunks_x) + x) as usize
    }

    fn count_neighbors(
        &self,
        chunk_x: usize,
        chunk_y: usize,
        mut options: Option<&mut Vec<Vec<usize>>>,
    ) -> i32 {
        let mut neighbors = 0;

        if chunk_x > 0 {
            let left = self.chunk_idx(chunk_x - 1, chunk_y);
            if let Some(l) = self.chunks[left] {
                neighbors += 1;
                if let Some(opt) = options.borrow_mut() {
                    opt.push(self.constraints[l].compatible_with[Direction::East as usize].clone());
                }
            }
        }

        if chunk_x < self.chunks_x - 1 {
            let right = self.chunk_idx(chunk_x + 1, chunk_y);
            if let Some(r) = self.chunks[right] {
                neighbors += 1;
                if let Some(opt) = options.borrow_mut() {
                    opt.push(self.constraints[r].compatible_with[Direction::West as usize].clone());
                }
            }
        }

        if chunk_y > 0 {
            let up = self.chunk_idx(chunk_x, chunk_y - 1);
            if let Some(u) = self.chunks[up] {
                neighbors += 1;
                if let Some(opt) = options.borrow_mut() {
                    opt.push(
                        self.constraints[u].compatible_with[Direction::South as usize].clone(),
                    );
                }
            }
        }

        if chunk_y < self.chunks_y - 1 {
            let down = self.chunk_idx(chunk_x, chunk_y + 1);
            if let Some(d) = self.chunks[down] {
                neighbors += 1;
                if let Some(opt) = options.borrow_mut() {
                    opt.push(
                        self.constraints[d].compatible_with[Direction::North as usize].clone(),
                    );
                }
            }
        }

        neighbors
    }

    fn apply_constraints_to_map(
        &self,
        map: &mut Map,
        chunk_x: usize,
        chunk_y: usize,
        new_chunk_idx: usize,
    ) {
        let lx = chunk_x as i32 * self.chunk_size;
        let rx = (chunk_x + 1) as i32 * self.chunk_size;
        let ty = chunk_y as i32 * self.chunk_size;
        let by = (chunk_y + 1) as i32 * self.chunk_size;

        let mut i: usize = 0;
        for y in ty..by {
            for x in lx..rx {
                let map_idx = map.xy_idx(x, y);
                map.tiles[map_idx] = self.constraints[new_chunk_idx].pattern[i];
                i += 1;
            }
        }
    }
}
