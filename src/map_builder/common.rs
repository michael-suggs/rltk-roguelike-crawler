use super::{Map, Rect, TileType};
use std::{
    cmp::{max, min},
    collections::HashMap,
    iter
};

pub fn apply_room_to_map(map: &mut Map, room: &Rect) {
    (room.y1 + 1 ..= room.y2)
        .map(|y| iter::repeat(y).zip(room.x1 + 1 ..= room.x2))
        .flatten()
        .for_each(|(y,x)| {
            let idx = map.xy_idx(x, y);
            map.tiles[idx] = TileType::Floor
        });
}

pub fn apply_horizontal_tunnel(map: &mut Map, x1: i32, x2: i32, y: i32) {
    (min(x1, x2) ..= max(x1, x2))
        .for_each(|x| {
            let idx = map.xy_idx(x, y);
            if idx > 0 && idx < map.width as usize * map.height as usize {
                map.tiles[idx as usize] = TileType::Floor;
            }
        });
}

pub fn apply_vertical_tunnel(map: &mut Map, y1: i32, y2: i32, x: i32) {
    (min(y1, y2) ..= max(y1, y2))
        .for_each(|y| {
            let idx = map.xy_idx(x, y);
            if idx > 0 && idx < map.width as usize * map.height as usize {
                map.tiles[idx as usize] = TileType::Floor;
            }
        })
}

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

#[allow(clippy::map_entry)]
pub fn generate_voronoi_spawn_regions(
    map: &Map, rng: &mut rltk::RandomNumberGenerator
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
