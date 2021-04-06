use crate::{spawner, Map, MapBuilder, Position, TileType, SHOW_MAPGEN_VISUALIZER};

#[allow(dead_code)]
#[derive(PartialEq, Clone)]
pub enum PrefabMode {
    RexLevel { template: &'static str },
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
            mode: PrefabMode::RexLevel {
                template: "../resources/wfc-populated.xp",
            },
            spawns: Vec::new(),
        }
    }

    fn build(&mut self) {
        match self.mode {
            PrefabMode::RexLevel { template } => self.load_rex_map(&template),
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
        self.take_snapshot();
    }

    fn load_rex_map(&mut self, path: &str) {
        self.spawns = crate::rex_assets::load_rex_map(&mut self.map, path);
    }
}
