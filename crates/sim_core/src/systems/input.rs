use hecs::World;
use crate::{Pos, Vel};

pub fn apply_input(world: &mut World, _id: u64, dx:f32, dy:f32, dt:f32) {
    let mut to_update = None;
    for (e, (pos, _vel)) in world.query_mut::<(&mut Pos, &mut Vel)>() {
        let _ = (pos, _vel);
        to_update = Some(e); break;
    }
    if let Some(e) = to_update {
        let mut q = world.query_one::<(&mut Pos, &mut Vel)>(e).unwrap();
        if let Some((pos, vel)) = q.get() {
            let max_speed = 6.0;
            let len = (dx*dx + dy*dy).sqrt();
            let (vx, vy) = if len > 0.0001 { (dx/len*max_speed, dy/len*max_speed) } else { (0.0,0.0) };
            pos.0 += vx * dt;
            pos.2 += vy * dt;
            vel.0 = vx; vel.2 = vy;
        }
    }
}