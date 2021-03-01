use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;
use super::{Map, Position, Player, RunState, State, TileType, Viewshed};
use std::cmp::{min, max};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let map = ecs.fetch::<Map>();

    for (_player, pos, viewshed) in (&mut players, &mut positions, &mut viewsheds).join() {
        let dest_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);
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
        None => { return RunState::Paused }
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

            _ => { return RunState::Paused }
        },
    } RunState::Running
}
