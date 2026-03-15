use wgpu::{
    BlendComponent, BlendFactor, BlendOperation, BlendState, Buffer,
    BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites, Device,
    FragmentState, FrontFace, IndexFormat, MultisampleState,
    PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology,
    Queue, RenderPass, RenderPipeline, RenderPipelineDescriptor,
    ShaderModuleDescriptor, ShaderSource, TextureFormat, VertexBufferLayout,
    VertexState, VertexStepMode,
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array,
};

use super::{Rectangle, RectangleId, RectangleStore};

const MAX_INSTANCE_COUNT: u64 = 1024;

#[rustfmt::skip]
const VERTICES: &[[f32; 3]; 4] = &[
    [-1.0,  1.0, 0.0],
    [-1.0, -1.0, 0.0],
    [ 1.0,  1.0, 0.0],
    [ 1.0, -1.0, 0.0],
];
#[rustfmt::skip]
const INDICES: &[u16; 6] = &[
    1, 0, 2,
    1, 3, 2,
];

pub struct RectangleRenderer {
    render_pipeline: RenderPipeline,
    vertex_buffer:   Buffer,
    index_buffer:    Buffer,
    instance_buffer: Buffer,
    instance_store:  RectangleStore,
}

impl RectangleRenderer {
    #[must_use]
    pub fn new(device: &Device, surface_format: TextureFormat) -> Self {
        let render_pipeline = create_render_pipeline(device, surface_format);

        let vertex_buffer_desc = BufferInitDescriptor {
            label:    Some("hui::rectangle::vertex_buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage:    BufferUsages::VERTEX,
        };
        let vertex_buffer = device.create_buffer_init(&vertex_buffer_desc);

        let index_buffer_desc = BufferInitDescriptor {
            label:    Some("hui::rectangle::index_buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage:    BufferUsages::INDEX,
        };
        let index_buffer = device.create_buffer_init(&index_buffer_desc);

        let instance_buffer_desc = BufferDescriptor {
            label:              Some("hui::rectangle::instance_buffer"),
            size:               MAX_INSTANCE_COUNT * Rectangle::SIZE as u64,
            usage:              BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };
        let instance_buffer = device.create_buffer(&instance_buffer_desc);

        Self {
            render_pipeline,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            instance_store: RectangleStore::new(),
        }
    }

    #[must_use]
    #[inline(always)]
    pub fn get_mut(&mut self, id: RectangleId) -> Option<&mut Rectangle> {
        self.instance_store.get_mut(id)
    }

    #[inline(always)]
    pub fn add(&mut self, instance: &Rectangle) -> RectangleId {
        self.instance_store.add(instance)
    }

    #[inline(always)]
    pub fn remove(&mut self, id: RectangleId) -> Option<Rectangle> {
        self.instance_store.remove(id)
    }

    pub fn render(&mut self, queue: &Queue, render_pass: &mut RenderPass) {
        if self.instance_store.is_empty() {
            return;
        }
        let instance_bytes = self.instance_store.bytes();

        render_pass.set_pipeline(&self.render_pipeline);
        queue.write_buffer(&self.instance_buffer, 0, instance_bytes);

        let vertex_buffer = self.vertex_buffer.slice(..);
        render_pass.set_vertex_buffer(0, vertex_buffer);

        let instance_buffer = self.instance_buffer.slice(..);
        render_pass.set_vertex_buffer(1, instance_buffer);

        let index_buffer = self.index_buffer.slice(..);
        render_pass.set_index_buffer(index_buffer, IndexFormat::Uint16);

        render_pass.draw_indexed(
            0..INDICES.len() as u32,
            0,
            0..self.instance_store.len() as u32,
        );
    }
}

fn create_render_pipeline(
    device: &Device,
    surface_format: TextureFormat,
) -> RenderPipeline {
    let shader_module_content =
        ShaderSource::Wgsl(include_str!("rectangle.wgsl").into());
    let shader_module_desc = ShaderModuleDescriptor {
        label:  Some("hui::rectangle::shader_module"),
        source: shader_module_content,
    };
    let shader_module = device.create_shader_module(shader_module_desc);

    let vertex_buffer_attributes = vertex_attr_array![
        0 => Float32x3,
    ];
    let vertex_buffer_layout = VertexBufferLayout {
        array_stride: size_of::<[f32; 3]>() as u64,
        step_mode:    VertexStepMode::Vertex,
        attributes:   &vertex_buffer_attributes,
    };

    let vertex_state = VertexState {
        module:              &shader_module,
        entry_point:         Some("vs_main"),
        compilation_options: Default::default(),
        buffers:             &[vertex_buffer_layout, Rectangle::LAYOUT],
    };

    let blend_state = BlendState {
        color: BlendComponent {
            src_factor: BlendFactor::SrcAlpha,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation:  BlendOperation::Add,
        },
        alpha: BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation:  BlendOperation::Add,
        },
    };
    let fragment_state_targets = [Some(ColorTargetState {
        format:     surface_format,
        blend:      Some(blend_state),
        write_mask: ColorWrites::ALL,
    })];
    let fragment_state = FragmentState {
        module:              &shader_module,
        entry_point:         Some("fs_main"),
        compilation_options: Default::default(),
        targets:             &fragment_state_targets,
    };

    let primitive_state = PrimitiveState {
        topology:           PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face:         FrontFace::Ccw,
        cull_mode:          None,
        polygon_mode:       PolygonMode::Fill,
        unclipped_depth:    false,
        conservative:       false,
    };
    let multisample_state = MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
    };

    let render_pipeline_layout_desc = PipelineLayoutDescriptor {
        label:                Some("hui::rectangle::render_pipeline_layout"),
        bind_group_layouts:   &[],
        push_constant_ranges: &[],
    };
    let render_pipeline_layout =
        device.create_pipeline_layout(&render_pipeline_layout_desc);

    let render_pipeline_desc = RenderPipelineDescriptor {
        label:         Some("hui::rectangle::render_pipeline"),
        layout:        Some(&render_pipeline_layout),
        vertex:        vertex_state,
        fragment:      Some(fragment_state),
        primitive:     primitive_state,
        depth_stencil: None,
        multisample:   multisample_state,
        multiview:     None,
        cache:         None,
    };
    device.create_render_pipeline(&render_pipeline_desc)
}
