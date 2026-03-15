use bon::Builder;
use glam::{Mat4, Quat, Vec3};

use crate::{
    components::common::{Bounds, InputState, MouseButtonState},
    core::{Rectangle, Renderer},
};
use block_states::{Positioned, Unpositioned};

pub mod block_states {
    use crate::{Bounds, RectangleId};

    pub struct Unpositioned;

    pub struct Positioned {
        pub(super) rectangle_id: RectangleId,
        pub(super) bounds:       Bounds,
    }
}

#[derive(Clone, Builder)]
#[builder(const)]
pub struct BlockStyle {
    #[builder(default = [1.0, 1.0, 1.0, 1.0])]
    fill_color:    [f32; 4],
    #[builder(default = [0.0, 0.0, 0.0, 0.0])]
    corner_radii:  [f32; 4],
    #[builder(default = [0.0, 0.0, 0.0, 0.0])]
    border_color:  [f32; 4],
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
    #[builder(default = [1.0, 1.0, 1.0, 1.0])]
    outline_color: [f32; 4],
    #[builder(default = 0.0)]
    outline_size:  f32,
}

#[derive(Clone)]
pub struct Block<State = Unpositioned> {
    state: State,
    style: BlockStyle,
}

impl Block<Unpositioned> {
    #[must_use]
    #[inline(always)]
    pub const fn new(style: BlockStyle) -> Self {
        Self { state: Unpositioned, style }
    }

    #[must_use]
    #[inline(always)]
    pub fn make_positioned(
        self,
        bounds: Bounds,
        view_projection: &Mat4,
        dpr: f32,
        renderer: &mut Renderer,
    ) -> Block<Positioned> {
        Block::<Positioned>::new(
            bounds,
            self.style,
            view_projection,
            dpr,
            renderer,
        )
    }

    #[inline(always)]
    pub const fn set_style(&mut self, style: BlockStyle) {
        self.style = style;
    }
}

impl Block<Positioned> {
    #[must_use]
    pub fn new(
        bounds: Bounds,
        style: BlockStyle,
        view_projection: &Mat4,
        dpr: f32,
        renderer: &mut Renderer,
    ) -> Self {
        let rectangle = build_rectangle(view_projection, dpr, &bounds, &style);
        let rectangle_id = renderer.add_rectangle(&rectangle);

        let state = Positioned { rectangle_id, bounds: bounds };
        Self { state, style }
    }

    #[must_use]
    #[inline(always)]
    pub const fn position(&self) -> [f32; 2] { self.state.bounds.position }

    #[must_use]
    #[inline(always)]
    pub const fn size(&self) -> [f32; 2] { self.state.bounds.size }

    #[must_use]
    #[inline(always)]
    pub const fn clip_rect(&self) -> [f32; 4] { self.state.bounds.clip_rect }

    #[inline(always)]
    const fn set_position(&mut self, position: [f32; 2]) {
        self.state.bounds.position = position;
    }

    #[inline(always)]
    const fn set_size(&mut self, size: [f32; 2]) {
        self.state.bounds.size = size;
    }

    #[inline(always)]
    const fn set_clip_rect(&mut self, clip_rect: [f32; 4]) {
        self.state.bounds.clip_rect = clip_rect;
    }

    pub fn update_size(
        &mut self,
        size: [f32; 2],
        view_projection: &Mat4,
        dpr: f32,
        renderer: &mut Renderer,
    ) {
        if let Some(rectangle) =
            renderer.get_mut_rectangle(self.state.rectangle_id)
        {
            let (model, half_size) = build_model(size, self.position(), dpr);
            let mvp = *view_projection * model;

            rectangle.mvp = mvp.to_cols_array_2d();
            rectangle.sizes = [
                half_size[0],
                half_size[1],
                self.style.shadow_offset[0],
                self.style.shadow_offset[1],
            ];
        }
        self.set_size(size);
    }

    pub fn update_position(
        &mut self,
        position: [f32; 2],
        view_projection: &Mat4,
        dpr: f32,
        renderer: &mut Renderer,
    ) {
        if let Some(rectangle) =
            renderer.get_mut_rectangle(self.state.rectangle_id)
        {
            let (model, half_size) = build_model(self.size(), position, dpr);
            let mvp = *view_projection * model;

            rectangle.mvp = mvp.to_cols_array_2d();
            rectangle.sizes = [
                half_size[0],
                half_size[1],
                self.style.shadow_offset[0],
                self.style.shadow_offset[1],
            ];
        }
        self.set_position(position);
    }

    pub fn update_clip_rect(
        &mut self,
        clip_rect: &[f32; 4],
        renderer: &mut Renderer,
    ) {
        if let Some(rectangle) =
            renderer.get_mut_rectangle(self.state.rectangle_id)
        {
            rectangle.clip_rect = *clip_rect;
        }
        self.set_clip_rect(*clip_rect);
    }

    pub fn update_style(
        &mut self,
        style: BlockStyle,
        renderer: &mut Renderer,
    ) {
        if let Some(rectangle) =
            renderer.get_mut_rectangle(self.state.rectangle_id)
        {
            rectangle.fill_color = style.fill_color;
            rectangle.border_color = style.border_color;
            rectangle.corner_radii = style.corner_radii;
            rectangle.shadow_color = style.shadow_color;
            rectangle.outline_color = style.outline_color;
            rectangle.sizes = [
                style.border_size,
                style.shadow_spread,
                style.shadow_blur,
                style.outline_size,
            ];
        }
        self.style = style;
    }

    #[inline(always)]
    pub fn destroy(&self, renderer: &mut Renderer) {
        renderer.remove_rectangle(self.state.rectangle_id);
    }

    #[must_use]
    #[inline(always)]
    pub const fn contains(&self, position: [f32; 2]) -> bool {
        let [x, y] = self.position();
        let [height, width] = self.size();

        position[0] >= x
            && position[0] <= x + width
            && position[1] >= y
            && position[1] <= y + height
    }

    #[must_use]
    #[inline(always)]
    pub fn is_pressed(&self, input_state: &InputState) -> bool {
        matches!(input_state.left_mouse_button(), MouseButtonState::Down)
            && input_state
                .mouse_position()
                .is_some_and(|pos| self.contains(pos))
    }
}

fn build_rectangle(
    view_projection: &Mat4,
    dpr: f32,
    bounds: &Bounds,
    block_style: &BlockStyle,
) -> Rectangle {
    let (model, half_size) = build_model(bounds.size, bounds.position, dpr);
    let mvp = view_projection * model;

    let rect_and_shadow = [
        half_size[0],
        half_size[1],
        block_style.shadow_offset[0],
        block_style.shadow_offset[1],
    ];
    let sizes = [
        block_style.border_size,
        block_style.shadow_spread,
        block_style.shadow_blur,
        block_style.outline_size,
    ];
    Rectangle::builder()
        .mvp(mvp.to_cols_array_2d())
        .fill_color(block_style.fill_color)
        .border_color(block_style.border_color)
        .corner_radii(block_style.corner_radii)
        .shadow_color(block_style.shadow_color)
        .outline_color(block_style.outline_color)
        .clip_rect(bounds.clip_rect)
        .rect_and_shadow(rect_and_shadow)
        .sizes(sizes)
        .build()
}

fn build_model(
    size: [f32; 2],
    position: [f32; 2],
    dpr: f32,
) -> (Mat4, [f32; 2]) {
    let size = [size[0] * dpr, size[1] * dpr];
    let position = [position[0] * dpr, position[1] * dpr];

    let half_size = [size[0] / 2.0, size[1] / 2.0];
    let center =
        Vec3::new(position[0] + half_size[0], position[1] + half_size[1], 0.0);
    let scale = Vec3::new(half_size[0], half_size[1], 1.0);
    let model =
        Mat4::from_scale_rotation_translation(scale, Quat::IDENTITY, center);

    (model, half_size)
}
