mod rectangle;
mod rectangle_store;
mod renderer;

pub use rectangle::{Rectangle, RectangleId};
pub use renderer::RectangleRenderer;

#[cfg(feature = "bench")]
pub use rectangle_store::RectangleStore;
#[cfg(not(feature = "bench"))]
use rectangle_store::RectangleStore;
