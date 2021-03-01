use rltk::{console, Point, Rltk, VirtualKeyCode};
use specs::prelude::*;
use super::{CombatStats, Map, Position, Player, RunState, State, TileType,
    Viewshed, WantsToMelee, Item, WantsToPickupItem, gamelog::GameLog};
use std::cmp::{min, max};

/// Tries to move the player by `(delta_x, delta_y)` amount.
pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let map = ecs.fetch::<Map>();

    let combat_stats = ecs.read_storage::<CombatStats>();
    let entities = ecs.entities();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();

    for (ent, _player, pos, viewshed)
            in (&entities, &mut players, &mut positions, &mut viewsheds).join() {
        let (new_x, new_y) = (pos.x + delta_x, pos.y+ delta_y);
        if new_x < 1 || new_x > map.width - 1 || new_y < 1 || new_y > map.height - 1 { return; }
        let dest_idx = map.xy_idx(new_x, new_y);

        for potential_target in map.tile_content[dest_idx].iter() {
            let target = combat_stats.get(*potential_target);
            match target {
                None => {}
                Some(t) => {
                    wants_to_melee
                        .insert(ent, WantsToMelee { target: *potential_target })
                        .expect("Add target failed.");
                    // Attack it!
                    console::log(&format!("I bite my thumb at you, good sir!"));
                    return; // don't move after an attack
                }
            }
        }

        // Can't move through walls!
        if !map.blocked[dest_idx] {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));

            // If player was moved, viewshed needs to be recalculated.
            viewshed.dirty = true;

            // Update the player's position resource.
            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState{
    match ctx.key {
        None => { return RunState::AwaitingInput }
        Some(key) => match key {
            VirtualKeyCode::Left
            | VirtualKeyCode::Numpad4
            | VirtualKeyCode::H
            | VirtualKeyCode::A => try_move_player(-1, 0, &mut gs.ecs),

            VirtualKeyCode::Right
            | VirtualKeyCode::Numpad6
            | VirtualKeyCode::L
            | VirtualKeyCode::D => try_move_player(1, 0, &mut gs.ecs),

            VirtualKeyCode::Up
            | VirtualKeyCode::Numpad8
            | VirtualKeyCode::K
            | VirtualKeyCode::W => try_move_player(0, -1, &mut gs.ecs),

            VirtualKeyCode::Down
            | VirtualKeyCode::Numpad2
            | VirtualKeyCode::J
            | VirtualKeyCode::S => try_move_player(0, 1, &mut gs.ecs),

            VirtualKeyCode::Numpad7
            | VirtualKeyCode::U
            | VirtualKeyCode::E => try_move_player(1, -1, &mut gs.ecs),

            VirtualKeyCode::Numpad9
            | VirtualKeyCode::Y
            | VirtualKeyCode::Q => try_move_player(-1, -1, &mut gs.ecs),

            VirtualKeyCode::Numpad1
            | VirtualKeyCode::B
            | VirtualKeyCode::C => try_move_player(1, 1, &mut gs.ecs),

            VirtualKeyCode::Numpad3
            | VirtualKeyCode::N
            | VirtualKeyCode::Z => try_move_player(-1, 1, &mut gs.ecs),

            VirtualKeyCode::G => get_item(&mut gs.ecs),

            VirtualKeyCode::B
            | VirtualKeyCode::I => return RunState::ShowInventory,

            VirtualKeyCode::P => return RunState::ShowDropItem,

            _ => { return RunState::AwaitingInput }
        },
    } RunState::PlayerTurn
}

/// Handles item pickup.
fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_ent = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut log = ecs.fetch_mut::<GameLog>();

    // Check to see if there's an item under the player to pick up.
    let mut target_item: Option<Entity> = None;
    for (item_ent, _, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_ent);
        }
    }

    // Pick up the item, if there is one.
    match target_item {
        None => log.entries.push("There is nothing here to pickup.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup
                .insert(*player_ent, WantsToPickupItem { collected_by: *player_ent, item })
                .expect("Unable to insert want to pickup");
        }
    }
}
