use rltk::rex::XpFile;

use crate::{Map, TileType};

rltk::embedded_resource!(SMALL_DUNGEON, "../resources/SmallDungeon_80x50.xp");
rltk::embedded_resource!(WFC_DEMO_IMAGE1, "../resources/wfc-demo1.xp");
rltk::embedded_resource!(WFC_DEMO_IMAGE2, "../resources/wfc-demo2.xp");
rltk::embedded_resource!(WFC_POPULATED, "../resources/wfc-populated.xp");

pub struct RexAssets {
    pub menu: XpFile,
}

impl RexAssets {
    #[allow(clippy::clippy::new_without_default)]
    pub fn new() -> RexAssets {
        rltk::link_resource!(SMALL_DUNGEON, "../resources/SmallDungeon_80x50.xp");
        rltk::link_resource!(WFC_DEMO_IMAGE1, "../resources/wfc-demo1.xp");
        rltk::link_resource!(WFC_DEMO_IMAGE2, "../resources/wfc-demo2.xp");
        rltk::link_resource!(WFC_POPULATED, "../resources/wfc-populated.xp");

        RexAssets {
            menu: XpFile::from_resource("../resources/SmallDungeon_80x50.xp").unwrap(),
        }
    }
}

#[allow(dead_code)]
pub fn load_rex_map(map: &mut Map, path: &str) -> Vec<(usize, String)> {
    let spawns: Vec<(usize, String)> = Vec::new();
    let xp_file = XpFile::from_resource(path).unwrap();

    for layer in &xp_file.layers {
        for y in 0..layer.height {
            for x in 0..layer.width {
                let cell = layer.get(x, y).unwrap();
                if x < map.width as usize && y < map.height as usize {
                    let idx = map.xy_idx(x as i32, y as i32);
                    match (cell.ch as u8) as char {
                        ' ' => map.tiles[idx] = TileType::Floor,
                        '#' => map.tiles[idx] = TileType::Wall,
                        '@' => {
                            map.tiles[idx] = TileType::Floor;
                        }
                        '>' => self.map.tiles[idx] = TileType::DownStairs,
                        'g' => {
                            map.tiles[idx] = TileType::Floor;
                            spawns.push((idx, "Goblin".to_string()));
                        }
                        'o' => {
                            map.tiles[idx] = TileType::Floor;
                            spawns.push((idx, "Orc".to_string()));
                        }
                        '^' => {
                            map.tiles[idx] = TileType::Floor;
                            spawns.push((idx, "Bear Trap".to_string()));
                        }
                        '%' => {
                            map.tiles[idx] = TileType::Floor;
                            spawns.push((idx, "Rations".to_string()));
                        }
                        '!' => {
                            map.tiles[idx] = TileType::Floor;
                            spawns.push((idx, "Health Potion".to_string()));
                        }
                        _ => rltk::console::log(format!(
                            "Unknown glyph when loading map: {}",
                            (cell.ch as u8) as char
                        )),
                    }
                }
            }
        }
    }

    spawns
}
