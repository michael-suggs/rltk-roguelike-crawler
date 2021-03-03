use rltk::{console};
use specs::prelude::*;
use super::{CombatStats, GameLog, Name, Player, SufferDamage};

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut damage) = data;

        for (mut stats, damage) in (&mut stats, &damage).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();
        }

        damage.clear();
    }
}

/// Removes dead entities (those with <1 hp) from the world.
pub fn delete_the_dead(ecs: &mut World) {
    // Vector to hold out "dead bodies"
    let mut dead: Vec<Entity> = Vec::new();
    // Scoping to appease the almighty borrow-checker
    {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let players = ecs.read_storage::<Player>();
        let entities = ecs.entities();
        let names = ecs.read_storage::<Name>();
        let mut log = ecs.write_resource::<GameLog>();

        for (ent, stats) in (&entities, &combat_stats).join() {
            if stats.hp < 1 {
                // Make sure we don't delete the player (will crash the game)
                match players.get(ent) {
                    None => {
                        let victim_name = names.get(ent);
                        if let Some(victim_name) = victim_name {
                            log.entries.push(format!("{} is dead", &victim_name.name));
                        } dead.push(ent)
                    }
                    Some(_) => {}
                }
            }
        }
    }

    // Remove all dead entities from the world.
    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete");
    }
}
