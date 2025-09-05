use hecs::World;
use crate::{Pos, Vel};

pub fn integrate(world: &mut World, dt:f32) {
    for (_, (pos, vel)) in world.query_mut::<(&mut Pos, &mut Vel)>() {
        pos.0 += vel.0 * dt;
        pos.1 += vel.1 * dt;
        pos.2 += vel.2 * dt;
    }
}