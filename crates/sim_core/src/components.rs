use bitflags::bitflags;

#[derive(Clone, Copy, Debug, Default)]
pub struct Pos(pub f32, pub f32, pub f32);

#[derive(Clone, Copy, Debug, Default)]
pub struct Vel(pub f32, pub f32, pub f32);

#[derive(Clone, Copy, Debug)]
pub struct Health { pub hp: u16, pub max: u16 }
impl Default for Health { fn default() -> Self { Self { hp:100, max:100 } } }

#[derive(Clone, Copy, Debug)]
pub struct PlayerTag { pub id: u64 }

#[derive(Clone, Copy, Debug)]
pub struct Collider { pub radius: f32 }

bitflags! {
    pub struct DirtyMask: u8 { const POS = 1<<0; const HP = 1<<1; }
}