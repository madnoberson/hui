use wgpu::{
    Color, CommandEncoder, Device, Extent3d, LoadOp, Operations, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, StoreOp,
    SurfaceConfiguration, Texture, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureView,
};

use super::{CompositeRenderer, Rectangle, RectangleId, RectangleRenderer};

pub struct Renderer {
    offscreen_texture:      Texture,
    offscreen_texture_view: TextureView,
    color_operations:       Operations<Color>,
    rectangle_renderer:     RectangleRenderer,
    composite_renderer:     CompositeRenderer,
    is_redraw_required:     bool,
}

impl Renderer {
    #[must_use]
    pub fn new(
        device: &Device,
        surface_config: &SurfaceConfiguration,
        color_operations: Operations<Color>,
    ) -> Self {
        let (offscreen_texture, offscreen_texture_view) =
            create_offscreen_texture(
                device,
                surface_config.width,
                surface_config.height,
                surface_config.format,
            );

        let rectangle_renderer =
            RectangleRenderer::new(device, surface_config.format);
        let composite_renderer = CompositeRenderer::new(
            device,
            surface_config.format,
            &offscreen_texture_view,
        );

        Self {
            offscreen_texture,
            offscreen_texture_view,
            color_operations,
            rectangle_renderer,
            composite_renderer,
            is_redraw_required: true,
        }
    }

    #[inline(always)]
    pub fn resize(&mut self, device: &Device, width: u32, height: u32) {
        (self.offscreen_texture, self.offscreen_texture_view) =
            create_offscreen_texture(
                device,
                width,
                height,
                self.offscreen_texture.format(),
            );

        self.composite_renderer
            .update_bind_group(device, &self.offscreen_texture_view);
        self.is_redraw_required = true;
    }

    #[must_use]
    #[inline(always)]
    pub fn get_mut_rectangle(
        &mut self,
        id: RectangleId,
    ) -> Option<&mut Rectangle> {
        self.rectangle_renderer.get_mut(id)
    }

    #[inline(always)]
    pub fn add_rectangle(&mut self, instance: &Rectangle) -> RectangleId {
        self.rectangle_renderer.add(instance)
    }

    #[inline(always)]
    pub fn remove_rectangle(&mut self, id: RectangleId) -> Option<Rectangle> {
        self.rectangle_renderer.remove(id)
    }

    pub fn render(
        &mut self,
        queue: &Queue,
        surface_texture_view: &TextureView,
        command_encoder: &mut CommandEncoder,
    ) {
        if self.is_redraw_required
            || self.rectangle_renderer.is_redraw_required()
        {
            let color_operations = Operations {
                load:  self.color_operations.load,
                store: StoreOp::Store,
            };
            let color_attachment = RenderPassColorAttachment {
                view:           &self.offscreen_texture_view,
                depth_slice:    None,
                resolve_target: None,
                ops:            color_operations,
            };
            let render_pass_desc = RenderPassDescriptor {
                label:                    Some("hui::render_pass"),
                color_attachments:        &[Some(color_attachment)],
                depth_stencil_attachment: None,
                occlusion_query_set:      None,
                timestamp_writes:         None,
            };
            let mut render_pass =
                command_encoder.begin_render_pass(&render_pass_desc);

            self.rectangle_renderer.render(queue, &mut render_pass);
            self.is_redraw_required = false;
        }

        let color_operations = Operations {
            load:  LoadOp::Load,
            store: self.color_operations.store,
        };
        let color_attachment = RenderPassColorAttachment {
            view:           surface_texture_view,
            depth_slice:    None,
            resolve_target: None,
            ops:            color_operations,
        };
        let composite_render_pass_desc = RenderPassDescriptor {
            label:                    Some("hui::composite_pass"),
            color_attachments:        &[Some(color_attachment)],
            depth_stencil_attachment: None,
            occlusion_query_set:      None,
            timestamp_writes:         None,
        };
        let mut composite_render_pass =
            command_encoder.begin_render_pass(&composite_render_pass_desc);

        self.composite_renderer.render(&mut composite_render_pass);
    }
}

fn create_offscreen_texture(
    device: &Device,
    width: u32,
    height: u32,
    format: TextureFormat,
) -> (Texture, TextureView) {
    let texture_desc = TextureDescriptor {
        label: Some("hui::offscreen_texture"),
        size: Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format,
        usage: TextureUsages::RENDER_ATTACHMENT
            | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    };
    let texture = device.create_texture(&texture_desc);
    let texture_view = texture.create_view(&Default::default());

    (texture, texture_view)
}
