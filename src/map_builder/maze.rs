use std::collections::HashMap;

use rltk::RandomNumberGenerator;
use specs::World;

use crate::{BuildData, InitialMapBuilder, Map, MapBuilder, Position, SHOW_MAPGEN_VISUALIZER, TileType, spawner};

use super::common::{
    generate_voronoi_spawn_regions, remove_unreachable_areas_returning_most_distant,
};

const TOP: usize = 0;
const RIGHT: usize = 1;
const BOTTOM: usize = 2;
const LEFT: usize = 3;

pub struct MazeBuilder {}

impl InitialMapBuilder for MazeBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut crate::BuildData) {
        self.build(rng, build_data);
    }
}

impl MazeBuilder {
    pub fn new() -> Box<MazeBuilder> {
        Box::new(MazeBuilder {})
    }

    #[allow(clippy::map_entry)]
    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuildData) {
        let mut maze = Grid::new(
            (build_data.map.width / 2) - 2,
            (build_data.map.height / 2) - 2,
            rng
        );
        maze.generate_maze(build_data);
    }
}

/// Single Cell on a [`Grid`].
#[derive(Clone, Copy)]
struct Cell {
    row: i32,
    col: i32,
    walls: [bool; 4],
    visited: bool,
}

impl Cell {
    /// Make a new, unvisited cell with walls in each direction.
    fn new(row: i32, col: i32) -> Cell {
        Cell {
            row,
            col,
            walls: [true, true, true, true],
            visited: false,
        }
    }

    fn remove_walls(&mut self, next: &mut Cell) {
        let x = self.col - next.col;
        let y = self.row - next.row;

        if x == 1 {
            self.walls[LEFT] = false;
            next.walls[RIGHT] = false;
        } else if x == -1 {
            self.walls[RIGHT] = false;
            next.walls[LEFT] = false;
        } else if y == 1 {
            self.walls[TOP] = false;
            next.walls[BOTTOM] = false;
        } else if y == -1 {
            self.walls[BOTTOM] = false;
            next.walls[TOP] = false;
        }
    }
}

struct Grid<'a> {
    width: i32,
    height: i32,
    cells: Vec<Cell>,
    backtrace: Vec<usize>,
    current: usize,
    rng: &'a mut RandomNumberGenerator,
}

impl<'a> Grid<'a> {
    fn new(width: i32, height: i32, rng: &mut RandomNumberGenerator) -> Grid {
        let mut grid = Grid {
            width,
            height,
            cells: Vec::new(),
            backtrace: Vec::new(),
            current: 0,
            rng,
        };

        for row in 0..height {
            for col in 0..width {
                grid.cells.push(Cell::new(row, col));
            }
        }

        grid
    }

    fn calculate_index(&self, row: i32, col: i32) -> i32 {
        if row < 0 || col < 0 || col > self.width - 1 || row > self.height - 1 {
            -1
        } else {
            col + (row * self.width)
        }
    }

    fn get_available_neighbors(&self) -> Vec<usize> {
        let mut neighbors: Vec<usize> = Vec::new();
        let current_row = self.cells[self.current].row;
        let current_col = self.cells[self.current].col;

        let neighbor_indices: [i32; 4] = [
            self.calculate_index(current_row - 1, current_col),
            self.calculate_index(current_row, current_col + 1),
            self.calculate_index(current_row + 1, current_col),
            self.calculate_index(current_row, current_col - 1),
        ];

        for i in neighbor_indices.iter() {
            if *i != -1 && !self.cells[*i as usize].visited {
                neighbors.push(*i as usize);
            }
        }

        neighbors
    }

    fn find_next_cell(&mut self) -> Option<usize> {
        let neighbors = self.get_available_neighbors();
        if !neighbors.is_empty() {
            if neighbors.len() == 1 {
                return Some(neighbors[0]);
            } else {
                return Some(
                    neighbors[(self.rng.roll_dice(1, neighbors.len() as i32) - 1) as usize],
                );
            }
        }
        None
    }

    fn generate_maze(&mut self, build_data: &mut BuildData) {
        let mut i = 0;
        loop {
            // Mark current cell as visited and get the next cell
            self.cells[self.current].visited = true;
            let next = self.find_next_cell();

            match next {
                Some(next) => {
                    // Mark next as visited and push the current onto the backtrace stack
                    self.cells[next].visited = true;
                    self.backtrace.push(self.current);

                    let (lower_part, higher_part) =
                        self.cells.split_at_mut(std::cmp::max(self.current, next));
                    let cell1 = &mut lower_part[std::cmp::min(self.current, next)];
                    let cell2 = &mut higher_part[0];
                    cell1.remove_walls(cell2);
                    self.current = next;
                }
                None => {
                    if !self.backtrace.is_empty() {
                        self.current = self.backtrace[0];
                        self.backtrace.remove(0);
                    } else {
                        break;
                    }
                }
            }

            if i % 50 == 0 {
                self.copy_to_map(&mut build_data.map);
                build_data.take_snapshot();
            }
            i += 1;
        }
    }

    fn copy_to_map(&self, map: &mut Map) {
        map.tiles.iter_mut().for_each(|i| *i = TileType::Wall);

        for cell in self.cells.iter() {
            let x = cell.col + 1;
            let y = cell.row + 1;
            let idx = map.xy_idx(x * 2, y * 2);

            map.tiles[idx] = TileType::Floor;
            if !cell.walls[TOP] {
                map.tiles[idx - map.width as usize] = TileType::Floor
            }
            if !cell.walls[RIGHT] {
                map.tiles[idx + 1] = TileType::Floor
            }
            if !cell.walls[BOTTOM] {
                map.tiles[idx + map.width as usize] = TileType::Floor
            }
            if !cell.walls[LEFT] {
                map.tiles[idx - 1] = TileType::Floor
            }
        }
    }
}
