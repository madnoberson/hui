use bon::Builder;
use bytemuck::{Pod, Zeroable};
use slotmap::{DefaultKey, SlotMap};
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

pub type RectangleId = DefaultKey;

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

#[repr(C, align(16))]
#[derive(Clone, Copy, Zeroable, Pod, Builder)]
pub struct Rectangle {
    pub mvp:           [[f32; 4]; 4],
    pub fill_color:    [f32; 4],
    pub border_color:  [f32; 4],
    pub corner_radii:  [f32; 4],
    pub shadow_color:  [f32; 4],
    pub half_size:     [f32; 2],
    pub border_size:   f32,
    pub shadow_spread: f32,
    pub shadow_offset: [f32; 2],
    pub shadow_blur:   f32,

    #[doc(hidden)]
    #[builder(skip)]
    _padding: f32,
}

impl Rectangle {
    pub(crate) const LAYOUT: VertexBufferLayout<'static> = {
        let instance_buffer_atributes = &vertex_attr_array![
            1 => Float32x4,  // MVP matrix, row 0
            2 => Float32x4,  // MVP matrix, row 1
            3 => Float32x4,  // MVP matrix, row 2
            4 => Float32x4,  // MVP matrix, row 3
            5 => Float32x4,  // Fill color
            6 => Float32x4,  // Border color
            7 => Float32x4,  // Corner radii
            8 => Float32x4,  // Shadow color
            9 => Float32x2,  // Half size
            10 => Float32,   // Border size
            11 => Float32,   // Shadow spread
            12 => Float32x2, // Shadow offset
            13 => Float32,   // Shadow blur
        ];
        VertexBufferLayout {
            array_stride: Self::SIZE as u64,
            step_mode:    VertexStepMode::Instance,
            attributes:   instance_buffer_atributes,
        }
    };
    pub const SIZE: usize = size_of::<Self>();
}

#[derive(PartialEq, Eq)]
enum Dirtiness {
    Clean,
    RedrawRequired,
    RebuildAndRedrawRequired,
}

pub(crate) struct RectangleRenderer {
    render_pipeline: RenderPipeline,
    vertex_buffer:   Buffer,
    index_buffer:    Buffer,
    instance_buffer: Buffer,
    instances:       SlotMap<RectangleId, Rectangle>,
    instance_bytes:  Vec<u8>,
    dirtiness:       Dirtiness,
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
            instances: SlotMap::new(),
            instance_buffer,
            instance_bytes: Vec::new(),
            dirtiness: Dirtiness::Clean,
        }
    }

    #[must_use]
    #[inline(always)]
    pub fn is_redraw_required(&self) -> bool {
        self.dirtiness != Dirtiness::Clean
    }

    #[must_use]
    #[inline(always)]
    pub fn get_mut(&mut self, id: RectangleId) -> Option<&mut Rectangle> {
        self.dirtiness = Dirtiness::RebuildAndRedrawRequired;
        self.instances.get_mut(id)
    }

    pub fn add(&mut self, instance: &Rectangle) -> RectangleId {
        if self.dirtiness != Dirtiness::RebuildAndRedrawRequired {
            self.dirtiness = Dirtiness::RedrawRequired;
        }
        let id = self.instances.insert(*instance);

        let new_instance_bytes = bytemuck::bytes_of(instance);
        self.instance_bytes.extend_from_slice(new_instance_bytes);

        id
    }

    #[inline(always)]
    pub fn remove(&mut self, id: RectangleId) -> Option<Rectangle> {
        self.dirtiness = Dirtiness::RebuildAndRedrawRequired;
        self.instances.remove(id)
    }

    pub fn render(&mut self, queue: &Queue, render_pass: &mut RenderPass) {
        if self.instances.is_empty() {
            return;
        }

        if self.dirtiness == Dirtiness::RebuildAndRedrawRequired {
            self.instance_bytes.clear();

            let instance_bytes_iter =
                self.instances.values().flat_map(bytemuck::bytes_of);
            self.instance_bytes.extend(instance_bytes_iter);
        }

        let bytes_written = self.instances.len() * Rectangle::SIZE;
        render_pass.set_pipeline(&self.render_pipeline);

        queue.write_buffer(
            &self.instance_buffer,
            0,
            &self.instance_bytes[..bytes_written],
        );

        let vertex_buffer = self.vertex_buffer.slice(..);
        render_pass.set_vertex_buffer(0, vertex_buffer);

        let instance_buffer =
            self.instance_buffer.slice(..bytes_written as u64);
        render_pass.set_vertex_buffer(1, instance_buffer);

        let index_buffer = self.index_buffer.slice(..);
        render_pass.set_index_buffer(index_buffer, IndexFormat::Uint16);

        render_pass.draw_indexed(
            0..INDICES.len() as u32,
            0,
            0..self.instances.len() as u32,
        );
        self.dirtiness = Dirtiness::Clean
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
