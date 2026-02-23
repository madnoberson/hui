use bon::Builder;
use glam::{Mat4, Quat, Vec3};

use crate::{Rectangle, Renderer};
use block_position_markers::{Positioned, Unpositioned};

pub mod block_position_markers {
    use crate::RectangleId;

    pub struct Unpositioned;

    pub struct Positioned {
        pub(super) rectangle_id: RectangleId,
        pub(super) position:     [f32; 2],
    }
}

#[derive(Builder, Clone)]
#[builder(const)]
pub struct BlockStyle {
    #[builder(default = [1.0, 1.0, 1.0, 1.0])]
    fill_color:    [f32; 4],
    #[builder(default = [0.0, 0.0, 0.0, 0.0])]
    border_color:  [f32; 4],
    #[builder(default = [0.0, 0.0, 0.0, 0.0])]
    corner_radii:  [f32; 4],
    #[builder(default = 0.0)]
    border_size:   f32,
    #[builder(default = [0.0, 0.0, 0.0, 0.0])]
    shadow_color:  [f32; 4],
    #[builder(default = [0.0, 0.0])]
    shadow_offset: [f32; 2],
    #[builder(default = 0.0)]
    shadow_blur:   f32,
    #[builder(default = 0.0)]
    shadow_spread: f32,
}

pub struct Block<State = Unpositioned> {
    state: State,
    style: BlockStyle,
    size:  [f32; 2],
}

impl Block<Unpositioned> {
    #[must_use]
    #[inline(always)]
    pub const fn new(size: [f32; 2], style: BlockStyle) -> Self {
        Self { state: Unpositioned, style, size }
    }

    #[must_use]
    #[inline(always)]
    pub fn make_positioned(
        self,
        position: [f32; 2],
        view_projection: &Mat4,
        renderer: &mut Renderer,
    ) -> Block<Positioned> {
        Block::<Positioned>::new(
            position,
            self.size,
            self.style,
            view_projection,
            renderer,
        )
    }

    #[inline(always)]
    pub const fn set_size(&mut self, size: [f32; 2]) { self.size = size; }

    #[inline(always)]
    pub const fn set_style(&mut self, style: BlockStyle) {
        self.style = style;
    }
}

impl Block<Positioned> {
    #[must_use]
    pub fn new(
        position: [f32; 2],
        size: [f32; 2],
        style: BlockStyle,
        view_projection: &Mat4,
        renderer: &mut Renderer,
    ) -> Self {
        let rectangle =
            build_rectangle(view_projection, position, size, &style);
        let rectangle_id = renderer.add_rectangle(&rectangle);

        let state = Positioned { rectangle_id, position };
        Self { state, style, size }
    }

    pub fn set_size(
        &mut self,
        size: [f32; 2],
        view_projection: &Mat4,
        renderer: &mut Renderer,
    ) {
        if let Some(rectangle) =
            renderer.get_mut_rectangle(self.state.rectangle_id)
        {
            let (model, half_size) = build_model(size, self.state.position);
            let mvp = *view_projection * model;

            rectangle.mvp = mvp.to_cols_array_2d();
            rectangle.half_size = half_size;
        }
        self.size = size;
    }

    pub fn set_position(
        &mut self,
        position: [f32; 2],
        view_projection: &Mat4,
        renderer: &mut Renderer,
    ) {
        if let Some(rectangle) =
            renderer.get_mut_rectangle(self.state.rectangle_id)
        {
            let (model, half_size) = build_model(self.size, position);
            let mvp = *view_projection * model;

            rectangle.mvp = mvp.to_cols_array_2d();
            rectangle.half_size = half_size;
        }
        self.state.position = position;
    }

    pub fn set_size_and_position(
        &mut self,
        size: [f32; 2],
        position: [f32; 2],
        view_projection: &Mat4,
        renderer: &mut Renderer,
    ) {
        if let Some(rectangle) =
            renderer.get_mut_rectangle(self.state.rectangle_id)
        {
            let (model, half_size) = build_model(size, position);
            let mvp = *view_projection * model;

            rectangle.mvp = mvp.to_cols_array_2d();
            rectangle.half_size = half_size;
        }
        self.state.position = position;
        self.size = size;
    }

    pub fn set_style(&mut self, style: BlockStyle, renderer: &mut Renderer) {
        if let Some(rectangle) =
            renderer.get_mut_rectangle(self.state.rectangle_id)
        {
            rectangle.fill_color = style.fill_color;
            rectangle.border_color = style.border_color;
            rectangle.corner_radii = style.corner_radii;
            rectangle.shadow_color = style.shadow_color;
            rectangle.border_size = style.border_size;
            rectangle.shadow_spread = style.shadow_spread;
            rectangle.shadow_offset = style.shadow_offset;
            rectangle.shadow_blur = style.shadow_blur;
        }
        self.style = style;
    }

    #[inline(always)]
    pub fn destroy(&self, renderer: &mut Renderer) {
        renderer.remove_rectangle(self.state.rectangle_id);
    }
}

fn build_rectangle(
    view_projection: &Mat4,
    position: [f32; 2],
    size: [f32; 2],
    block_style: &BlockStyle,
) -> Rectangle {
    let (model, half_size) = build_model(size, position);
    let mvp = view_projection * model;

    Rectangle::builder()
        .mvp(mvp.to_cols_array_2d())
        .half_size(half_size)
        .fill_color(block_style.fill_color)
        .border_color(block_style.border_color)
        .corner_radii(block_style.corner_radii)
        .border_size(block_style.border_size)
        .shadow_color(block_style.shadow_color)
        .shadow_offset(block_style.shadow_offset)
        .shadow_blur(block_style.shadow_blur)
        .shadow_spread(block_style.shadow_spread)
        .build()
}

fn build_model(size: [f32; 2], position: [f32; 2]) -> (Mat4, [f32; 2]) {
    let half_size = [size[0] / 2.0, size[1] / 2.0];
    let center =
        Vec3::new(position[0] + half_size[0], position[1] + half_size[1], 0.0);
    let scale = Vec3::new(half_size[0], half_size[1], 1.0);
    let model =
        Mat4::from_scale_rotation_translation(scale, Quat::IDENTITY, center);

    (model, half_size)
}
