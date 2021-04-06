use crate::{Map, TileType};

pub fn load_rex_map(new_depth: i32, xp_file: &rltk::XpFile) -> Map {
    let mut map: Map = Map::new(new_depth);

    for layer in &xp_file.layers {
        for y in 0..layer.height {
            for x in 0..layer.width {
                if x < map.width as usize && y < map.height as usize {
                    let idx = map.xy_idx(x as i32, y as i32);
                    match layer.get(x, y).unwrap().ch {
                        32 => map.tiles[idx] = TileType::Floor,
                        35 => map.tiles[idx] = TileType::Wall,
                        _ => {}
                    }
                }
            }
        }
    }

    map
}
