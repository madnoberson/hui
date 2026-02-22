struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0)       uv:            vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let x = f32((vertex_index & 1u) * 2u) - 1.0;
    let y = f32((vertex_index >> 1u) * 2u) - 1.0;

    var output: VertexOutput;
    output.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    output.uv            = vec2<f32>(x * 0.5 + 0.5, 1.0 - (y * 0.5 + 0.5));

    return output;
}

@group(0) @binding(0) var ui_texture: texture_2d<f32>;
@group(0) @binding(1) var ui_sampler: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(ui_texture, ui_sampler, input.uv);
}
