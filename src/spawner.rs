use rltk::{RandomNumberGenerator, RGB};
use specs::{
    prelude::*,
    saveload::{MarkedBuilder, SimpleMarker},
};
use std::collections::HashMap;

use crate::{Map, TileType};

use super::{components::*, random_table::RandomTable, Rect, MAPWIDTH};

const MAX_MONSTERS: i32 = 4;

/// Spawns the player and returns its entity.
pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    ecs.create_entity()
        .with(Position {
            x: player_x,
            y: player_y,
        })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(Player {})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Name {
            name: "Player".to_string(),
        })
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .with(HungerClock {
            state: HungerState::WellFed,
            duration: 20,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

/// Spawns a room with entities from the spawn table.
pub fn spawn_room(ecs: &mut World, room: &Rect, map_depth: i32) {
    let mut possible_targets: Vec<usize> = Vec::new();
    {
        let map = ecs.fetch::<Map>();
        for y in room.y1 + 1..room.y2 {
            for x in room.x1 + 1..room.x2 {
                let idx = map.xy_idx(x, y);
                if map.tiles[idx] == TileType::Floor {
                    possible_targets.push(idx);
                }
            }
        }
    }
    spawn_region(ecs, &possible_targets, map_depth);
}

/// Spawns a contiguous area with entities from the spawn table.
pub fn spawn_region(ecs: &mut World, area: &[usize], map_depth: i32) {
    // Get spawn table for the current depth.
    let spawn_table = room_table(map_depth);
    // Map map indices to entity names for spawning.
    let mut spawn_points: HashMap<usize, String> = HashMap::new();
    // Copy to prevent modifying original slice.
    let mut areas: Vec<usize> = Vec::from(area);
    {
        // Get the rng we registered with the game to use for spawning.
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        // Cap the number of entities to spawn, so we don't spawn more than we have room for.
        let num_spawns = i32::min(
            areas.len() as i32,
            rng.roll_dice(1, MAX_MONSTERS + 3) + (map_depth - 1) - 3,
        );

        // If we're not spawning anything, might as well return.
        if num_spawns == 0 {
            return;
        }
        for _ in 0..num_spawns {
            // Otherwise, get an index from areas.
            let array_index = if areas.len() == 1 {
                // Only one entry, so we'll take that one.
                0usize
            } else {
                // More than one; roll a dice to see which one.
                (rng.roll_dice(1, areas.len() as i32) - 1) as usize
            };
            // Insert the new spawn point with a random entity to spawn.
            spawn_points.insert(areas[array_index], spawn_table.roll(&mut rng));
            // Already used as a spawn point, so take it out.
            areas.remove(array_index);
        }
    }
    for spawn in spawn_points.iter() {
        spawn_entity(ecs, &spawn);
    }
}

pub fn spawn_entity(ecs: &mut World, spawn: &(&usize, &String)) {
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
        "Longsword" => longsword(ecs, x, y),
        "Tower Shield" => tower_shield(ecs, x, y),
        "Rations" => rations(ecs, x, y),
        "Magic Mapping Scroll" => scroll_magic_mapping(ecs, x, y),
        "Bear Trap" => bear_trap(ecs, x, y),
        _ => {}
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
        .add("Longsword", map_depth - 3)
        .add("Tower Shield", map_depth - 3)
        .add("Rations", 6)
        .add("Magic Mapping Scroll", 2)
        .add("Bear Trap", 5)
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
fn monster<S: ToString>(ecs: &mut World, x: i32, y: i32, glyph: rltk::FontCharType, name: S) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: glyph,
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 1,
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Monster {})
        .with(Name {
            name: name.to_string(),
        })
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

fn rations(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('%'),
            fg: RGB::named(rltk::GREEN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Rations".to_string(),
        })
        .with(Item {})
        .with(ProvidesFood {})
        .with(Consumable {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

/// Spawns a health potion at `(x,y)`.
fn potion_health(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437(';'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Health Potion".to_string(),
        })
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
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Magic Missile Scroll".to_string(),
        })
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
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Fireball Scroll".to_string(),
        })
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
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::PINK),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Confusion Scroll".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(Confusion { turns: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn scroll_magic_mapping(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('◙'),
            fg: RGB::named(rltk::SANDY_BROWN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Scroll of Magic Mapping".to_string(),
        })
        .with(Item {})
        .with(MagicMapper {})
        .with(Consumable {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn dagger(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('/'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Dagger".to_string(),
        })
        .with(Item {})
        .with(Equippable {
            slot: EquipmentSlot::Melee,
        })
        .with(MeleePowerBonus { power: 2 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn longsword(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('/'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Longsword".to_string(),
        })
        .with(Item {})
        .with(Equippable {
            slot: EquipmentSlot::Melee,
        })
        .with(MeleePowerBonus { power: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn shield(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('('),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Shield".to_string(),
        })
        .with(Item {})
        .with(Equippable {
            slot: EquipmentSlot::Shield,
        })
        .with(DefenseBonus { defense: 1 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn tower_shield(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('('),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Tower Shield".to_string(),
        })
        .with(Item {})
        .with(Equippable {
            slot: EquipmentSlot::Shield,
        })
        .with(DefenseBonus { defense: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn bear_trap(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('^'),
            fg: RGB::named(rltk::GREY),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Bear Trap".to_string(),
        })
        .with(Hidden {})
        .with(EntryTrigger {})
        .with(SingleActivation {})
        .with(InflictsDamage { damage: 6 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
