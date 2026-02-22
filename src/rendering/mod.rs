mod composite;
mod rectangle;
mod renderer;

use rectangle::RectangleRenderer;
pub use rectangle::{Rectangle, RectangleId};

use composite::CompositeRenderer;

pub use renderer::Renderer;
