use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry,
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingResource, BindingType, BlendComponent, BlendFactor, BlendOperation,
    BlendState, ColorTargetState, ColorWrites, Device, FilterMode,
    FragmentState, PipelineLayoutDescriptor, PrimitiveState,
    PrimitiveTopology, RenderPass, RenderPipeline, RenderPipelineDescriptor,
    Sampler, SamplerDescriptor, ShaderModuleDescriptor, ShaderSource,
    ShaderStages, TextureFormat, TextureSampleType, TextureView,
    TextureViewDimension, VertexState,
};

pub(crate) struct CompositeRenderer {
    render_pipeline:   RenderPipeline,
    bind_group_layout: BindGroupLayout,
    bind_group:        BindGroup,
    sampler:           Sampler,
}

impl CompositeRenderer {
    #[must_use]
    pub fn new(
        device: &Device,
        surface_format: TextureFormat,
        offscreen_texture_view: &TextureView,
    ) -> Self {
        let sample_desc = SamplerDescriptor {
            label: Some("hui::composite::sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            ..Default::default()
        };
        let sampler = device.create_sampler(&sample_desc);

        let bind_group_layout_entries = [
            BindGroupLayoutEntry {
                binding:    0,
                visibility: ShaderStages::FRAGMENT,
                ty:         BindingType::Texture {
                    sample_type:    TextureSampleType::Float {
                        filterable: true,
                    },
                    view_dimension: TextureViewDimension::D2,
                    multisampled:   false,
                },
                count:      None,
            },
            BindGroupLayoutEntry {
                binding:    1,
                visibility: ShaderStages::FRAGMENT,
                ty:         BindingType::Sampler(
                    wgpu::SamplerBindingType::Filtering,
                ),
                count:      None,
            },
        ];
        let bind_group_layout_desc = BindGroupLayoutDescriptor {
            label:   Some("hui::composite::bind_group_layout"),
            entries: &bind_group_layout_entries,
        };
        let bind_group_layout =
            device.create_bind_group_layout(&bind_group_layout_desc);

        let bind_group = create_bind_group(
            device,
            &bind_group_layout,
            offscreen_texture_view,
            &sampler,
        );

        let render_pipeline =
            create_render_pipeline(device, surface_format, &bind_group_layout);

        Self { render_pipeline, bind_group_layout, bind_group, sampler }
    }

    pub fn update_bind_group(
        &mut self,
        device: &Device,
        offscreen_texture_view: &TextureView,
    ) {
        self.bind_group = create_bind_group(
            device,
            &self.bind_group_layout,
            offscreen_texture_view,
            &self.sampler,
        );
    }

    pub fn render(&self, render_pass: &mut RenderPass) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}

fn create_bind_group(
    device: &Device,
    layout: &BindGroupLayout,
    offscreen_texture_view: &TextureView,
    sampler: &Sampler,
) -> BindGroup {
    let bind_group_entries = [
        BindGroupEntry {
            binding:  0,
            resource: BindingResource::TextureView(offscreen_texture_view),
        },
        BindGroupEntry {
            binding:  1,
            resource: BindingResource::Sampler(&sampler),
        },
    ];
    let bind_group_desc = BindGroupDescriptor {
        label: Some("hui::composite::bind_group"),
        layout,
        entries: &bind_group_entries,
    };
    device.create_bind_group(&bind_group_desc)
}

fn create_render_pipeline(
    device: &Device,
    surface_format: TextureFormat,
    bind_group_layout: &BindGroupLayout,
) -> RenderPipeline {
    let shader_module_content =
        ShaderSource::Wgsl(include_str!("composite.wgsl").into());
    let shader_module_desc = ShaderModuleDescriptor {
        label:  Some("hui::composite::shader_module"),
        source: shader_module_content,
    };
    let shader_module = device.create_shader_module(shader_module_desc);

    let vertex_state = VertexState {
        module:              &shader_module,
        entry_point:         Some("vs_main"),
        compilation_options: Default::default(),
        buffers:             &[],
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
        topology: PrimitiveTopology::TriangleList,
        ..Default::default()
    };

    let pipeline_layout_desc = PipelineLayoutDescriptor {
        label:                Some("hui::composite::pipeline_layout"),
        bind_group_layouts:   &[bind_group_layout],
        push_constant_ranges: &[],
    };
    let pipeline_layout = device.create_pipeline_layout(&pipeline_layout_desc);

    let render_pipeline_desc = RenderPipelineDescriptor {
        label:         Some("hui::composite::render_pipeline"),
        layout:        Some(&pipeline_layout),
        vertex:        vertex_state,
        fragment:      Some(fragment_state),
        primitive:     primitive_state,
        depth_stencil: None,
        multisample:   Default::default(),
        multiview:     None,
        cache:         None,
    };
    device.create_render_pipeline(&render_pipeline_desc)
}
