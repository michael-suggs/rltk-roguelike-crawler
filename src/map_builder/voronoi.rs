use std::collections::HashMap;

use rltk::RandomNumberGenerator;

use crate::{
    spawner, Map, MapBuilder, Position, TileType, MAPHEIGHT, MAPWIDTH, SHOW_MAPGEN_VISUALIZER,
};

use super::common::{
    generate_voronoi_spawn_regions, remove_unreachable_areas_returning_most_distant,
    DistanceAlgorithm,
};

/// Builer to construct a map by way of voronoi diagrams.
pub struct VoronoiBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
    diagram: VoronoiDiagram,
    spawn_list: Vec<(usize, String)>,
}

impl MapBuilder for VoronoiBuilder {
    fn build_map(&mut self) {
        self.build()
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

    fn get_spawn_list(&self) -> &Vec<(usize, String)> {
        &self.spawn_list
    }
}

impl VoronoiBuilder {
    fn new(new_depth: i32, distance_algorithm: DistanceAlgorithm) -> VoronoiBuilder {
        VoronoiBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            diagram: VoronoiDiagram::new(MAPWIDTH as i32, MAPHEIGHT as i32, distance_algorithm),
            spawn_list: Vec::new(),
        }
    }

    /// Constructs a new [`VoronoiBuilder`] using the distance algorithm
    /// [`rltk::DistanceAlg::Pythagoras`].
    pub fn pythagoras(new_depth: i32) -> VoronoiBuilder {
        VoronoiBuilder::new(new_depth, DistanceAlgorithm::Pythagoras)
    }

    /// Constructs a new [`VoronoiBuilder`] using the distance algorithm
    /// [`rltk::DistanceAlg::Manhattan`].
    pub fn manhattan(new_depth: i32) -> VoronoiBuilder {
        VoronoiBuilder::new(new_depth, DistanceAlgorithm::Manhattan)
    }

    /// Constructs a new [`VoronoiBuilder`] using the distance algorithm
    /// [`rltk::DistanceAlg::Chebyshev`].
    pub fn chebyshev(new_depth: i32) -> VoronoiBuilder {
        VoronoiBuilder::new(new_depth, DistanceAlgorithm::Chebyshev)
    }

    pub fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();
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

        let exit_tile = remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);
        self.map.tiles[exit_tile] = TileType::DownStairs;
        self.take_snapshot();

        self.noise_areas = generate_voronoi_spawn_regions(&self.map, &mut rng);
        for area in self.noise_areas.iter().skip(1) {
            spawner::spawn_region(
                &self.map,
                &mut rng,
                area.1,
                self.depth,
                &mut self.spawn_list,
            );
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
