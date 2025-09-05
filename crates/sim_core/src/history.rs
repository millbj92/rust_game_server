#[derive(Clone, Copy)]
pub struct SnapshotPlayer { pub id: u64, pub x:f32, pub y:f32, pub z:f32, pub r:f32 }

#[derive(Clone)]
pub struct Snapshot { pub tick: u32, pub players: Vec<SnapshotPlayer> }

pub struct History { buf: std::collections::VecDeque<Snapshot>, cap: usize }
impl History {
    pub fn new(cap: usize) -> Self { Self { buf: std::collections::VecDeque::with_capacity(cap), cap } }
    pub fn push(&mut self, s: Snapshot) {
        if self.buf.len() == self.cap { self.buf.pop_front(); }
        self.buf.push_back(s);
    }
    pub fn nearest_at_or_before(&self, tick: u32) -> Option<&Snapshot> {
        self.buf.iter().rev().find(|s| s.tick <= tick)
    }
}