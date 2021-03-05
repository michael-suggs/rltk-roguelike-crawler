use specs::prelude::*;
use super::{components::*, particle_system::ParticleBuilder, Map, RunState};
use rltk::{BLACK, MAGENTA, Point, RGB};

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
        WriteStorage<'a, Confusion>,
        WriteExpect<'a, ParticleBuilder>,
        WriteStorage<'a, EntityMoved>,
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
            mut wants_to_melee,
            mut confused,
            mut particle_builder,
            mut entity_moved,
        ) = data;

        // If it's not the monster's turn, immediately return.
        if *runstate != RunState::MonsterTurn { return; }

        // Else, do the AI.
        for (ent, mut viewshed, _monster, mut pos)
                in (&entities, &mut viewshed, &monster, &mut position).join() {
            // Check to see if the mob is confused.
            let mut can_act = true;
            if let Some(am_confused) = confused.get_mut(ent) {
                // If confused, decrement remaining turns to be confused.
                am_confused.turns -= 1;
                // If they're no longer confused, take them out of confused;
                // will be able to act on their next turn.
                if am_confused.turns < 1 { confused.remove(ent); }
                // Confused--can't act.
                can_act = false;
                // Play the confusion particle effect for each confused turn.
                particle_builder.request(
                    pos.x, pos.y, RGB::named(MAGENTA), RGB::named(BLACK),
                    rltk::to_cp437('?'), 200.0
                );
            }

            // If they're not confused, let them act as normal.
            if can_act {
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
                        entity_moved.insert(ent, EntityMoved {})
                                    .expect("Unable to insert marker");
                    }
                }
            }
        }
    }
}
