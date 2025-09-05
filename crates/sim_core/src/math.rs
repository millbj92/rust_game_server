#[derive(Clone, Copy, Default)]
pub struct Vec3(pub f32, pub f32, pub f32);

pub fn norm(v: Vec3) -> Vec3 {
    let l = (v.0*v.0 + v.1*v.1 + v.2*v.2).sqrt().max(1e-6);
    Vec3(v.0/l, v.1/l, v.2/l)
}
pub fn ray_sphere(o: Vec3, d: Vec3, c: Vec3, r: f32) -> Option<f32> {
    let oc = Vec3(o.0-c.0, o.1-c.1, o.2-c.2);
    let b = oc.0*d.0 + oc.1*d.1 + oc.2*d.2;
    let cterm = oc.0*oc.0 + oc.1*oc.1 + oc.2*oc.2 - r*r;
    let disc = b*b - cterm;
    if disc < 0.0 { return None; }
    let t = -b - disc.sqrt();
    if t >= 0.0 { Some(t) } else {
        let t2 = -b + disc.sqrt();
        if t2 >= 0.0 { Some(t2) } else { None }
    }
}