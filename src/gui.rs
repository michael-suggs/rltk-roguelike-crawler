use rltk::{Console, Point, RGB, Rltk};
use specs::prelude::*;
use super::{CombatStats, GameLog, Map, Name, Player, Position};

/// Draws the UI to the bottom of the screen.
pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    ctx.draw_box(0, 43, 79, 6, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();

    for (_player, stats) in (&players, &combat_stats).join() {
        let health = format!(" HP: {} / {} ", stats.hp, stats.max_hp);
        ctx.print_color(12, 34, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), &health);
        ctx.draw_bar_horizontal(23, 43, 51, stats.hp, stats.max_hp,
            RGB::named(rltk::RED), RGB::named(rltk::BLACK));
    }

    let log = ecs.fetch::<GameLog>();
    let mut y = 44;
    for s in log.entries.iter().rev() {
        if y < 49 { ctx.print(2, y, s); }
        y += 1;
    }

    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::MAGENTA));
    draw_tooltips(ecs, ctx);
}

fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    // Get access to names and positions to make tooltips with.
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();

    // Make sure the map cursor is actually on the map.
    let mouse_pos = ctx.mouse_pos();
    if mouse_pos.0 >= map.width || mouse_pos.1 >= map.height { return; }

    // If there's something under the mouse, we'll make a tooltip for it.
    let mut tooltip: Vec<String> = Vec::new();
    for (name, pos) in (&names, &positions).join() {
        let idx = map.xy_idx(pos.x, pos.y);
        if pos.x == mouse_pos.0 && pos.y == mouse_pos.1 && map.visible_tiles[idx] {
            tooltip.push(name.name.to_string());
        }
    }

    // Make tooltips if we found things to make them for.
    if !tooltip.is_empty() {
        let mut width: i32 = 0;
        for s in tooltip.iter() {
            if width < s.len() as i32 { width = s.len() as i32; }
        } width += 3;

        // Switch which side we place the tooltip on based on mouse position.
        if mouse_pos.0 > 40 {
            let arrow_pos = Point::new(mouse_pos.0 - 2, mouse_pos.1);
            let left_x = mouse_pos.0 - width;
            let mut y = mouse_pos.1;

            for s in tooltip.iter() {
                ctx.print_color(left_x, y,
                    RGB::named(rltk::WHITE), RGB::named(rltk::GREY), s);
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x - i, y,
                        RGB::named(rltk::WHITE), RGB::named(rltk::GREY), &" ".to_string());
                }
                y += 1;
            }
            // Tooltip is to the left of the item, point right.
            ctx.print_color(arrow_pos.x, arrow_pos.y,
                RGB::named(rltk::WHITE), RGB::named(rltk::GREY), &"->".to_string());

        } else {
            let arrow_pos = Point::new(mouse_pos.0 + 1, mouse_pos.1);
            let left_x = mouse_pos.0 + 3;
            let mut y = mouse_pos.1;

            for s in tooltip.iter() {
                ctx.print_color(left_x + 1, y,
                    RGB::named(rltk::WHITE), RGB::named(rltk::GREY), s);
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x + 1 + i, y,
                        RGB::named(rltk::WHITE), RGB::named(rltk::GREY), &" ".to_string());
                }
                y += 1;
            }
            // Tooltip is to the right of the item, point left.
            ctx.print_color(arrow_pos.x, arrow_pos.y,
                RGB::named(rltk::WHITE), RGB::named(rltk::GREY), &"<-".to_string());
        }
    }
}
