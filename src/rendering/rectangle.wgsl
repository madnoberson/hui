struct VertexInput {
    @location(0)  position:      vec3<f32>,
    @location(1)  mvp_0:         vec4<f32>,
    @location(2)  mvp_1:         vec4<f32>,
    @location(3)  mvp_2:         vec4<f32>,
    @location(4)  mvp_3:         vec4<f32>,
    @location(5)  fill_color:    vec4<f32>,
    @location(6)  border_color:  vec4<f32>,
    @location(7)  corner_radii:  vec4<f32>,
    @location(8)  shadow_color:  vec4<f32>,
    @location(9)  half_size:     vec2<f32>,
    @location(10) border_size:   f32,
    @location(11) shadow_spread: f32,
    @location(12) shadow_offset: vec2<f32>,
    @location(13) shadow_blur:   f32,
}

struct VertexOutput {
    @builtin(position)               clip_position:  vec4<f32>,
    @location(0)                     local_position: vec2<f32>,
    @location(1)  @interpolate(flat) half_size:      vec2<f32>,
    @location(2)  @interpolate(flat) fill_color:     vec4<f32>,
    @location(3)  @interpolate(flat) border_color:   vec4<f32>,
    @location(4)  @interpolate(flat) corner_radii:   vec4<f32>,
    @location(5)  @interpolate(flat) border_size:    f32,
    @location(6)  @interpolate(flat) shadow_color:   vec4<f32>,
    @location(7)  @interpolate(flat) shadow_offset:  vec2<f32>,
    @location(8)  @interpolate(flat) shadow_blur:    f32,
    @location(9)  @interpolate(flat) shadow_spread:  f32,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    let mvp = mat4x4<f32>(
        input.mvp_0,
        input.mvp_1,
        input.mvp_2,
        input.mvp_3,
    );

    let shadow_extent = input.shadow_blur + input.shadow_spread
                      + max(abs(input.shadow_offset.x), abs(input.shadow_offset.y));
    let expanded_pos  = input.position.xy * (1.0 + shadow_extent / input.half_size);

    var output: VertexOutput;
    output.clip_position  = mvp * vec4<f32>(expanded_pos, input.position.z, 1.0);
    output.local_position = expanded_pos * input.half_size;
    output.half_size      = input.half_size;
    output.fill_color     = input.fill_color;
    output.border_color   = input.border_color;
    output.corner_radii   = input.corner_radii;
    output.border_size    = input.border_size;
    output.shadow_color   = input.shadow_color;
    output.shadow_offset  = input.shadow_offset;
    output.shadow_blur    = input.shadow_blur;
    output.shadow_spread  = input.shadow_spread;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let p  = input.local_position;
    let hs = input.half_size;
    let r  = input.corner_radii;
    let b  = input.border_size;

    let shadow_p        = p - input.shadow_offset;
    let shadow_hs       = hs + vec2<f32>(input.shadow_spread);
    let shadow_distance = sd_rounded_rect_4(
        shadow_p,
        shadow_hs,
        r + vec4<f32>(input.shadow_spread),
    );
    let shadow_blur     = max(input.shadow_blur, 0.001);
    let shadow_alpha    = input.shadow_color.a
                        * (1.0 - smoothstep(-shadow_blur, shadow_blur, shadow_distance));

    let rect_distance = sd_rounded_rect_4(p, hs, r);
    let inside_rect   = step(0.0, -rect_distance);
    let shadow_a      = shadow_alpha * (1.0 - inside_rect);

    var color = vec4<f32>(input.shadow_color.rgb, shadow_a);

    let aa_width = fwidth(rect_distance) * 1.4;
    let alpha    = 1.0 - smoothstep(0.0, aa_width, rect_distance);

    let inner_radii    = max(r - vec4<f32>(b), vec4<f32>(0.0));
    let inner_distance = sd_rounded_rect_4(p, hs - vec2<f32>(b), inner_radii);

    let border_alpha = (1.0 - smoothstep(0.0, aa_width, rect_distance))
                     * smoothstep(-aa_width, 0.0, inner_distance);
    let fill_alpha   = 1.0 - smoothstep(0.0, aa_width, inner_distance);

    var rect_color = vec4<f32>(input.fill_color.rgb, input.fill_color.a * fill_alpha);

    let b_a    = input.border_color.a * border_alpha;
    rect_color = vec4<f32>(
        mix(rect_color.rgb, input.border_color.rgb, b_a),
        rect_color.a + b_a * (1.0 - rect_color.a),
    );
    rect_color.a *= alpha;

    color = vec4<f32>(
        mix(color.rgb, rect_color.rgb, rect_color.a),
        color.a + rect_color.a * (1.0 - color.a),
    );

    return color;
}

fn sd_rounded_rect_4(p: vec2<f32>, half_size: vec2<f32>, radii: vec4<f32>) -> f32 {
    var r = select(radii.xw, radii.yz, p.x > 0.0);
    r = vec2<f32>(select(r.x, r.y, p.y > 0.0));

    let q = abs(p) - half_size + r;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - r.x;
}
