use wgpu::{
    Color, CommandEncoder, Device, Extent3d, Operations, Queue,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, SurfaceConfiguration, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};

use super::{Rectangle, RectangleRenderer};

pub struct Renderer {
    depth_texture_view: TextureView,
    color_operations:   Operations<Color>,
    depth_operations:   Option<Operations<f32>>,
    rectangle_renderer: RectangleRenderer,
}

impl Renderer {
    #[must_use]
    pub fn new(
        device: &Device,
        surface_config: &SurfaceConfiguration,
        color_operations: Operations<Color>,
        depth_operations: Option<Operations<f32>>,
        max_rectangle_count: u64,
    ) -> Self {
        let depth_texture_view = create_depth_texture_view(
            device,
            surface_config.width,
            surface_config.height,
        );
        let rectangle_renderer = RectangleRenderer::new(
            device,
            surface_config.format,
            max_rectangle_count,
        );

        Self {
            depth_texture_view,
            color_operations,
            depth_operations,
            rectangle_renderer,
        }
    }

    pub fn resize(&mut self, device: &Device, width: u32, height: u32) {
        self.depth_texture_view =
            create_depth_texture_view(device, width, height);
    }

    pub fn add_rectangle(&mut self, instance: &Rectangle) {
        self.rectangle_renderer.add(instance);
    }

    pub fn render(
        &mut self,
        queue: &Queue,
        surface_texture_view: &TextureView,
        command_encoder: &mut CommandEncoder,
    ) {
        let color_attachment = RenderPassColorAttachment {
            view:           surface_texture_view,
            depth_slice:    None,
            resolve_target: None,
            ops:            self.color_operations,
        };
        let depth_stencil_attachment = RenderPassDepthStencilAttachment {
            view:        &self.depth_texture_view,
            depth_ops:   self.depth_operations,
            stencil_ops: None,
        };

        let render_pass_desc = RenderPassDescriptor {
            label:                    Some("hui::render_pass"),
            color_attachments:        &[Some(color_attachment)],
            depth_stencil_attachment: Some(depth_stencil_attachment),
            occlusion_query_set:      None,
            timestamp_writes:         None,
        };
        let mut render_pass =
            command_encoder.begin_render_pass(&render_pass_desc);

        self.rectangle_renderer.render(queue, &mut render_pass);
    }
}

fn create_depth_texture_view(
    device: &Device,
    width: u32,
    height: u32,
) -> TextureView {
    let texture_desc = TextureDescriptor {
        label:           Some("hui::depth_texture"),
        size:            Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count:    1,
        dimension:       TextureDimension::D2,
        format:          TextureFormat::Depth32Float,
        usage:           TextureUsages::RENDER_ATTACHMENT,
        view_formats:    &[],
    };
    let texture = device.create_texture(&texture_desc);
    let texture_view_desc = TextureViewDescriptor::default();

    texture.create_view(&texture_view_desc)
}
