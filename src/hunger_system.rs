use specs::prelude::*;
use super::{gamelog::GameLog, HungerClock, HungerState, RunState, SufferDamage};

pub struct HungerSystem {}

impl<'a> System<'a> for HungerSystem {
    #[allow(clippy::clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, HungerClock>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut hunger_clock,
            player_ent,
            runstate,
            mut damage,
            mut log,
        ) = data;

        for (ent, mut clock) in (&entities, &mut hunger_clock).join() {
            let mut proceed = false;

            match *runstate {
                RunState::PlayerTurn => if ent == *player_ent { proceed = true; },
                RunState::MonsterTurn => if ent != *player_ent { proceed = true; },
                _ => proceed = false,
            }

            if proceed {
                clock.duration -= 1;
                if clock.duration < 1 {
                    match clock.state {
                        HungerState::WellFed => {
                            clock.state = HungerState::Normal;
                            clock.duration = 200;
                            if ent == *player_ent {
                                log.entries.push("You are no longer well fed.".to_string());
                            }
                        },
                        HungerState::Normal => {
                            clock.state = HungerState::Hungry;
                            clock.duration = 200;
                            if ent == *player_ent {
                                log.entries.push("You are hungry.".to_string());
                            }
                        },
                        HungerState::Hungry => {
                            clock.state = HungerState::Starving;
                            clock.duration = 200;
                            if ent == *player_ent {
                                log.entries.push("You are starving!".to_string());
                            }
                        },
                        HungerState::Starving => {
                            if ent == *player_ent {
                                log.entries.push(
                                    "Your hunger pangs are getting painful!".to_string()
                                );
                            }
                            SufferDamage::new_damage(&mut damage, ent, 1);
                        }
                    }
                }
            }
        }
    }
}
