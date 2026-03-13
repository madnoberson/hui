#[derive(Debug, Clone, PartialEq)]
pub struct Bounds {
    pub position:  [f32; 2],
    pub size:      [f32; 2],
    pub clip_rect: [f32; 4],
}

impl Bounds {
    #[must_use]
    #[inline(always)]
    pub const fn without_clip_rect(
        position: [f32; 2],
        size: [f32; 2],
    ) -> Bounds {
        Bounds { position, size, clip_rect: [0.0, 0.0, f32::MAX, f32::MAX] }
    }
}
