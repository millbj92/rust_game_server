use hecs::World;
use crate::{math::{Vec3, norm, ray_sphere}, Pos, Collider};

pub struct FireEvent {
    pub shooter_id: u64,
    pub origin: [f32;3],
    pub dir: [f32;3],
    pub range: f32,
}
pub enum Hit { None, Victim(u64) }

pub fn hitscan(world: &World, evt: &FireEvent) -> Hit {
    let o = Vec3(evt.origin[0], evt.origin[1], evt.origin[2]);
    let d = norm(Vec3(evt.dir[0], evt.dir[1], evt.dir[2]));
    let mut best: Option<(u64,f32)> = None;
    for (_e, (pos, col)) in world.query::<(&Pos, &Collider)>().iter() {
        let c = Vec3(pos.0, pos.1, pos.2);
        if let Some(t) = ray_sphere(o, d, c, col.radius) {
            if t >= 0.0 && t <= evt.range {
                if best.map_or(true, |(_,bt)| t < bt) { best = Some((_e.id() as u64, t)); }
            }
        }
    }
    best.map_or(Hit::None, |(id,_)| Hit::Victim(id))
}