use hecs::World;
use moonhold_protocol::EntityDelta;
use crate::{Pos, Health};

pub fn collect_deltas(world: &World) -> Vec<EntityDelta> {
    let mut out = Vec::new();
    for (_e, (pos, hp)) in world.query::<(&Pos, &Health)>().iter() {
        out.push(EntityDelta {
            id: _e.id() as u64,
            pos: [pos.0, pos.1, pos.2],
            hp: hp.hp,
        });
    }
    out
}