use std::collections::HashMap;

use rltk::RandomNumberGenerator;

use crate::{
    spawner, BuildData, InitialMapBuilder, Map, MapBuilder, Position, TileType, MAPHEIGHT,
    MAPWIDTH, SHOW_MAPGEN_VISUALIZER,
};

use super::common::{
    generate_voronoi_spawn_regions, remove_unreachable_areas_returning_most_distant,
    DistanceAlgorithm,
};

/// Builer to construct a map by way of voronoi diagrams.
pub struct VoronoiBuilder {
    n_seeds: i32,
    diagram: VoronoiDiagram,
}

impl InitialMapBuilder for VoronoiBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuildData) {
        self.build(rng, build_data);
    }
}

impl VoronoiBuilder {
    pub fn new() -> Box<VoronoiBuilder> {
        match RandomNumberGenerator::new().roll_dice(1, 3) {
            1 => Self::pythagoras(64),
            2 => Self::manhattan(64),
            _ => Self::chebyshev(64),
        }
    }

    /// Constructs a new [`VoronoiBuilder`] using the distance algorithm
    /// [`rltk::DistanceAlg::Pythagoras`].
    pub fn pythagoras(n_seeds: i32) -> Box<VoronoiBuilder> {
        Box::new(VoronoiBuilder {
            n_seeds: 64,
            diagram: VoronoiDiagram::new(
                MAPWIDTH as i32,
                MAPHEIGHT as i32,
                DistanceAlgorithm::Pythagoras,
            ),
        })
    }

    /// Constructs a new [`VoronoiBuilder`] using the distance algorithm
    /// [`rltk::DistanceAlg::Manhattan`].
    pub fn manhattan(n_seeds: i32) -> Box<VoronoiBuilder> {
        Box::new(VoronoiBuilder {
            n_seeds: 64,
            diagram: VoronoiDiagram::new(
                MAPWIDTH as i32,
                MAPHEIGHT as i32,
                DistanceAlgorithm::Manhattan,
            ),
        })
    }

    /// Constructs a new [`VoronoiBuilder`] using the distance algorithm
    /// [`rltk::DistanceAlg::Chebyshev`].
    pub fn chebyshev(n_seeds: i32) -> Box<VoronoiBuilder> {
        Box::new(VoronoiBuilder {
            n_seeds: 64,
            diagram: VoronoiDiagram::new(
                MAPWIDTH as i32,
                MAPHEIGHT as i32,
                DistanceAlgorithm::Chebyshev,
            ),
        })
    }

    pub fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuildData) {
        for y in 1..build_data.map.height - 1 {
            for x in 1..build_data.map.width - 1 {
                let idx = build_data.map.xy_idx(x, y);
                let seed = self.diagram.membership[idx];
                let neighbors = self.diagram.neighbors(x, y, seed);

                if neighbors < 2 {
                    build_data.map.tiles[idx] = TileType::Floor;
                }
            }
            build_data.take_snapshot();
        }
    }
}

/// Handles seeding, membership, and neighboring.
struct VoronoiDiagram {
    pub membership: Vec<i32>,
    rng: rltk::RandomNumberGenerator,
    seeds: Vec<(usize, rltk::Point)>,
    width: i32,
    height: i32,
    distance: fn(rltk::Point, rltk::Point) -> f32,
}

impl VoronoiDiagram {
    /// Constructs a new seeded VoronoiDiagram with distance and
    /// membership calculated.
    pub fn new(width: i32, height: i32, distance_algorithm: DistanceAlgorithm) -> VoronoiDiagram {
        let mut vd = VoronoiDiagram {
            membership: vec![0; (width * height) as usize],
            rng: rltk::RandomNumberGenerator::new(),
            seeds: Vec::new(),
            width,
            height,
            distance: DistanceAlgorithm::get_func(&distance_algorithm),
        };
        vd.populate_seeds(64);
        vd.determine_membership(64);
        vd
    }

    /// Generates `n_seeds` random seeds within the specified dimensions.
    fn populate_seeds(&mut self, n_seeds: usize) {
        while self.seeds.len() < n_seeds {
            let vx = self.rng.roll_dice(1, self.width - 1);
            let vy = self.rng.roll_dice(1, self.height - 1);
            let vidx = self.xy_idx(vx, vy);

            let candidate = (vidx, rltk::Point::new(vx, vy));
            if !self.seeds.contains(&candidate) {
                self.seeds.push(candidate);
            }
        }
    }

    fn determine_membership(&mut self, n_seeds: usize) {
        let distance = self.distance;
        let mut vdistance = vec![(0, 0.0f32); n_seeds];
        for (i, vid) in self.membership.iter_mut().enumerate() {
            let x = i as i32 % self.width;
            let y = i as i32 / self.width;

            self.seeds.iter().enumerate().for_each(|(seed, pos)| {
                vdistance[seed] = (seed, (distance)(rltk::Point::new(x, y), pos.1))
            });

            vdistance.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            *vid = vdistance[0].0 as i32;
        }
    }

    fn neighbors(&self, x: i32, y: i32, seed: i32) -> i32 {
        let mut neighbors = 0;

        if self.membership[self.xy_idx(x - 1, y)] != seed {
            neighbors += 1;
        }
        if self.membership[self.xy_idx(x + 1, y)] != seed {
            neighbors += 1;
        }
        if self.membership[self.xy_idx(x, y - 1)] != seed {
            neighbors += 1;
        }
        if self.membership[self.xy_idx(x, y + 1)] != seed {
            neighbors += 1;
        }

        neighbors
    }

    fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }
}
