mod composite;
mod rectangle;
mod renderer;

use composite::CompositeRenderer;

use rectangle::RectangleRenderer;
pub use rectangle::{Rectangle, RectangleId};

pub use renderer::Renderer;
