use crate::{spawner, Map, MapBuilder, Position, TileType, SHOW_MAPGEN_VISUALIZER};

use super::common::remove_unreachable_areas_returning_most_distant;

use prefab_rooms::{PrefabRoom, Vault};
use prefab_sections::{HorizontalPlacement, VerticalPlacement};

mod prefab_levels;
mod prefab_rooms;
mod prefab_sections;

#[allow(dead_code)]
#[derive(PartialEq, Clone)]
pub enum PrefabMode {
    RexLevel {
        template: &'static str,
    },
    Constant {
        level: prefab_levels::PrefabLevel,
    },
    Sectional {
        section: prefab_sections::PrefabSection,
    },
    RoomVaults,
}

pub struct PrefabBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    mode: PrefabMode,
    spawn_list: Vec<(usize, String)>,
    previous_builder: Option<Box<dyn MapBuilder>>,
}

impl MapBuilder for PrefabBuilder {
    fn build_map(&mut self) {
        self.build();
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

    fn spawn_entities(&mut self, ecs: &mut specs::World) {
        for ent in self.spawn_list.iter() {
            spawner::spawn_entity(ecs, &(&ent.0, &ent.1));
        }
    }

    fn get_spawn_list(&self) -> &Vec<(usize, String)> {
        &self.spawn_list
    }
}

impl PrefabBuilder {
    pub fn new(new_depth: i32, previous_builder: Option<Box<dyn MapBuilder>>) -> PrefabBuilder {
        PrefabBuilder {
            map: Map::new(new_depth),
            starting_position: Position::default(),
            depth: new_depth,
            history: Vec::new(),
            // mode: PrefabMode::RexLevel {
            //     template: "../resources/wfc-populated.xp",
            // },
            // mode: PrefabMode::Constant {
            //     level: prefab_levels::WFC_POPULATED,
            // },
            mode: PrefabMode::Sectional {
                section: prefab_sections::UNDERGROUND_FORT,
            },
            spawn_list: Vec::new(),
            previous_builder,
        }
    }

    fn build(&mut self) {
        match self.mode {
            PrefabMode::RexLevel { template } => self.load_rex_map(&template),
            PrefabMode::Constant { level } => self.load_ascii_map(&level),
            PrefabMode::Sectional { section } => self.apply_sectional(&section),
            PrefabMode::RoomVaults => self.apply_room_vaults(),
        }
        self.take_snapshot();

        let mut start_idx: usize;
        if self.starting_position.x == 0 {
            self.starting_position = Position::from(self.map.center());
            start_idx = self
                .map
                .xy_idx(self.starting_position.x, self.starting_position.y);

            while self.map.tiles[start_idx] != TileType::Floor {
                self.starting_position.x -= 1;
                start_idx = self
                    .map
                    .xy_idx(self.starting_position.x, self.starting_position.y);
            }
            self.take_snapshot();
        }
        let mut has_exit = false;
        for t in self.map.tiles.iter() {
            if *t == TileType::DownStairs {
                has_exit = true;
            }
        }

        if !has_exit {
            start_idx = self
                .map
                .xy_idx(self.starting_position.x, self.starting_position.y);
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
                self.starting_position = Position {
                    x: idx as i32 % self.map.width,
                    y: idx as i32 / self.map.width,
                };
            }
            '>' => self.map.tiles[idx] = TileType::DownStairs,
            'g' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "Goblin".to_string()));
            }
            'o' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "Orc".to_string()));
            }
            '^' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "Bear Trap".to_string()));
            }
            '%' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "Rations".to_string()));
            }
            '!' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "Health Potion".to_string()));
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

    fn read_ascii_to_vec(template: &str) -> Vec<char> {
        let mut string_vec: Vec<char> = template
            .chars()
            .filter(|c| *c != '\r' && *c != '\n')
            .collect();
        string_vec.iter_mut().for_each(|c| {
            if *c as u8 == 160u8 {
                *c = ' ';
            }
        });
        string_vec
    }

    fn load_ascii_map(&mut self, level: &prefab_levels::PrefabLevel) {
        let string_vec: Vec<char> = PrefabBuilder::read_ascii_to_vec(level.template);
        let mut i = 0;
        for y in 0..level.height {
            for x in 0..level.width {
                if x > 0 && y > 0 && x < self.map.width as usize && y < self.map.height as usize {
                    let idx = self.map.xy_idx(x as i32, y as i32);
                    self.char_to_map(string_vec[i], idx);
                }
                i += 1;
            }
        }
    }

    fn apply_sectional(&mut self, section: &prefab_sections::PrefabSection) {
        let string_vec = PrefabBuilder::read_ascii_to_vec(
            prefab_sections::get_template_str(section.to_owned()).as_str(),
        );
        let chunk_x = match section.placement.0 {
            HorizontalPlacement::Left => 0,
            HorizontalPlacement::Center => (self.map.width / 2) - (section.width as i32 / 2),
            HorizontalPlacement::Right => (self.map.width - 1) - section.width as i32,
        };
        let chunk_y = match section.placement.1 {
            VerticalPlacement::Top => 0,
            VerticalPlacement::Center => (self.map.height / 2) - (section.height as i32 / 2),
            VerticalPlacement::Bottom => (self.map.height - 1) - section.height as i32,
        };

        self.apply_previous_iteration(|x, y, e| {
            x < chunk_x
                || x > (chunk_x + section.width as i32)
                || y < chunk_y
                || y > (chunk_y + section.height as i32)
        });
        self.take_snapshot();
    }

    fn apply_room_vaults(&mut self) {
        let mut rng = rltk::RandomNumberGenerator::new();
        self.apply_previous_iteration(|_,_,_| true);

        let master_vault_list = vec![prefab_rooms::NOT_A_TRAP];
        let possible_vaults: Vec<&PrefabRoom> = master_vault_list
            .iter()
            .filter(|v| self.depth >= v.first_depth && self.depth <= v.last_depth)
            .collect();

        if possible_vaults.is_empty() {
            return;
        }

        let vidx = match possible_vaults.len() {
            1 => 0,
            _ => (rng.roll_dice(1, possible_vaults.len() as i32) - 1) as usize,
        };
        let vault = possible_vaults[vidx];
        let mut vault_positions: Vec<Position> = Vec::new();

        let mut i = 0usize;
        while i < (self.map.tiles.len() - 1) {
            let x = (i % self.map.width as usize) as i32;
            let y = (i / self.map.width as usize) as i32;

            if x > 1
                && y > 1
                && (x + vault.width as i32) < self.map.width - 2
                && (y + vault.height as i32) < self.map.height - 2
            {
                let mut possible = true;
                for vy in 0..vault.height as i32 {
                    for vx in 0..vault.width as i32 {
                        let idx = self.map.xy_idx(vx + x, vy + y);
                        if self.map.tiles[idx] != TileType::Floor {
                            possible = false;
                        }
                    }
                }

                if possible {
                    vault_positions.push(Position { x, y });
                    break;
                }
            }
            i += 1;
        }

        if !vault_positions.is_empty() {
            let pos_idx = match vault_positions.len() {
                1 => 0,
                _ => (rng.roll_dice(1, vault_positions.len() as i32) - 1) as usize,
            };
            let pos = &vault_positions[pos_idx];

            let string_vec = PrefabBuilder::read_ascii_to_vec(vault.template);
            let mut i = 0;
            for y in 0..vault.height {
                for x in 0..vault.width {
                    let idx = self.map.xy_idx(x as i32 + pos.x, y as i32 + pos.y);
                    self.char_to_map(string_vec[i], idx);
                    i += 1;
                }
            }
            self.take_snapshot();
        }
    }

    fn apply_previous_iteration<F>(&mut self, mut filter: F)
    where
        F: FnMut(i32, i32, &(usize, String)) -> bool,
    {
        let prev_builder = self.previous_builder.as_mut().unwrap();
        prev_builder.build_map();
        self.starting_position = prev_builder.get_starting_position();
        self.map = prev_builder.get_map().clone();

        for ent in prev_builder.get_spawn_list().iter() {
            let x = ent.0 as i32 % self.map.width;
            let y = ent.0 as i32 / self.map.width;
            if filter(x, y, ent) {
                self.spawn_list.push((ent.0, ent.1.to_string()));
            }
        }
    }
}
