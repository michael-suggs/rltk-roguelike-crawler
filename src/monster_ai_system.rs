use specs::prelude::*;
use super::{Map, Monster, Name, Position, RunState, Viewshed, WantsToMelee};
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
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToMelee>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            player_pos,
            player_entity,
            runstate,
            entities,
            mut viewshed,
            monster,
            mut position,
            mut wants_to_melee
        ) = data;

        for (ent, mut viewshed, _monster, mut pos)
                in (&entities, &mut viewshed, &monster, &mut position).join() {
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(
                Point::new(pos.x, pos.y), *player_pos);

            // If player is in melee range, initiate combat
            if distance < 1.5 {
                wants_to_melee
                    .insert(ent, WantsToMelee { target: *player_entity })
                    .expect("Unable to insert attack");
            } else if viewshed.visible_tiles.contains(&*player_pos) {
                // If player is visible, get path to them with A*.
                let path = rltk::a_star_search(
                    map.xy_idx(pos.x, pos.y) as i32,
                    map.xy_idx(player_pos.x, player_pos.y) as i32,
                    &mut *map
                );

                // If path is found, take a step and recalculate the viewshed.
                // `steps[0]` is the current position, so take the next step.
                if path.success && path.steps.len() > 1 {
                    let mut idx = map.xy_idx(pos.x, pos.y);
                    map.blocked[idx] = false;
                    pos.x = path.steps[1] as i32 % map.width;
                    pos.y = path.steps[1] as i32 / map.width;
                    idx = map.xy_idx(pos.x, pos.y);
                    map.blocked[idx] = true;
                    viewshed.dirty = true;
                }
            }

        }
    }
}
