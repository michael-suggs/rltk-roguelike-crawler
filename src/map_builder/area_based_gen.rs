use rltk::DistanceAlg;

use crate::{MetaMapBuilder, Position, TileType};

use super::common::DistanceAlgorithm;

pub enum XStart { LEFT, CENTER, RIGHT }
pub enum YStart { TOP, CENTER, BOTTOM }

pub struct AreaStartingPosition {
    x: XStart,
    y: YStart,
}

impl MetaMapBuilder for AreaStartingPosition {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut crate::BuildData) {
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
                            idx as i32 / build_data.map.width
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


