use rltk::{RGB, RandomNumberGenerator, to_cp437};
use serde::de::value::MapDeserializer;
use specs::{prelude::*, saveload::{MarkedBuilder, SimpleMarker}};
use std::collections::HashMap;
use super::{components::*, random_table::RandomTable, Rect, MAPWIDTH};

const MAX_MONSTERS: i32 = 4;

/// Spawns the player and returns its entity.
pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(Player {})
        .with(Viewshed { visible_tiles: Vec::new(), range: 8, dirty: true })
        .with(Name { name: "Player".to_string() })
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

/// Spawns stuff in a room.
pub fn spawn_room(ecs: &mut World, room: &Rect, map_depth: i32) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points: HashMap<usize, String> = HashMap::new();

    // Get some spawn points (scoped to appease the borrow checker).
    {
        // Get a random number of monsters to spawn in the room.
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_spawns = rng.roll_dice(1, MAX_MONSTERS + 3) + (map_depth - 1) - 3;

        // Get spawn points for the monsters in the room.
        for _ in 0..num_spawns {
            let mut added = false;
            let mut tries = 0;

            // Try to get an unoccupied spawn point.
            while !added && tries < 20 {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH) + x;

                // If spawn point is unoccupied, add it as a new spawn point
                // then continue to the next.
                if !spawn_points.contains_key(&idx) {
                    spawn_points.insert(idx, spawn_table.roll(&mut rng));
                    added = true;
                } else {
                    tries += 1
                }
            }
        }
    }

    // Spawn monsters and items, matching on our rolls.
    for spawn in spawn_points.iter() {
        let (x, y) = ((*spawn.0 % MAPWIDTH) as i32, (*spawn.0 / MAPWIDTH) as i32);
        match spawn.1.as_ref() {
            "Goblin" => goblin(ecs, x, y),
            "Orc" => orc(ecs, x, y),
            "Health Potion" => potion_health(ecs, x, y),
            "Fireball Scroll" => scroll_fireball(ecs, x, y),
            "Confusion Scroll" => scroll_confusion(ecs, x, y),
            "Magic Missile Scroll" => scroll_magic_missile(ecs, x, y),
            "Dagger" => dagger(ecs, x, y),
            "Shield" => shield(ecs, x, y),
            _ => {}
        }
    }
}

fn room_table(map_depth: i32) -> RandomTable {
    RandomTable::new()
        .add("Goblin", 10)
        .add("Orc", 1 + map_depth)
        .add("Health Potion", 7)
        .add("Fireball Scroll", 2 + map_depth)
        .add("Confusion Scroll", 2 + map_depth)
        .add("Magic Missile Scroll", 4)
        .add("Dagger", 3)
        .add("Shield", 3)
}

/// Makes an orc.
fn orc(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, rltk::to_cp437('o'), "Orc");
}

/// Makes a goblin.
fn goblin(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, rltk::to_cp437('g'), "Goblin");
}

/// Spawns a monster at `(x,y)` with a given glyph and name.
fn monster<S: ToString>(
    ecs: &mut World, x: i32, y: i32, glyph: rltk::FontCharType, name: S
) {
    ecs
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: glyph,
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 1,
        })
        .with(Viewshed { visible_tiles: Vec::new(), range: 8, dirty: true })
        .with(Monster {})
        .with(Name { name: name.to_string() })
        .with(BlocksTile {})
        .with(CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

/// Spawns a health potion at `(x,y)`.
fn potion_health(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437(';'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "Health Potion".to_string() })
        .with(Item {})
        .with(Consumable {})
        .with(ProvidesHealing { heal_amount: 8 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

/// Spawns a Magic Missile Scroll at `(x,y)`.
///
/// Magic missile scrolls target a single entity, and are consumed on use.
fn scroll_magic_missile(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name { name: "Magic Missile Scroll".to_string() })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 8 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

/// Spawns a fireball scroll at `(x,y)`.
///
/// Fireball is an area-of-effect ability, hitting all entities within range
/// of the targeted location. Like other scrolls, these are consumed on use.
fn scroll_fireball(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name { name: "Fireball Scroll".to_string() })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 20 })
        .with(AreaOfEffect { radius: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

/// Spawns a confusion scroll at `(x,y)`.
///
/// Confusion targets a single entity at range, and confuses them for a number
/// of turns. During this time, the entity is unable to perform any actions.
fn scroll_confusion(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::PINK),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "Confusion Scroll".to_string() })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(Confusion { turns: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn dagger(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('/'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "Dagger".to_string() })
        .with(Item {})
        .with(Equippable { slot: EquipmentSlot::Melee })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn shield(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('('),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "Shield".to_string() })
        .with(Item {})
        .with(Equippable { slot: EquipmentSlot::Shield })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
