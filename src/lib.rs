#![feature(never_type)]
#![feature(impl_trait_in_assoc_type)]
#![feature(maybe_uninit_array_assume_init)]

pub mod components;
pub mod core;

pub use components::{
    Block, BlockStyle, BoundableWidget, Bounds, DesiredSize, HasDesiredHeight,
    HasDesiredWidth, InputState, LayoutItem, MouseButtonState,
    SpawnableWidget, block_states, vertical_layout,
};
pub use core::{Rectangle, RectangleId, Renderer};
