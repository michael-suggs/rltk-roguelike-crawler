use rltk::{BLACK, ORANGE, RGB};
use specs::prelude::*;
use super::{components::*, gamelog::GameLog, Position,
            particle_system::ParticleBuilder};

/// Handle for our melee combat system.
pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, MeleePowerBonus>,
        ReadStorage<'a, DefenseBonus>,
        ReadStorage<'a, Equipped>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, HungerClock>,
    );

    fn run(&mut self, data: Self::SystemData) {
        #[allow(clippy::type_complexity)]
        let (
            entities,
            mut log,
            mut melee,
            names,
            combat_stats,
            mut inflict_damage,
            melee_power_bonuses,
            defense_bonuses,
            equipped,
            mut particle_builder,
            positions,
            hunger_clock,
        ) = data;

        for (ent, wants_melee, name, stats) in (&entities, &melee, &names, &combat_stats).join() {
            // If no HP, combat doesn't make much sense does it
            if stats.hp > 0 {
                // Get the offensive bonus offered by equipped items.
                let mut offense_bonus: i32 =
                    (&melee_power_bonuses, &equipped)
                        .join()
                        .filter(|(_, equipped_by)| { equipped_by.owner == ent })
                        .map(|(p, _)| p)
                        .fold(0, |acc, item| acc + item.power);

                // Give a power bonus for being well fed.
                if let Some(hc) = hunger_clock.get(ent) {
                    if hc.state == HungerState::WellFed { offense_bonus += 1; }
                }

                let target_stats = combat_stats.get(wants_melee.target).unwrap();
                if target_stats.hp > 0 {
                    // Get defense bonus offered by equipped items.
                    let defense_bonus: i32 =
                        (&defense_bonuses, &equipped)
                            .join()
                            .filter(|(_, equipped_by)| { equipped_by.owner == wants_melee.target })
                            .map(|(d, _)| d)
                            .fold(0, |acc, item| acc + item.defense);

                    // Render some particles to denote combat is ongoing.
                    if let Some(pos) = positions.get(wants_melee.target) {
                        particle_builder.request(
                            pos.x, pos.y, RGB::named(ORANGE), RGB::named(BLACK),
                            rltk::to_cp437('‼'), 200.0
                        );
                    }

                    // Calculate damage, accounting for equipment bonuses.
                    let damage = i32::max(
                        0, (stats.power + offense_bonus) - (target_stats.defense + defense_bonus)
                    );

                    // Deal the damage and write it to the log.
                    let target_name = names.get(wants_melee.target).unwrap();
                    if damage == 0 {
                        log.entries.push(format!("{} is left unscathed from {}'s attack!",
                                         &target_name.name, &name.name));
                    } else {
                        log.entries.push(format!("{} hits {} for {} hp.", &name.name, &target_name.name, damage));
                        SufferDamage::new_damage(&mut inflict_damage, wants_melee.target, damage);
                    }
                }
            }
        } melee.clear();
    }
}
