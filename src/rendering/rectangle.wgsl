struct VertexInput {
    @location(0) position:      vec3<f32>,
    @location(1) mvp_0:         vec4<f32>,
    @location(2) mvp_1:         vec4<f32>,
    @location(3) mvp_2:         vec4<f32>,
    @location(4) mvp_3:         vec4<f32>,
    @location(5) fill_color:    vec4<f32>,
    @location(6) border_color:  vec4<f32>,
    @location(7) corner_radii:  vec4<f32>,
    @location(8) half_size:     vec2<f32>,
    @location(9) border_size:   f32,
}

struct VertexOutput {
    @builtin(position)              clip_position:  vec4<f32>,
    @location(0)                    local_position: vec2<f32>,
    @location(1) @interpolate(flat) half_size:      vec2<f32>,
    @location(2) @interpolate(flat) fill_color:     vec4<f32>,
    @location(3) @interpolate(flat) border_color:   vec4<f32>,
    @location(4) @interpolate(flat) corner_radii:   vec4<f32>,
    @location(5) @interpolate(flat) border_size:    f32,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    let mvp = mat4x4<f32>(
        input.mvp_0,
        input.mvp_1,
        input.mvp_2,
        input.mvp_3,
    );

    var output: VertexOutput;
    output.clip_position  = mvp * vec4<f32>(input.position, 1.0);
    output.local_position = input.position.xy * input.half_size;
    output.half_size      = input.half_size;
    output.fill_color     = input.fill_color;
    output.border_color   = input.border_color;
    output.corner_radii   = input.corner_radii;
    output.border_size    = input.border_size;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let p  = input.local_position;
    let hs = input.half_size;
    let r  = input.corner_radii;
    let b  = input.border_size;

    let distance = sd_rounded_rect_4(p, hs, r);
    let aa_width = fwidth(distance) * 1.4;
    let alpha    = 1.0 - smoothstep(0.0, aa_width, distance);

    let inner_radii    = max(r - vec4<f32>(b), vec4<f32>(0.0));
    let inner_distance = sd_rounded_rect_4(p, hs - vec2<f32>(b), inner_radii);

    let border_alpha = (1.0 - smoothstep(0.0, aa_width, distance))
                     * smoothstep(-aa_width, 0.0, inner_distance);
    let fill_alpha   = 1.0 - smoothstep(0.0, aa_width, inner_distance);

    var color = vec4<f32>(input.fill_color.rgb, input.fill_color.a * fill_alpha);

    let b_a = input.border_color.a * border_alpha;
    color = vec4<f32>(
        mix(color.rgb, input.border_color.rgb, b_a),
        color.a + b_a * (1.0 - color.a),
    );

    color.a *= alpha;

    return color;
}

fn sd_rounded_rect_4(p: vec2<f32>, half_size: vec2<f32>, radii: vec4<f32>) -> f32 {
    var r = select(radii.xw, radii.yz, p.x > 0.0);
    r = vec2<f32>(select(r.x, r.y, p.y > 0.0));

    let q = abs(p) - half_size + r;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - r.x;
}
