use rltk::RGB;
use specs::prelude::*;
use super::{Rltk, ParticleLifetime, Position, Renderable};

/// Requests a new particle with defined attributes.
struct ParticleRequest {
    x: i32,
    y: i32,
    fg: RGB,
    bg: RGB,
    glyph: rltk::FontCharType,
    lifetime: f32,
}

/// Builds particles from a vector of `ParticleRequest`.
pub struct ParticleBuilder {
    requests: Vec<ParticleRequest>,
}

impl ParticleBuilder {
    #[allow(clippy::clippy::new_without_default)]
    pub fn new() -> ParticleBuilder {
        ParticleBuilder { requests: Vec::new() }
    }

    pub fn request(&mut self, x: i32, y: i32, fg: RGB, bg: RGB,
                   glyph: rltk::FontCharType, lifetime: f32) {
        self.requests.push(ParticleRequest { x, y, fg, bg, glyph, lifetime });
    }
}

pub struct ParticleSpawnSystem {}

impl<'a> System<'a> for ParticleSpawnSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, ParticleLifetime>,
        WriteExpect<'a, ParticleBuilder>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut positions,
            mut renders,
            mut particles,
            mut builder,
        ) = data;

        // Spawn particles stored in the particle builder resource.
        for new_particle in builder.requests.iter() {
            // Make an entity for the new particle.
            let p = entities.create();
            // Give it a position.
            positions.insert(p, Position { x: new_particle.x, y: new_particle.y })
                     .expect("Unable to insert position");
            // Make it renderable.
            renders.insert(p, Renderable {
                fg: new_particle.fg, bg: new_particle.bg,
                glyph: new_particle.glyph, render_order: 0
            }).expect("Unable to insert renderable");
            // Give it a lifetime.
            particles.insert(p, ParticleLifetime { lifetime_ms: new_particle.lifetime })
                     .expect("Unable to insert lifetime");
        }
        // All requested particles made; clear the queue.
        builder.requests.clear();
    }
}

pub fn cull_dead_particles(ecs: &mut World, ctx: &Rltk) {
    let mut dead_particles: Vec<Entity> = Vec::new();
    {
        let mut particles = ecs.write_storage::<ParticleLifetime>();
        let entities = ecs.entities();
        for (ent, pt) in (&entities, &mut particles).join() {
            pt.lifetime_ms -= ctx.frame_time_ms;
            if pt.lifetime_ms < 0.0 {
                dead_particles.push(ent);
            }
        }
    }
    for dead in dead_particles.iter() {
        ecs.delete_entity(*dead).expect("Particle will not die");
    }
}
