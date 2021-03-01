use rltk::{console};
use specs::prelude::*;
use super::{CombatStats, Name, SufferDamage, WantsToMelee};

/// Handle for our melee combat system.
pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (ent, mut melee, names, combat_stats, mut inflict_damage) = data;

        for (_ent, melee, name, stats) in (&ent, &melee, &names, &combat_stats).join() {
            if stats.hp > 0 {
                let target_stats = combat_stats.get(melee.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(melee.target).unwrap();
                    let damage = i32::max(0, stats.power - target_stats.defense);

                    if damage == 0 {
                        console::log(&format!("{} is left unscathed from {}!", &name.name, &target_name.name));
                    } else {
                        console::log(&format!("{} hits {} for {} hp.", &name.name, &target_name.name, damage));
                        SufferDamage::new_damage(&mut inflict_damage, melee.target, damage);
                    }
                }
            }
        } melee.clear();
    }
}