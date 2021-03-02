use specs::prelude::*;
use super::{components::*, gamelog::GameLog, Map};

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player, mut log, mut wants_pickup, mut positions, names, mut backpack) = data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack
                .insert(pickup.item, InBackpack { owner: pickup.collected_by })
                .expect("Unable to insert into backpack");

            if pickup.collected_by == *player {
                log.entries.push(format!("You pick up the {}.",
                                            names.get(pickup.item).unwrap().name));
            }
        } wants_pickup.clear();
    }
}

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        ReadExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, InflictsDamage>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, AreaOfEffect>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_ent, mut log, map, entities, mut wants_use, names, consumables,
            healing, inflicts_damage, mut combat_stats, mut suffer, aoe) = data;

        for (ent, useitem) in (&entities, &wants_use).join() {
            let mut item_used = true;
            let mut targets: Vec<Entity> = Vec::new();

            match useitem.target {
                // If no target, target the player (eg, a potion).
                None => targets.push(*player_ent),
                // Else, there's at least one non-player target.
                Some(target) => {
                    // If the item's in AreaOfEffect storage, more than one target.
                    match aoe.get(useitem.item) {
                        // Not in AoE storage--target a single mob.
                        None => {
                            let idx = map.xy_idx(target.x, target.y);
                            for mob in map.tile_content[idx].iter() {
                                targets.push(*mob);
                            }
                        },
                        // In AoE--determine what's in radius of the item's effect.
                        Some(area_effect) => {
                            // Plot a viewshed from the centroid (target) of appropriate
                            // radius and constrain to only valid map tiles.
                            let mut blast_tiles =
                                rltk::field_of_view(target, area_effect.radius, &*map)
                                    .into_iter()
                                    .filter(|p| p.x > 0 && p.x < map.width-1
                                        && p.y > 0 && p.y < map.height-1)
                                    .collect::<Vec<_>>();

                            // Look at each tile in the area of effect; content
                            // of these tiles will be added to our targets.
                            for tile_idx in blast_tiles.iter() {
                                let idx = map.xy_idx(tile_idx.x, tile_idx.y);
                                for mob in map.tile_content[idx].iter() {
                                    targets.push(*mob);
                                }
                            }
                        }
                    }
                }
            }

            // Check to see if the item inflicts damage.
            match inflicts_damage.get(useitem.item) {
                None => {}
                // If so, apply damage to the targets we found.
                Some(damage) => {
                    let target_pos = useitem.target.unwrap();
                    let idx = map.xy_idx(target_pos.x, target_pos.y);
                    item_used = false;

                    // Apply damage to the targets.
                    for mob in targets.iter() {
                        SufferDamage::new_damage(&mut suffer, *mob, damage.damage);
                        if ent == *player_ent {
                            let mob_name = names.get(*mob).unwrap();
                            let item_name = names.get(useitem.item).unwrap();
                            log.entries.push(
                                format!("You use {} on {}, inflicting {} damage.",
                                        item_name.name, mob_name.name, damage.damage)
                            );
                        } item_used = true;
                    }
                }
            }

            // Check if the item provides healing.
            let item_heals = healing.get(useitem.item);
            match item_heals {
                None => {}
                // If so, apply healing to its target(s).
                Some(healer) => {
                    item_used = false;
                    for target in targets.iter() {
                        // Get the target's stats, so we can heal them.
                        if let Some(stats) = combat_stats.get_mut(*target) {
                            // Heals the target by the items healing amount, up to their max hp.
                            stats.hp = i32::min(stats.max_hp, stats.hp + healer.heal_amount);
                            if ent == *player_ent {
                                log.entries.push(format!("You drink the {}, healing {} hp.",
                                    names.get(useitem.item).unwrap().name,
                                    healer.heal_amount));
                            }
                            item_used = true;
                        }
                    }
                }
            }

            // Discard consumable items after they have been used.
            if item_used {
                let consumable = consumables.get(useitem.item);
                match consumable {
                    None => {}
                    Some(_) => {
                        entities.delete(useitem.item).expect("Delete failed");
                    }
                }
            }
        }
        wants_use.clear();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_ent, mut log, entities, mut wants_drop, names, mut positions, mut backpack) = data;

        for (ent, to_drop) in (&entities, &wants_drop).join() {
            let mut dropper_pos: Position = Position { x: 0, y: 0 };
            {
                let dropped_pos = positions.get(ent).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }
            positions
                .insert(to_drop.item, Position { x: dropper_pos.x, y: dropper_pos.y })
                .expect("Unable to insert position");
            backpack.remove(to_drop.item);

            if ent == *player_ent {
                log.entries.push(format!("You drop the {}.",
                    names.get(to_drop.item).unwrap().name));
            }
        }

        wants_drop.clear();
    }
}