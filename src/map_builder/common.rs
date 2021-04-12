use crate::{BuildData, MetaMapBuilder, Position};

use super::{Map, Rect, TileType};
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use rltk::RandomNumberGenerator;
use std::{
    cmp::{max, min},
    collections::HashMap,
    iter,
};

/// Implemented distance algorithm function definitions.
#[derive(PartialEq, Clone, Copy)]
pub enum DistanceAlgorithm {
    Pythagoras,
    Manhattan,
    Chebyshev,
}

impl DistanceAlgorithm {
    /// Returns the [`rltk::DistanceAlg`] function indicated by the specified
    /// enum variant.
    pub fn get_func(&self) -> fn(rltk::Point, rltk::Point) -> f32 {
        match *self {
            DistanceAlgorithm::Pythagoras => |start: rltk::Point, end: rltk::Point| {
                rltk::DistanceAlg::PythagorasSquared.distance2d(start, end)
            },
            DistanceAlgorithm::Manhattan => |start: rltk::Point, end: rltk::Point| {
                rltk::DistanceAlg::Manhattan.distance2d(start, end)
            },
            DistanceAlgorithm::Chebyshev => |start: rltk::Point, end: rltk::Point| {
                rltk::DistanceAlg::Chebyshev.distance2d(start, end)
            },
        }
    }

    pub fn apply(&self, p1: rltk::Point, p2: rltk::Point) -> f32 {
        self.get_func()(p1, p2)
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum Symmetry {
    None,
    Horizontal,
    Vertical,
    Both,
}

/// Places a rectangular room onto the [`Map`] by setting all tiles within its
/// boundaries to [`TileType::Floor`] tiles.
pub fn apply_room_to_map(map: &mut Map, room: &Rect) {
    (room.y1 + 1..=room.y2)
        .map(|y| iter::repeat(y).zip(room.x1 + 1..=room.x2))
        .flatten()
        .for_each(|(y, x)| {
            let idx = map.xy_idx(x, y);
            map.tiles[idx] = TileType::Floor
        });
}

/// Places a horizontal tunnel between two coordinates on the same `y` level.
pub fn apply_horizontal_tunnel(map: &mut Map, x1: i32, x2: i32, y: i32) {
    (min(x1, x2)..=max(x1, x2)).for_each(|x| {
        let idx = map.xy_idx(x, y);
        if idx > 0 && idx < map.width as usize * map.height as usize {
            map.tiles[idx as usize] = TileType::Floor;
        }
    });
}

/// Places a vertical tunnel between two points on the same `x` level.
pub fn apply_vertical_tunnel(map: &mut Map, y1: i32, y2: i32, x: i32) {
    (min(y1, y2)..=max(y1, y2)).for_each(|y| {
        let idx = map.xy_idx(x, y);
        if idx > 0 && idx < map.width as usize * map.height as usize {
            map.tiles[idx as usize] = TileType::Floor;
        }
    })
}

pub fn draw_corridor(map: &mut Map, x1: i32, y1: i32, x2: i32, y2: i32) {
    let mut x = x1;
    let mut y = y1;

    while x != x2 || y != y2 {
        if x < x2 {
            x += 1;
        } else if x > x2 {
            x -= 1;
        } else if y < y2 {
            y += 1;
        } else if y > y2 {
            y -= 1;
        }
        let idx = map.xy_idx(x, y);
        map.tiles[idx] = TileType::Floor;
    }
}

/// Removes areas from the map that are unreachable from the starting position.
///
/// Uses Dijkstra's algorithm to both calculate the reachable distance and reachability.
/// At most, this will check up to 200 tiles away from its `start_idx`.
pub struct CullUnreachable {}

impl MetaMapBuilder for CullUnreachable {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuildData) {
        self.build(rng, build_data);
    }
}

impl CullUnreachable {
    pub fn new() -> Box<CullUnreachable> {
        Box::new(CullUnreachable {})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuildData) {
        let start = build_data.start.as_ref().unwrap().clone();
        let start_idx = build_data.map.xy_idx(start.x, start.y);
        build_data.map.populate_blocked();

        let map_starts: Vec<usize> = vec![start_idx];
        let dijkstra = rltk::DijkstraMap::new(
            build_data.map.width as usize,
            build_data.map.height as usize,
            &map_starts,
            &build_data.map,
            1000.0,
        );
        for (i, tile) in build_data.map.tiles.iter_mut().enumerate() {
            if *tile == TileType::Floor && dijkstra.map[i] == std::f32::MAX {
                *tile = TileType::Wall;
            }
        }
    }
}

/// Returns the furthest reachable point on the map from its starting position.
///
/// Uses Dijkstra's algorithm to both calculate the reachable distance and reachability.
/// At most, this will check up to 200 tiles away from its `start_idx`.
pub struct DistantExit {}

impl MetaMapBuilder for DistantExit {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuildData) {
        self.build(rng, build_data);
    }
}

impl DistantExit {
    pub fn new() -> Box<DistantExit> {
        Box::new(DistantExit {})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuildData) {
        let start = build_data.start.as_ref().unwrap().clone();
        let start_idx = build_data.map.xy_idx(start.x, start.y);
        build_data.map.populate_blocked();

        let map_starts: Vec<usize> = vec![start_idx];
        let dijkstra = rltk::DijkstraMap::new(
            build_data.map.width as usize,
            build_data.map.height as usize,
            &map_starts,
            &build_data.map,
            1000.0,
        );
        let mut exit_tile = (0, 0.0f32);

        for (i, tile) in build_data.map.tiles.iter_mut().enumerate() {
            let dist_to_start = dijkstra.map[i];
            if *tile == TileType::Floor
                && dist_to_start != std::f32::MAX
                && dist_to_start > exit_tile.1
            {
                exit_tile = (i, dist_to_start);
            }
        }

        let stairs = exit_tile.0;
        build_data.map.tiles[stairs] = TileType::DownStairs;
        build_data.take_snapshot();
    }
}

pub trait Digger {
    fn get_position(&self) -> (i32, i32);
    fn get_position_mut(&mut self) -> (&mut i32, &mut i32);
    fn set_position(&mut self, x: i32, y: i32);
    fn stagger(&mut self, map: &mut Map, rng: &mut rltk::RandomNumberGenerator) -> (i32, i32);

    /// Randomly generates the digger's new position, and moves them to it.
    /// Moves one tile (at most) in one of the four cardinal directions.
    fn stagger_direction(&mut self, map: &Map, rng: &mut rltk::RandomNumberGenerator) {
        let (x, y): (&mut i32, &mut i32) = self.get_position_mut();
        // Roll dice to pick a direction to move, then update the digger's
        // position based on said roll. If movement would take the digger
        // outside the map bounds, do nothing instead.
        match rng.roll_dice(1, 4) {
            1 => {
                if *x > 2 {
                    *x -= 1
                }
            }
            2 => {
                if *x < map.width - 2 {
                    *x += 1;
                }
            }
            3 => {
                if *y > 2 {
                    *y -= 1;
                }
            }
            _ => {
                if *y < map.height - 2 {
                    *y += 1;
                }
            }
        };
    }
}

impl Distribution<Symmetry> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Symmetry {
        match rng.gen_range(0..=3) {
            0 => Symmetry::None,
            1 => Symmetry::Horizontal,
            2 => Symmetry::Vertical,
            _ => Symmetry::Both,
        }
    }
}

pub fn paint(map: &mut Map, mode: Symmetry, brush_size: i32, x: i32, y: i32) {
    let center = Position::from(map.center());
    let idx = map.xy_idx(x, y);
    map.tiles[idx] = TileType::Floor;

    // Match on symmetry type
    match mode {
        // No symmetry--just paint
        Symmetry::None => apply_paint(map, brush_size, x, y),
        Symmetry::Horizontal => {
            if x == center.x {
                // If on the tile, paint it
                apply_paint(map, brush_size, x, y);
            } else {
                // Else, apply paint symmetrically in the x-direction
                // based on distance from it
                let d_x = i32::abs(center.x - x);
                apply_paint(map, brush_size, center.x + d_x, y);
                apply_paint(map, brush_size, center.x - d_x, y);
            }
        }
        Symmetry::Vertical => {
            if y == center.y {
                // If on the tile, paint it
                apply_paint(map, brush_size, x, y);
            } else {
                // Else, apply paint symmetrically in the y-direction
                // based on distance from it
                let d_y = i32::abs(center.y - y);
                apply_paint(map, brush_size, x, center.y + d_y);
                apply_paint(map, brush_size, x, center.y + d_y);
            }
        }
        Symmetry::Both => {
            // Break center down into parts to appease the borrow checker
            let (center_x, center_y) = center.into();
            if (x, y) == (center_x, center_y) {
                // If on the tile, paint it
                apply_paint(map, brush_size, x, y);
            } else {
                // Apply symmetric paint horizontally about the tile
                let d_x = i32::abs(center_x - x);
                apply_paint(map, brush_size, center_x + d_x, y);
                apply_paint(map, brush_size, center_x - d_x, y);
                // Apply symmetric paint vertically about the tile
                let d_y = i32::abs(center_y - y);
                apply_paint(map, brush_size, x, center_y + d_y);
                apply_paint(map, brush_size, x, center_y - d_y);
            }
        }
    }
}

/// Applies paint to a tile based on brush size.
fn apply_paint(map: &mut Map, brush_size: i32, x: i32, y: i32) {
    if brush_size == 1 {
        // Single-tile brush--paint just that floor tile
        let idx = map.xy_idx(x, y);
        map.tiles[idx] = TileType::Floor;
    } else {
        // Else, loop through brush size
        let half_brush = brush_size / 2;
        for brush_y in y - half_brush..y + half_brush {
            for brush_x in x - half_brush..x + half_brush {
                // Make sure the `half_brush` index is in bounds
                if map.in_bounds(brush_x, 0, brush_y, 0) {
                    // Paint at each `half_brush` index
                    let idx = map.xy_idx(brush_x, brush_y);
                    map.tiles[idx] = TileType::Floor;
                }
            }
        }
    }
}
