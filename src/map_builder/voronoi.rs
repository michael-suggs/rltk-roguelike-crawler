use std::collections::HashMap;

use crate::{
    spawner, Map, MapBuilder, Position, TileType, MAPHEIGHT, MAPWIDTH, SHOW_MAPGEN_VISUALIZER,
};

#[derive(PartialEq, Clone, Copy)]
pub enum DistanceAlgorithm {
    Pythagoras,
    Manhattan,
    Chebyshev,
}

fn match_distance_algorithm(alg: DistanceAlgorithm) -> fn(rltk::Point, rltk::Point) -> f32 {
    match alg {
        DistanceAlgorithm::Pythagoras => |start: rltk::Point, end: rltk::Point| {
            rltk::DistanceAlg::PythagorasSquared.distance2d(start, end)
        },
        DistanceAlgorithm::Manhattan => |start: rltk::Point, end: rltk::Point| {
            rltk::DistanceAlg::Manhattan.distance2d(start, end)
        },
        DistanceAlgorithm::Chebyshev => |start: rltk::Point, end: rltk::Point| {
            rltk::DistanceAlg::Chebyshev.distance2d(start, end)
        },
    }
}

pub struct VoronoiBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
    diagram: VoronoiDiagram,
}

impl MapBuilder for VoronoiBuilder {
    fn build_map(&mut self) {
        self.build()
    }

    fn spawn_entities(&mut self, ecs: &mut specs::World) {
        self.noise_areas
            .iter()
            .for_each(|area| spawner::spawn_region(ecs, area.1, self.depth));
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

impl VoronoiBuilder {
    pub fn new(new_depth: i32) -> VoronoiBuilder {
        VoronoiBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            diagram: VoronoiDiagram::new(
                MAPWIDTH as i32,
                MAPHEIGHT as i32,
                DistanceAlgorithm::Pythagoras,
            ),
        }
    }

    pub fn build(&mut self) {
        for y in 1..self.map.height - 1 {
            for x in 1..self.map.width - 1 {
                let idx = self.map.xy_idx(x, y);
                let seed = self.diagram.membership[idx];
                let neighbors = self.diagram.neighbors(x, y, seed);

                if neighbors < 2 {
                    self.map.tiles[idx] = TileType::Floor;
                }
            }
            self.take_snapshot();
        }
    }
}

struct VoronoiDiagram {
    pub membership: Vec<i32>,
    rng: rltk::RandomNumberGenerator,
    seeds: Vec<(usize, rltk::Point)>,
    width: i32,
    height: i32,
    distance: fn(rltk::Point, rltk::Point) -> f32,
}

impl VoronoiDiagram {
    pub fn new(width: i32, height: i32, distance_algorithm: DistanceAlgorithm) -> VoronoiDiagram {
        let mut vd = VoronoiDiagram {
            membership: vec![0; (width * height) as usize],
            rng: rltk::RandomNumberGenerator::new(),
            seeds: Vec::new(),
            width,
            height,
            distance: match_distance_algorithm(distance_algorithm),
        };
        vd.populate_seeds(64);
        vd.determine_membership(64);
        vd
    }

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
