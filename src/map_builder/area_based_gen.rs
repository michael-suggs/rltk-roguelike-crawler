use std::collections::HashMap;

use rltk::DistanceAlg;

use crate::{spawner, MetaMapBuilder, Position, TileType};

use super::common::DistanceAlgorithm;

pub enum XStart {
    LEFT,
    CENTER,
    RIGHT,
}
pub enum YStart {
    TOP,
    CENTER,
    BOTTOM,
}

pub struct AreaStartingPosition {
    x: XStart,
    y: YStart,
}

impl MetaMapBuilder for AreaStartingPosition {
    fn build_map(
        &mut self,
        rng: &mut rltk::RandomNumberGenerator,
        build_data: &mut crate::BuildData,
    ) {
        self.build(rng, build_data);
    }
}

impl AreaStartingPosition {
    pub fn new(x: XStart, y: YStart) -> Box<AreaStartingPosition> {
        Box::new(AreaStartingPosition { x, y })
    }

    fn build(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut crate::BuildData) {
        let seed_x = match self.x {
            XStart::LEFT => 1,
            XStart::CENTER => build_data.map.width / 2,
            XStart::RIGHT => build_data.map.width - 2,
        };

        let seed_y = match self.y {
            YStart::TOP => 1,
            YStart::CENTER => build_data.map.height / 2,
            YStart::BOTTOM => build_data.map.height - 2,
        };

        let mut available_floors: Vec<(usize, f32)> = Vec::new();
        for (idx, tile) in build_data.map.tiles.iter().enumerate() {
            if *tile == TileType::Floor {
                available_floors.push((
                    idx,
                    DistanceAlgorithm::Pythagoras.apply(
                        rltk::Point::new(
                            idx as i32 % build_data.map.width,
                            idx as i32 / build_data.map.width,
                        ),
                        rltk::Point::new(seed_x, seed_y),
                    ),
                ));
            }
        }

        if available_floors.is_empty() {
            panic!("No valid floors to start on");
        }

        available_floors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        build_data.start = Some(Position {
            x: available_floors[0].0 as i32 % build_data.map.width,
            y: available_floors[0].0 as i32 / build_data.map.width,
        });
    }
}

pub struct VoronoiSpawning {}

impl MetaMapBuilder for VoronoiSpawning {
    fn build_map(
        &mut self,
        rng: &mut rltk::RandomNumberGenerator,
        build_data: &mut crate::BuildData,
    ) {
        self.build(rng, build_data);
    }
}

impl VoronoiSpawning {
    pub fn new() -> Box<VoronoiSpawning> {
        Box::new(VoronoiSpawning {})
    }

    fn build(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut crate::BuildData) {
        let mut noise_areas: HashMap<i32, Vec<usize>> = HashMap::new();
        let mut noise = rltk::FastNoise::seeded(rng.roll_dice(1, 65536) as u64);
        noise.set_noise_type(rltk::NoiseType::Cellular);
        noise.set_frequency(0.08);
        noise.set_cellular_distance_function(rltk::CellularDistanceFunction::Manhattan);

        for (x, y) in build_data.map.iter_xy() {
            let idx = build_data.map.xy_idx(x, y);
            if build_data.map.tiles[idx] == TileType::Floor {
                let cell_value = (noise.get_noise(x as f32, y as f32) * 10240.0) as i32;
                if noise_areas.contains_key(&cell_value) {
                    noise_areas.get_mut(&cell_value).unwrap().push(idx);
                } else {
                    noise_areas.insert(cell_value, vec![idx]);
                }
            }
        }

        for area in noise_areas.iter() {
            spawner::spawn_region(
                &build_data.map,
                rng,
                area.1,
                build_data.map.depth,
                &mut build_data.spawn_list,
            );
        }
    }
}
