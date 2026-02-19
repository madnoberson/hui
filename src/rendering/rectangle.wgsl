struct VertexInput {
    @location(0) position:      vec3<f32>,
    @location(1) mvp_0:         vec4<f32>,
    @location(2) mvp_1:         vec4<f32>,
    @location(3) mvp_2:         vec4<f32>,
    @location(4) mvp_3:         vec4<f32>,
    @location(5) half_size:     vec2<f32>,
    @location(6) fill_color:    vec4<f32>,
    @location(7) border_color:  vec4<f32>,
    @location(8) corner_radius: f32,
    @location(9) border_size:   f32,
}

struct VertexOutput {
    @builtin(position) clip_position:  vec4 <f32>,
    @location(0)       local_position: vec2<f32>,
    @location(1)       half_size:      vec2<f32>,
    @location(2)       fill_color:     vec4<f32>,
    @location(3)       border_color:   vec4<f32>,
    @location(4)       corner_radius:  f32,
    @location(5)       border_size:    f32,
}

@vertex
fn vs_main(input : VertexInput) -> VertexOutput {
    let mvp = mat4x4<f32>(
        input.mvp_0,
        input.mvp_1,
        input.mvp_2,
        input.mvp_3
    );

    var output : VertexOutput;
    output.clip_position = mvp * vec4<f32>(input.position, 1.0);
    output.local_position = input.position.xy;
    output.half_size = input.half_size;
    output.fill_color = input.fill_color;
    output.border_color = input.border_color;
    output.corner_radius = input.corner_radius;
    output.border_size = input.border_size;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let r = input.corner_radius;
    let p = input.local_position * input.half_size;

    let distance = sd_rounded_rect(p, input.half_size - r, r);

    let aa_width = fwidth(distance) * 1.4;
    let alpha = 1.0 - smoothstep(0.0, aa_width, distance);

    let border = input.border_size;

    let inner_distance = sd_rounded_rect(p, input.half_size - r - border, max(r - border, 0.0));
    let border_alpha = (1.0 - smoothstep(0.0, aa_width, distance)) * smoothstep(-aa_width, 0.0, inner_distance);

    let fill_alpha = 1.0 - smoothstep(0.0, aa_width, inner_distance);

    var color = vec4<f32>(input.fill_color.rgb, input.fill_color.a * fill_alpha);

    let b_a = input.border_color.a * border_alpha;
    color = vec4<f32>(
        mix(color.rgb, input.border_color.rgb, b_a),
        color.a + b_a * (1.0 - color.a)
    );

    color.a *= alpha;

    return color;
}

fn sd_rounded_rect(p: vec2<f32>, half_size: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - half_size + vec2<f32> (r);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - r;
}
