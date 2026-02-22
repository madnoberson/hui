use wgpu::{
    Color, CommandEncoder, Device, Extent3d, LoadOp, Operations, Queue,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, StoreOp, SurfaceConfiguration, Texture,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureView,
};

use super::{CompositeRenderer, Rectangle, RectangleId, RectangleRenderer};

pub struct Renderer {
    depth_texture_view:     TextureView,
    offscreen_texture:      Texture,
    offscreen_texture_view: TextureView,
    color_operations:       Operations<Color>,
    depth_operations:       Option<Operations<f32>>,
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
        depth_operations: Option<Operations<f32>>,
    ) -> Self {
        let depth_texture_view = create_depth_texture_view(
            device,
            surface_config.width,
            surface_config.height,
        );
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
            depth_texture_view,
            offscreen_texture,
            offscreen_texture_view,
            color_operations,
            depth_operations,
            rectangle_renderer,
            composite_renderer,
            is_redraw_required: true,
        }
    }

    #[inline(always)]
    pub fn resize(&mut self, device: &Device, width: u32, height: u32) {
        self.depth_texture_view =
            create_depth_texture_view(device, width, height);
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
            let color_attachment = RenderPassColorAttachment {
                view:           &self.offscreen_texture_view,
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
            self.is_redraw_required = false;
        }

        let color_operations =
            Operations { load: LoadOp::Load, store: StoreOp::Store };
        let color_attachment = RenderPassColorAttachment {
            view:           surface_texture_view,
            depth_slice:    None,
            resolve_target: None,
            ops:            color_operations,
        };
        let render_pass_desc = RenderPassDescriptor {
            label:                    Some("hui::composite_pass"),
            color_attachments:        &[Some(color_attachment)],
            depth_stencil_attachment: None,
            occlusion_query_set:      None,
            timestamp_writes:         None,
        };
        let mut render_pass =
            command_encoder.begin_render_pass(&render_pass_desc);

        self.composite_renderer.render(&mut render_pass);
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

    texture.create_view(&Default::default())
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
