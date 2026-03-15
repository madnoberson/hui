mod composite;
mod rectangle;
mod renderer;

use composite::CompositeRenderer;

use rectangle::RectangleRenderer;
#[cfg(feature = "bench")]
pub use rectangle::RectangleStore;
pub use rectangle::{Rectangle, RectangleId};

pub use renderer::Renderer;
