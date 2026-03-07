use enum_dispatch::enum_dispatch;
use glam::Mat4;

use super::{
    Block,
    block_states::{Positioned, Unpositioned},
};
use crate::{
    components::common::{Boundable, Spawnable},
    core::Renderer,
};

impl Boundable for ! {
    #[cold]
    #[inline(never)]
    #[allow(unused_variables)]
    fn set_bounds(
        &mut self,
        position: [f32; 2],
        size: [f32; 2],
        view_projection: &Mat4,
        renderer: &mut Renderer,
    ) {
        unreachable!("NoBoundable::set_bounds should never be called")
    }
}

#[enum_dispatch(Boundable)]
pub enum BoundableWidget<'a, Custom: Boundable = !> {
    Block(&'a mut Block<Positioned>),
    Custom(&'a mut Custom),
}

impl Spawnable for ! {
    #[cold]
    #[inline(never)]
    #[allow(unused_variables)]
    fn spawn(
        self,
        position: [f32; 2],
        view_projection: &Mat4,
        renderer: &mut Renderer,
    ) -> impl Boundable {
        unreachable!("NoSpawnable::spawn should never be called")
    }
}

#[enum_dispatch(Spawnable)]
pub enum SpawnableWidget<'a, Custom: Spawnable = !> {
    Block(&'a mut Block<Unpositioned>),
    Custom(&'a mut Custom),
}
