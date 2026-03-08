#[derive(Debug, Clone, PartialEq)]
pub struct Bounds {
    pub position:  [f32; 2],
    pub size:      [f32; 2],
    pub clip_rect: [f32; 4],
}
