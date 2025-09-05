#[derive(Clone)]
pub struct GridAOI {
    cell: f32,
}
impl GridAOI {
    pub fn new(cell: f32) -> Self { Self { cell } }
    pub fn cell_key(&self, x:f32, z:f32) -> (i32,i32) {
        ((x / self.cell).floor() as i32, (z / self.cell).floor() as i32)
    }
}