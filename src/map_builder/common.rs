use super::{Map, Rect, TileType};
use std::{
    cmp::{max, min},
    collections::HashMap,
    iter,
};

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

/// Removes areas from the map that are unreachable from the starting position
/// and returns the furthest reachable point on the map from said starting position.
///
/// Uses Dijkstra's algorithm to both calculate the reachable distance and reachability.
/// At most, this will check up to 200 tiles away from its `start_idx`.
pub fn remove_unreachable_areas_returning_most_distant(map: &mut Map, start_idx: usize) -> usize {
    map.populate_blocked();
    let map_starts: Vec<usize> = vec![start_idx];
    // Make a `DijkstraMap` matching the map dimensions that runs for a max of 200 steps.
    let dijkstra = rltk::DijkstraMap::new(
        map.width as usize,
        map.height as usize,
        &map_starts,
        map,
        200.0,
    );
    // `exit_tile` holds the tile index and distance to the proposed exit.
    let mut exit_tile = (0, 0.0f32);

    // Enumerate map tiles looking for an exit.
    map.tiles.iter_mut().enumerate().for_each(|(i, tile)| {
        // Found a floor tile--get the distance between it and the start.
        if *tile == TileType::Floor {
            let dist_to_start = dijkstra.map[i];
            if dist_to_start == std::f32::MAX {
                // Unreachable, so might as well make it a wall tile.
                *tile = TileType::Wall;
            } else {
                // If the new candidate tile is reachable and further that our previous
                // best exit candidate, set it's index and distance to the new best.
                if dist_to_start > exit_tile.1 {
                    exit_tile = (i, dist_to_start);
                }
            }
        }
    });

    // Visited all tiles--return DownStairs location.
    exit_tile.0
}

/// Generates valid noise areas on the [`Map`] from where we may generate noise.
#[allow(clippy::map_entry)]
pub fn generate_voronoi_spawn_regions(
    map: &Map,
    rng: &mut rltk::RandomNumberGenerator,
) -> HashMap<i32, Vec<usize>> {
    // Make a new noise generator to generate Cellular (Voronoi) noise.
    let mut noise_areas: HashMap<i32, Vec<usize>> = HashMap::new();
    let mut noise = rltk::FastNoise::seeded(rng.roll_dice(1, 65536) as u64);
    noise.set_noise_type(rltk::NoiseType::Cellular);
    // 0.08 is arbitrary, but seems to work nice.
    noise.set_frequency(0.08);
    // Uses L1 distance to favor enlongated shapes.
    noise.set_cellular_distance_function(rltk::CellularDistanceFunction::Manhattan);

    // Iterate the map
    map.iter_xy().iter().for_each(|(x, y)| {
        let idx = map.xy_idx(*x, *y);
        // Exclude wall tiles, focus on floors.
        if map.tiles[idx] == TileType::Floor {
            // Query for a noise value for the current coordinates, and scale to make useful.
            let cell_value = (noise.get_noise(*x as f32, *y as f32) * 10240.0) as i32;
            if noise_areas.contains_key(&cell_value) {
                noise_areas.get_mut(&cell_value).unwrap().push(idx);
            } else {
                noise_areas.insert(cell_value, vec![idx]);
            }
        }
    });

    noise_areas
}

/// Digger that staggers around the map, creating open areas.
pub struct DrunkDigger<'a> {
    // Digger's current x position
    pub x: i32,
    // Diggers current y position
    pub y: i32,
    // If the digger has actually dug any wall tiles out
    pub did_something: bool,
    // How long the drunk will stagger for
    // pub life: i32,
    rng: &'a mut rltk::RandomNumberGenerator,
    // Current (x, y) in the map index (1D array indexing from 2D index)
    idx: usize,
}

impl<'a> DrunkDigger<'a> {
    /// Creates and returns a new [`DrunkDigger`].
    pub fn new(x: i32, y: i32, rng: &mut rltk::RandomNumberGenerator) -> DrunkDigger {
        DrunkDigger {
            x: x,
            y: y,
            did_something: false,
            // life: life,
            rng: rng,
            idx: usize::default(),
        }
    }

    /// Moves the drunk around the map one tile at a time in a random direction
    /// until they've run out of life.
    ///
    /// Uses [`TileType::DownStairs`] as a marker to differentiate tiles dug by the
    /// digger (the [`TileType::DownStairs`] tiles) from tiles that were already
    /// floor tiles; keeps us from having to add another TileType enum variant,
    /// which could possibly break exhaustion on TileType match statements.
    /// These will be turned into floor tiles during the `build` loop.
    pub fn stagger_lifetime(&mut self, map: &mut Map, life: i32) {
        let mut life = life;
        while life > 0 {
            self.idx = map.xy_idx(self.x, self.y);
            // If they've landed on a wall tile, dig it out
            if map.tiles[self.idx] == TileType::Wall {
                self.did_something = true;
            }
            // Mark the tiles dug by the digger
            map.tiles[self.idx] = TileType::DownStairs;
            // Get its position for the next iteration and reduce its remaining life
            self.stagger_direction(map);
            life -= 1;
        }
    }

    pub fn stagger_tiles(&mut self, map: &mut Map, tile_type: TileType) -> (i32, i32) {
        let mut prev_pos: (i32, i32) = (self.x, self.y);
        while map.tiles[self.idx] == tile_type {
            prev_pos = (self.x, self.y);
            self.stagger_direction(map);
            self.idx = map.xy_idx(self.x, self.y);
        }
        prev_pos
    }

    /// Randomly generates the digger's new position, and moves them to it.
    /// Moves one tile (at most) in one of the four cardinal directions.
    fn stagger_direction(&mut self, map: &Map) {
        // Roll dice to pick a direction to move, then update the digger's
        // position based on said roll. If movement would take the digger
        // outside the map bounds, do nothing instead.
        match self.rng.roll_dice(1, 4) {
            1 => {
                if self.x > 2 {
                    self.x -= 1
                }
            }
            2 => {
                if self.x < map.width - 2 {
                    self.x += 1;
                }
            }
            3 => {
                if self.y > 2 {
                    self.y -= 1;
                }
            }
            _ => {
                if self.y < map.height - 2 {
                    self.y += 1;
                }
            }
        };
    }
}
