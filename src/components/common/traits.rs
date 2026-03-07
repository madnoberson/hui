use glam::Mat4;

use crate::Renderer;

pub trait Boundable {
    fn set_bounds(
        &mut self,
        position: [f32; 2],
        size: [f32; 2],
        view_projection: &Mat4,
        renderer: &mut Renderer,
    );
}

pub trait Spawnable {
    #[must_use]
    fn spawn(
        self,
        position: [f32; 2],
        view_projection: &Mat4,
        renderer: &mut Renderer,
    ) -> impl Boundable;
}

pub trait HasDesiredHeight {
    fn desired_height(&self) -> f32;
}

pub trait HasDesiredWidth {
    fn desired_width(&self) -> f32;
}
