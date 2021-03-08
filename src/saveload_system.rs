use specs::{
    prelude::*,
    saveload::{SimpleMarker, SimpleMarkerAllocator, SerializeComponents,
               DeserializeComponents, MarkedBuilder},
    error::NoError,
};
use std::{fs, fs::File, path::Path};
use super::{components::*, Map};

macro_rules! serialize_individually {
    ($ecs:expr, $ser:expr, $data:expr, $( $type:ty),*) => {
        $(
        SerializeComponents::<NoError, SimpleMarker<SerializeMe>>::serialize(
            &( $ecs.read_storage::<$type>(), ),
            &$data.0,
            &$data.1,
            &mut $ser,
        )
        .unwrap();
        )*
    };
}

pub fn does_save_exist() -> bool {
    Path::new("./savegame.json").exists()
}

pub fn delete_save() {
    if Path::new("./savegame.json").exists() {
        std::fs::remove_file("./savegame.json").expect("Unable to delete file");
    }
}

#[cfg(target_arch = "wasm32")]
pub fn save_game(_ecs: &mut World) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_game(ecs: &mut World) {
    // Create helper with copy of the game map
    let mapcopy = ecs.get_mut::<Map>().unwrap().clone();
    let savehelper = ecs
        .create_entity()
        .with(SerializationHelper { map: mapcopy })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    // Serialize
    {
        let data = (
            ecs.entities(),
            ecs.read_storage::<SimpleMarker<SerializeMe>>()
        );
        let writer = File::create("./savegame.json").unwrap();
        let mut serializer = serde_json::Serializer::new(writer);
        serialize_individually!(
            ecs, serializer, data, Position, Renderable, Player, Viewshed, Monster,
            Name, BlocksTile, CombatStats, SufferDamage, WantsToMelee, Item, Consumable,
            Ranged, InflictsDamage, AreaOfEffect, Confusion, ProvidesHealing, InBackpack,
            WantsToPickupItem, WantsToUseItem, WantsToDropItem, WantsToRemoveItem,
            SerializationHelper, Equippable, Equipped, MeleePowerBonus, DefenseBonus,
            ParticleLifetime, HungerClock, ProvidesFood, MagicMapper, Hidden, EntryTrigger,
            EntityMoved, SingleActivation
        );
    }

    // Clean up
    ecs.delete_entity(savehelper).expect("Crash on cleanup");
}

macro_rules! deserialize_individually {
    ($ecs:expr, $de:expr, $data:expr, $( $type:ty),*) => {
        $(
        DeserializeComponents::<NoError, _>::deserialize(
            &mut ( &mut $ecs.write_storage::<$type>(), ),
            &mut $data.0, // entities
            &mut $data.1, // marker
            &mut $data.2, // allocator
            &mut $de,
        )
        .unwrap();
        )*
    };
}

pub fn load_game(ecs: &mut World) {
    // Two-step iteration of entities, deleting all entities in the game.
    {
        // Vec to store entities from the first pass, to delete from in the second.
        let mut to_delete = Vec::new();
        // First pass--get entities to delete.
        for e in ecs.entities().join() {
            to_delete.push(e);
        }
        // Second pass--delete the entities we found in the previous loop.
        for del in to_delete.iter() {
            ecs.delete_entity(*del).expect("Deletion failed");
        }
    }

    // Open the savegame path and attach the deserializer.
    let data = fs::read_to_string("./savegame.json").unwrap();
    let mut de = serde_json::Deserializer::from_str(&data);

    {
        // Build deserialize macro tuple.
        let mut d = (
            &mut ecs.entities(),
            &mut ecs.write_storage::<SimpleMarker<SerializeMe>>(),
            &mut ecs.write_resource::<SimpleMarkerAllocator<SerializeMe>>(),
        );

        // Call macro on tuple, deserializing each type in turn (done in same
        // order as with saving).
        deserialize_individually!(
            ecs, de, d, Position, Renderable, Player, Viewshed, Monster,
            Name, BlocksTile, CombatStats, SufferDamage, WantsToMelee, Item, Consumable,
            Ranged, InflictsDamage, AreaOfEffect, Confusion, ProvidesHealing, InBackpack,
            WantsToPickupItem, WantsToUseItem, WantsToDropItem, WantsToRemoveItem,
            SerializationHelper, Equippable, Equipped, MeleePowerBonus, DefenseBonus,
            ParticleLifetime, HungerClock, ProvidesFood, MagicMapper, Hidden, EntryTrigger,
            EntityMoved, SingleActivation
        );
    }

    let mut deleteme: Option<Entity> = None;
    {
        let entities = ecs.entities();
        let helper = ecs.read_storage::<SerializationHelper>();
        let player = ecs.read_storage::<Player>();
        let position = ecs.read_storage::<Position>();

        // Iterate entities with SerializationHelper component.
        for (e, h) in (&entities, &helper).join() {
            // Replace resource storing the map.
            let mut worldmap = ecs.write_resource::<Map>();
            *worldmap = h.map.clone();
            // `tile_content` isn't serialized, so replace with empty set of vectors.
            worldmap.tile_content = vec![Vec::new(); super::MAPCOUNT];
            deleteme = Some(e);
        }

        // Find the player and store its world resource and position.
        for (e, _p, pos) in (&entities, &player, &position).join() {
            let mut ppos = ecs.write_resource::<rltk::Point>();
            *ppos = rltk::Point::new(pos.x, pos.y);

            let mut player_resource = ecs.write_resource::<Entity>();
            *player_resource = e;
        }
    }

    ecs.delete_entity(deleteme.unwrap()).expect("Unable to delete helper");
}
