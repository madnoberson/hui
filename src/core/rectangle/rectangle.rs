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
    pub mvp:             [[f32; 4]; 4],
    pub fill_color:      [f32; 4],
    pub border_color:    [f32; 4],
    pub shadow_color:    [f32; 4],
    pub outline_color:   [f32; 4],
    pub corner_radii:    [f32; 4],
    pub clip_rect:       [f32; 4],
    // half_size.x, half_size.y, shadow_offset.x, shadow_offset.y
    pub rect_and_shadow: [f32; 4],
    // border_size, shadow_spread, shadow_blur, outline_size
    pub sizes:           [f32; 4],
}

impl Rectangle {
    pub(crate) const LAYOUT: VertexBufferLayout<'static> = {
        let instance_buffer_atributes = &vertex_attr_array![
            1  => Float32x4, // mvp matrix, row 0
            2  => Float32x4, // mvp matrix, row 1
            3  => Float32x4, // mvp matrix, row 2
            4  => Float32x4, // mvp matrix, row 3
            5  => Float32x4, // fill color
            6  => Float32x4, // border color
            7  => Float32x4, // shadow color
            8  => Float32x4, // outline color
            9  => Float32x4, // corner radii
            10 => Float32x4, // clip rect
            11 => Float32x4, // half_size.xy, shadow_offset.zw
            12 => Float32x4, // border_size, shadow_spread, shadow_blur, outline_size
        ];
        VertexBufferLayout {
            array_stride: Self::SIZE as u64,
            step_mode:    VertexStepMode::Instance,
            attributes:   instance_buffer_atributes,
        }
    };
    pub const SIZE: usize = size_of::<Self>();
}
