use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMsg {
    Ping { client_time_ms: u64 },
    Input { seq: u32, dt_ms: u16, dx: f32, dy: f32 },
    Fire  { seq: u32, client_time_ms: u64, origin: [f32;3], dir: [f32;3], weapon: u16 },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMsg {
    Welcome { tick_hz: u16, your_id: u64 },
    Pong    { server_time_ms: u64 },
    WorldDelta { tick: u32, last_processed_input: u32, ents: Vec<EntityDelta> },
    HitConfirm { shooter: u64, victim: u64, dmg: u16, at_tick: u32 },
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct EntityDelta {
    pub id: u64,
    pub pos: [f32;3],
    pub hp:  u16,
}