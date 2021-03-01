use specs::prelude::*;
use super::{Map, Monster, Name, Position, Viewshed};
use rltk::{console, field_of_view, Point};

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    // Really big type--let the linter know that we know.
    #[allow(clippy::type_complexity)]
    type SystemData = (
        // Expect a map resource; fail if not found.
        WriteExpect<'a, Map>,
        // Expect a player position resource; fail if not found.
        ReadExpect<'a, Point>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            player_pos,
            mut viewshed,
            monster,
            name,
            mut position
        ) = data;

        for (viewshed, _monster, name, mut pos)
                in (&mut viewshed, &monster, &name, &mut position).join() {
            if viewshed.visible_tiles.contains(&*player_pos) {
                console::log(&format!("{} considers their existence", name.name));
            }

            // Get path to the player with A*.
            let path = rltk::a_star_search(
                map.xy_idx(pos.x, pos.y) as i32,
                map.xy_idx(player_pos.x, player_pos.y) as i32,
                &mut *map
            );

            // If path is found, take a step and recalculate the viewshed.
            // `steps[0]` is the current position, so take the next step.
            if path.success && path.steps.len() > 1 {
                pos.x = path.steps[1] as i32 % map.width;
                pos.y = path.steps[1] as i32 / map.width;
                viewshed.dirty = true;
            }
        }
    }
}
