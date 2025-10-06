// Screen uniform for proper coordinate conversion
struct ScreenUniforms {
    screen_size: vec2<f32>,
}

@group(1) @binding(0)
var<uniform> screen: ScreenUniforms;

// Text vertex shader
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Convert world position to NDC using actual screen dimensions
    let ndc_x = (vertex.position.x / screen.screen_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (vertex.position.y / screen.screen_size.y) * 2.0; // Flip Y for screen coordinates

    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.tex_coords = vertex.tex_coords;
    out.color = vertex.color;

    return out;
}

// Fragment shader
@group(0) @binding(0)
var glyph_texture: texture_2d<f32>;
@group(0) @binding(1)
var glyph_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let alpha = textureSample(glyph_texture, glyph_sampler, in.tex_coords).r;
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}