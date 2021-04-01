use std::collections::HashMap;

use rltk::RandomNumberGenerator;
use specs::World;

use crate::{spawner, Map, MapBuilder, Position, TileType, SHOW_MAPGEN_VISUALIZER};

use super::common::{
    generate_voronoi_spawn_regions, remove_unreachable_areas_returning_most_distant,
};

const TOP: usize = 0;
const RIGHT: usize = 1;
const BOTTOM: usize = 2;
const LEFT: usize = 3;

pub struct MazeBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
}

impl MapBuilder for MazeBuilder {
    fn build_map(&mut self) {
        self.build();
    }

    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
        self.noise_areas.iter().for_each(|area| {
            spawner::spawn_region(ecs, area.1, self.depth);
        });
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            snapshot.revealed_tiles.iter_mut().for_each(|v| *v = true);
            self.history.push(snapshot);
        }
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }
}

impl MazeBuilder {
    pub fn new(new_depth: i32) -> MazeBuilder {
        MazeBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
        }
    }

    #[allow(clippy::map_entry)]
    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();
        let (cx, cy) = self.map.center();

        let mut maze = Grid::new(cx - 2, cy - 2, &mut rng);
        maze.generate_maze(self);

        self.starting_position = Position { x: 2, y: 2 };
        let start_idx = self
            .map
            .xy_idx(self.starting_position.x, self.starting_position.y);
        self.take_snapshot();

        let exit_tile = remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);
        self.take_snapshot();

        self.map.tiles[exit_tile] = TileType::DownStairs;
        self.take_snapshot();

        self.noise_areas = generate_voronoi_spawn_regions(&self.map, &mut rng);
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

    fn generate_maze(&mut self, generator: &mut MazeBuilder) {
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
                self.copy_to_map(&mut generator.map);
                generator.take_snapshot();
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
