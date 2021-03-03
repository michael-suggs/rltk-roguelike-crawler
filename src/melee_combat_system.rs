use specs::prelude::*;
use super::{components::*, gamelog::GameLog};

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
        ) = data;

        for (ent, wants_melee, name, stats) in (&entities, &melee, &names, &combat_stats).join() {
            // If no HP, combat doesn't make much sense does it
            if stats.hp > 0 {
                // Get the offensive bonus offered by equipped items.
                let offense_bonus: i32 =
                    (&melee_power_bonuses, &equipped)
                        .join()
                        .filter(|(_, equipped_by)| { equipped_by.owner == ent })
                        .map(|(p, _)| p)
                        .fold(0, |acc, item| acc + item.power);

                let target_stats = combat_stats.get(wants_melee.target).unwrap();
                if target_stats.hp > 0 {
                    // Get defense bonus offered by equipped items.
                    let defense_bonus: i32 =
                        (&defense_bonuses, &equipped)
                            .join()
                            .filter(|(_, equipped_by)| { equipped_by.owner == wants_melee.target })
                            .map(|(d, _)| d)
                            .fold(0, |acc, item| acc + item.defense);

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
