use std::collections::HashSet;

use crate::{
    spawner, BuildData, InitialMapBuilder, Map, MetaMapBuilder, Position, TileType,
    SHOW_MAPGEN_VISUALIZER,
};

use prefab_rooms::PrefabRoom;
use prefab_sections::{HorizontalPlacement, VerticalPlacement};
use rltk::RandomNumberGenerator;

pub mod prefab_levels;
pub mod prefab_rooms;
pub mod prefab_sections;

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
    mode: PrefabMode,
}

impl InitialMapBuilder for PrefabBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuildData) {
        self.build(rng, build_data);
    }
}

impl MetaMapBuilder for PrefabBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuildData) {
        self.build(rng, build_data);
    }
}

impl PrefabBuilder {
    pub fn room_vaults() -> Box<PrefabBuilder> {
        Box::new(PrefabBuilder {
            mode: PrefabMode::RoomVaults,
        })
    }

    pub fn rex_level(template: &'static str) -> Box<PrefabBuilder> {
        Box::new(PrefabBuilder {
            mode: PrefabMode::RexLevel { template },
        })
    }

    pub fn constant(level: prefab_levels::PrefabLevel) -> Box<PrefabBuilder> {
        Box::new(PrefabBuilder {
            mode: PrefabMode::Constant { level },
        })
    }

    pub fn sectional(section: prefab_sections::PrefabSection) -> Box<PrefabBuilder> {
        Box::new(PrefabBuilder {
            mode: PrefabMode::Sectional { section },
        })
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuildData) {
        match self.mode {
            PrefabMode::RexLevel { template } => self.load_rex_map(&template, build_data),
            PrefabMode::Constant { level } => self.load_ascii_map(&level, build_data),
            PrefabMode::Sectional { section } => self.apply_sectional(&section, rng, build_data),
            PrefabMode::RoomVaults => self.apply_room_vaults(rng, build_data),
        }
        build_data.take_snapshot();
    }

    fn char_to_map(&mut self, ch: char, idx: usize, build_data: &mut BuildData) {
        match ch {
            ' ' => build_data.map.tiles[idx] = TileType::Floor,
            '#' => build_data.map.tiles[idx] = TileType::Wall,
            '@' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.start = Some(Position {
                    x: idx as i32 % build_data.map.width,
                    y: idx as i32 / build_data.map.width,
                });
            }
            '>' => build_data.map.tiles[idx] = TileType::DownStairs,
            'g' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, "Goblin".to_string()));
            }
            'o' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, "Orc".to_string()));
            }
            '^' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, "Bear Trap".to_string()));
            }
            '%' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, "Rations".to_string()));
            }
            '!' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data
                    .spawn_list
                    .push((idx, "Health Potion".to_string()));
            }
            _ => rltk::console::log(format!("Unknown glyph when loading map: {}", ch)),
        }
    }

    fn load_rex_map(&mut self, path: &str, build_data: &mut BuildData) {
        let xp_file = rltk::rex::XpFile::from_resource(path).unwrap();

        for layer in &xp_file.layers {
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let cell = layer.get(x, y).unwrap();
                    if x < build_data.map.width as usize && y < build_data.map.height as usize {
                        let idx = build_data.map.xy_idx(x as i32, y as i32);
                        self.char_to_map(cell.ch as u8 as char, idx, build_data);
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

    fn load_ascii_map(&mut self, level: &prefab_levels::PrefabLevel, build_data: &mut BuildData) {
        let string_vec: Vec<char> = PrefabBuilder::read_ascii_to_vec(level.template);
        let mut i = 0;
        for y in 0..level.height {
            for x in 0..level.width {
                if x > 0
                    && y > 0
                    && x < build_data.map.width as usize
                    && y < build_data.map.height as usize
                {
                    let idx = build_data.map.xy_idx(x as i32, y as i32);
                    self.char_to_map(string_vec[i], idx, build_data);
                }
                i += 1;
            }
        }
    }

    fn apply_sectional(
        &mut self,
        section: &prefab_sections::PrefabSection,
        rng: &mut RandomNumberGenerator,
        build_data: &mut BuildData,
    ) {
        let string_vec = PrefabBuilder::read_ascii_to_vec(
            prefab_sections::get_template_str(section.to_owned()).as_str(),
        );
        let chunk_x = match section.placement.0 {
            HorizontalPlacement::Left => 0,
            HorizontalPlacement::Center => (build_data.map.width / 2) - (section.width as i32 / 2),
            HorizontalPlacement::Right => (build_data.map.width - 1) - section.width as i32,
        };
        let chunk_y = match section.placement.1 {
            VerticalPlacement::Top => 0,
            VerticalPlacement::Center => (build_data.map.height / 2) - (section.height as i32 / 2),
            VerticalPlacement::Bottom => (build_data.map.height - 1) - section.height as i32,
        };

        self.apply_previous_iteration(
            |x, y, e| {
                x < chunk_x
                    || x > (chunk_x + section.width as i32)
                    || y < chunk_y
                    || y > (chunk_y + section.height as i32)
            },
            rng,
            build_data,
        );

        let mut i = 0;
        for y in 0..section.height {
            for x in 0..section.width {
                if build_data.map.in_bounds(x as i32, 0, y as i32, 0) {
                    let idx = build_data
                        .map
                        .xy_idx(x as i32 + chunk_x, y as i32 + chunk_y);
                    if i < string_vec.len() {
                        self.char_to_map(string_vec[i], idx, build_data);
                    }
                }
                i += 1;
            }
        }
        build_data.take_snapshot();
    }

    fn apply_room_vaults(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuildData) {
        self.apply_previous_iteration(|_, _, _| true, rng, build_data);

        if rng.roll_dice(1, 6) + build_data.map.depth < 4 {
            return;
        }

        let master_vault_list = vec![
            prefab_rooms::NOT_A_TRAP,
            prefab_rooms::CHECKERBOARD,
            prefab_rooms::SILLY_SMILE,
        ];
        let possible_vaults: Vec<&PrefabRoom> = master_vault_list
            .iter()
            .filter(|v| {
                build_data.map.depth >= v.first_depth && build_data.map.depth <= v.last_depth
            })
            .collect();
        if possible_vaults.is_empty() {
            return;
        }

        let mut used_tiles: HashSet<usize> = HashSet::new();
        let n_vaults = i32::min(
            rng.roll_dice(1, master_vault_list.len() as i32),
            possible_vaults.len() as i32,
        );

        for _ in 0..n_vaults {
            let vidx = match possible_vaults.len() {
                1 => 0,
                _ => (rng.roll_dice(1, possible_vaults.len() as i32) - 1) as usize,
            };
            let vault = possible_vaults[vidx];
            let mut vault_positions: Vec<Position> = Vec::new();

            let mut i = 0usize;
            while i < (build_data.map.tiles.len() - 1) {
                let x = (i % build_data.map.width as usize) as i32;
                let y = (i / build_data.map.width as usize) as i32;

                if x > 1
                    && y > 1
                    && (x + vault.width as i32) < build_data.map.width - 2
                    && (y + vault.height as i32) < build_data.map.height - 2
                {
                    let mut possible = true;
                    for vy in 0..vault.height as i32 {
                        for vx in 0..vault.width as i32 {
                            let idx = build_data.map.xy_idx(vx + x, vy + y);
                            possible = (build_data.map.tiles[idx] == TileType::Floor)
                                && !used_tiles.contains(&idx);
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

                let width = build_data.map.width;
                let height = build_data.map.height;
                build_data.spawn_list.retain(|ent| {
                    let x = ent.0 as i32 % width;
                    let y = ent.0 as i32 / height;
                    x < pos.x
                        || x > pos.x + vault.width as i32
                        || y < pos.y
                        || y > pos.y + vault.height as i32
                });

                let string_vec = PrefabBuilder::read_ascii_to_vec(vault.template);
                let mut i = 0;
                for y in 0..vault.height {
                    for x in 0..vault.width {
                        let idx = build_data.map.xy_idx(x as i32 + pos.x, y as i32 + pos.y);
                        self.char_to_map(string_vec[i], idx, build_data);
                        used_tiles.insert(idx);
                        i += 1;
                    }
                }
                build_data.take_snapshot();
            }
        }
    }

    fn apply_previous_iteration<F>(
        &mut self,
        mut filter: F,
        rng: &mut RandomNumberGenerator,
        build_data: &mut BuildData,
    ) where
        F: FnMut(i32, i32, &(usize, String)) -> bool,
    {
        let width = build_data.map.width;
        build_data.spawn_list.retain(|ent| {
            let x = ent.0 as i32 % width;
            let y = ent.0 as i32 / width;
            filter(x, y, ent)
        });
        build_data.take_snapshot();
        // let prev_builder = self.previous_builder.as_mut().unwrap();
        // prev_builder.build_map();
        // self.starting_position = prev_builder.get_starting_position();
        // self.map = prev_builder.get_map().clone();

        // for ent in prev_builder.get_spawn_list().iter() {
        //     let x = ent.0 as i32 % self.map.width;
        //     let y = ent.0 as i32 / self.map.width;
        //     if filter(x, y, ent) {
        //         self.spawn_list.push((ent.0, ent.1.to_string()));
        //     }
        // }
    }
}
