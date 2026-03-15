use bon::Builder;
use bytemuck::{Pod, Zeroable};
use slotmap::DefaultKey;
use wgpu::{VertexBufferLayout, VertexStepMode, vertex_attr_array};

pub type RectangleId = DefaultKey;

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(Zeroable, Pod, Builder)]
#[builder(const)]
pub struct Rectangle {
    pub mvp:           [[f32; 4]; 4],
    pub fill_color:    [f32; 4],
    pub border_color:  [f32; 4],
    pub corner_radii:  [f32; 4],
    pub shadow_color:  [f32; 4],
    pub clip_rect:     [f32; 4],
    pub half_size:     [f32; 2],
    pub border_size:   f32,
    pub shadow_spread: f32,
    pub shadow_offset: [f32; 2],
    pub shadow_blur:   f32,

    #[doc(hidden)]
    #[builder(skip = 0.0)]
    _padding: f32,
}

impl Rectangle {
    pub(crate) const LAYOUT: VertexBufferLayout<'static> = {
        let instance_buffer_atributes = &vertex_attr_array![
            1  => Float32x4, // MVP matrix, row 0
            2  => Float32x4, // MVP matrix, row 1
            3  => Float32x4, // MVP matrix, row 2
            4  => Float32x4, // MVP matrix, row 3
            5  => Float32x4, // Fill color
            6  => Float32x4, // Border color
            7  => Float32x4, // Corner radii
            8  => Float32x4, // Shadow color
            9  => Float32x4, // Clip rect
            10 => Float32x2, // Half size
            11 => Float32,   // Border size
            12 => Float32,   // Shadow spread
            13 => Float32x2, // Shadow offset
            14 => Float32,   // Shadow blur
        ];
        VertexBufferLayout {
            array_stride: Self::SIZE as u64,
            step_mode:    VertexStepMode::Instance,
            attributes:   instance_buffer_atributes,
        }
    };
    pub const SIZE: usize = size_of::<Self>();
}
