use crate::{InflictsDamage, SingleActivation, SufferDamage};
use specs::prelude::*;

use super::{
    gamelog::GameLog, particle_system::ParticleBuilder, EntityMoved, EntryTrigger, Hidden, Map,
    Name, Position,
};

pub struct TriggerSystem {}

impl<'a> System<'a> for TriggerSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteStorage<'a, EntityMoved>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, EntryTrigger>,
        ReadStorage<'a, SingleActivation>,
        WriteStorage<'a, Hidden>,
        ReadStorage<'a, Name>,
        Entities<'a>,
        ReadStorage<'a, InflictsDamage>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, ParticleBuilder>,
        WriteExpect<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            map,
            mut ent_moved,
            position,
            entry_trigger,
            activation,
            mut hidden,
            names,
            entities,
            inflicts_damage,
            mut suffering,
            mut particle_builder,
            mut log,
        ) = data;

        let mut remove_entities: Vec<Entity> = Vec::new();
        for (ent, _, pos) in (&entities, &ent_moved, &position).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            map.tile_content[idx]
                .iter()
                .filter(|ent_id| ent != **ent_id)
                .for_each(|ent_id| match entry_trigger.get(*ent_id) {
                    None => {}
                    Some(_) => {
                        if let Some(name) = names.get(*ent_id) {
                            log.entries.push(format!("{} triggers!", &name.name));
                        }

                        hidden.remove(*ent_id);

                        if let Some(damage) = inflicts_damage.get(*ent_id) {
                            particle_builder.request(
                                pos.x,
                                pos.y,
                                rltk::RGB::named(rltk::ORANGE),
                                rltk::RGB::named(rltk::BLACK),
                                rltk::to_cp437('‼'),
                                200.0,
                            );
                            SufferDamage::new_damage(&mut suffering, ent, damage.damage);
                        }

                        if let Some(_) = activation.get(*ent_id) {
                            remove_entities.push(*ent_id);
                        }
                    }
                });
        }
        // Removed traps with single-activation.
        remove_entities.iter().for_each(|trap| {
            entities.delete(*trap).expect("Unable to delete trap.");
        });
        // Clear the list of moved entities.
        ent_moved.clear();
    }
}
