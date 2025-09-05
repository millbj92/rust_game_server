use hecs::World;
use crate::history::History;

pub struct SimState {
    pub tick: u32,
    pub history: History,
    pub max_players: usize,
    pub tick_hz: u16,
}
impl SimState {
    pub fn new(tick_hz: u16, hist_ticks: usize, max_players: usize) -> Self {
        Self { tick:0, history: History::new(hist_ticks), max_players, tick_hz }
    }
}

pub struct Scene {
    pub world: World,
}
impl Scene {
    pub fn new() -> Self { Self { world: World::new() } }
}