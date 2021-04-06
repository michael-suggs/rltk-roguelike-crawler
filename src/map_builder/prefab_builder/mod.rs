use crate::{spawner, Map, MapBuilder, Position, TileType, SHOW_MAPGEN_VISUALIZER};

use super::common::remove_unreachable_areas_returning_most_distant;

mod prefab_levels;

#[allow(dead_code)]
#[derive(PartialEq, Clone)]
pub enum PrefabMode {
    RexLevel { template: &'static str },
    Constant { level: prefab_levels::PrefabLevel },
}

pub struct PrefabBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    mode: PrefabMode,
    spawns: Vec<(usize, String)>,
}

impl MapBuilder for PrefabBuilder {
    fn build_map(&mut self) {
        self.build();
    }

    fn spawn_entities(&mut self, ecs: &mut specs::World) {
        for ent in self.spawns.iter() {
            spawner::spawn_entity(ecs, &(&ent.0, &ent.1));
        }
    }

    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            snapshot.revealed_tiles.iter_mut().for_each(|v| *v = true);
            self.history.push(snapshot);
        }
    }
}

impl PrefabBuilder {
    pub fn new(new_depth: i32) -> PrefabBuilder {
        PrefabBuilder {
            map: Map::new(new_depth),
            starting_position: Position::default(),
            depth: new_depth,
            history: Vec::new(),
            // mode: PrefabMode::RexLevel {
            //     template: "../resources/wfc-populated.xp",
            // },
            mode: PrefabMode::Constant {
                level: prefab_levels::WFC_POPULATED,
            },
            spawns: Vec::new(),
        }
    }

    fn build(&mut self) {
        match self.mode {
            PrefabMode::RexLevel { template } => self.load_rex_map(&template),
            PrefabMode::Constant { level } => self.load_ascii_map(&level),
        }
        self.take_snapshot();

        if self.starting_position.x == 0 {
            self.starting_position = Position::from(self.map.center());

            let mut start_idx = self
                .map
                .xy_idx(self.starting_position.x, self.starting_position.y);

            while self.map.tiles[start_idx] != TileType::Floor {
                self.starting_position.x -= 1;
                start_idx = self
                    .map
                    .xy_idx(self.starting_position.x, self.starting_position.y);
            }
            self.take_snapshot();

            let exit_tile =
                remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);
            self.map.tiles[exit_tile] = TileType::DownStairs;
            self.take_snapshot();
        }
    }

    fn char_to_map(&mut self, ch: char, idx: usize) {
        match ch {
            ' ' => self.map.tiles[idx] = TileType::Floor,
            '#' => self.map.tiles[idx] = TileType::Wall,
            '@' => {
                self.map.tiles[idx] = TileType::Floor;
            }
            '>' => self.map.tiles[idx] = TileType::DownStairs,
            'g' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawns.push((idx, "Goblin".to_string()));
            }
            'o' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawns.push((idx, "Orc".to_string()));
            }
            '^' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawns.push((idx, "Bear Trap".to_string()));
            }
            '%' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawns.push((idx, "Rations".to_string()));
            }
            '!' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawns.push((idx, "Health Potion".to_string()));
            }
            _ => rltk::console::log(format!("Unknown glyph when loading map: {}", ch)),
        }
    }

    fn load_rex_map(&mut self, path: &str) {
        let xp_file = rltk::rex::XpFile::from_resource(path).unwrap();

        for layer in &xp_file.layers {
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let cell = layer.get(x, y).unwrap();
                    if x < self.map.width as usize && y < self.map.height as usize {
                        let idx = self.map.xy_idx(x as i32, y as i32);
                        self.char_to_map(cell.ch as u8 as char, idx);
                    }
                }
            }
        }
    }

    fn load_ascii_map(&mut self, level: &prefab_levels::PrefabLevel) {
        let mut string_vec: Vec<char> = level
            .template
            .chars()
            .filter(|c| *c != '\r' && *c != '\n')
            .collect();
        string_vec.iter_mut().for_each(|c| {
            if *c as u8 == 160u8 {
                *c = ' '
            }
        });

        let mut i = 0;
        for y in 0..level.height {
            for x in 0..level.width {
                if x < self.map.width as usize && y < self.map.height as usize {
                    let idx = self.map.xy_idx(x as i32, y as i32);
                    self.char_to_map(string_vec[i], idx);
                }
                i += 1;
            }
        }
    }
}
