use crate::{spawner, BuildData, MetaMapBuilder, Position, TileType};

pub struct RoomBasedSpawner {}

impl MetaMapBuilder for RoomBasedSpawner {
    fn build_map(
        &mut self,
        rng: &mut rltk::RandomNumberGenerator,
        build_data: &mut crate::BuildData,
    ) {
        self.build(rng, build_data);
    }
}

impl RoomBasedSpawner {
    pub fn new() -> Box<RoomBasedSpawner> {
        Box::new(RoomBasedSpawner {})
    }

    fn build(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuildData) {
        if let Some(rooms) = &build_data.rooms {
            for room in rooms.iter().skip(1) {
                spawner::spawn_room(
                    &build_data.map,
                    rng,
                    room,
                    build_data.map.depth,
                    &mut build_data.spawn_list,
                );
            }
        } else {
            panic!("Room-based spawning only works after rooms have been created");
        }
    }
}

pub struct RoomBasedStartingPosition {}

impl MetaMapBuilder for RoomBasedStartingPosition {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuildData) {
        self.build(rng, build_data);
    }
}

impl RoomBasedStartingPosition {
    pub fn new() -> Box<RoomBasedStartingPosition> {
        Box::new(RoomBasedStartingPosition {})
    }

    fn build(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuildData) {
        if let Some(rooms) = &build_data.rooms {
            let start = rooms[0].center();
            build_data.start = Some(Position::from(start));
        } else {
            panic!("Room-based start only works after rooms have been created");
        }
    }
}

pub struct RoomBasedStairs {}

impl MetaMapBuilder for RoomBasedStairs {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuildData) {
        self.build(rng, build_data);
    }
}

impl RoomBasedStairs {
    pub fn new() -> Box<RoomBasedStairs> {
        Box::new(RoomBasedStairs {})
    }

    fn build(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuildData) {
        if let Some(rooms) = &build_data.rooms {
            let stairs = rooms.last().unwrap().center();
            let idx = build_data.map.xy_idx(stairs.0, stairs.1);
            build_data.map.tiles[idx] = TileType::DownStairs;
            build_data.take_snapshot();
        } else {
            panic!("Room-based stairs only works after rooms have been created")
        }
    }
}
