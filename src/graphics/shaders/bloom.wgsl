// Bloom post-processing shaders for neon glow effect

// Vertex shader for fullscreen quad
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_fullscreen(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    
    // Generate fullscreen triangle (covers entire screen)
    // This creates vertices at: (-1, -1), (3, -1), (-1, 3)
    let x = f32(i32(vertex_index & 1u) << 2) - 1.0;
    let y = f32(i32(vertex_index & 2u) << 1) - 1.0;
    
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    
    return out;
}

// Brightness extraction shader
@group(0) @binding(0) var scene_texture: texture_2d<f32>;
@group(0) @binding(1) var scene_sampler: sampler;

@fragment
fn fs_brightness_extract(input: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(scene_texture, scene_sampler, input.uv);
    
    // Extract bright areas using luminance threshold
    let luminance = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));
    let brightness_threshold = 0.2; // Even lower threshold for neon effect
    
    if (luminance > brightness_threshold) {
        // Sharp cutoff for crisp neon edges, less blur
        let boost = (luminance - brightness_threshold) * 4.0; // Higher boost for intensity
        return vec4<f32>(color.rgb * (1.0 + boost), color.a);
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0); // No glow for dark areas
    }
}

// Gaussian blur shader
@group(0) @binding(0) var blur_texture: texture_2d<f32>;
@group(0) @binding(1) var blur_sampler: sampler;

struct BlurUniforms {
    direction: vec2<f32>,      // (1,0) for horizontal, (0,1) for vertical
    blur_strength: f32,        // Controls glow spread
    _padding: f32,
}

@group(1) @binding(0) var<uniform> blur_uniforms: BlurUniforms;

@fragment
fn fs_blur(input: VertexOutput) -> @location(0) vec4<f32> {
    let texel_size = 1.0 / vec2<f32>(textureDimensions(blur_texture));
    let direction = blur_uniforms.direction * texel_size * blur_uniforms.blur_strength;
    
    var color = vec4<f32>(0.0);
    
    // 3-tap blur for very sharp neon edges
    let offsets = array<f32, 3>(
        -1.0, 0.0, 1.0
    );
    let weights = array<f32, 3>(
        0.25, 0.5, 0.25
    );
    
    for (var i = 0; i < 3; i++) {
        let sample_uv = input.uv + direction * offsets[i];
        color += textureSample(blur_texture, blur_sampler, sample_uv) * weights[i];
    }
    
    return color;
}

// Final composition shader
@group(0) @binding(0) var original_texture: texture_2d<f32>;
@group(0) @binding(1) var original_sampler: sampler;
@group(0) @binding(2) var bloom_texture: texture_2d<f32>;
@group(0) @binding(3) var bloom_sampler: sampler;

struct CompositionUniforms {
    bloom_intensity: f32,      // Controls overall glow strength
    bloom_radius: f32,         // Controls glow spread
    exposure: f32,             // HDR exposure adjustment
    _padding: f32,
}

@group(1) @binding(0) var<uniform> composition_uniforms: CompositionUniforms;

@fragment
fn fs_composition(input: VertexOutput) -> @location(0) vec4<f32> {
    let original = textureSample(original_texture, original_sampler, input.uv);
    let bloom = textureSample(bloom_texture, bloom_sampler, input.uv);
    
    // Create sharper bloom falloff by applying power function
    let bloom_luminance = dot(bloom.rgb, vec3<f32>(0.299, 0.587, 0.114));
    let sharp_falloff = pow(bloom_luminance, 2.0); // Square for sharper edges
    let sharpened_bloom = bloom.rgb * (sharp_falloff / max(bloom_luminance, 0.001));
    
    // Combine original scene with sharpened bloom
    let final_color = original.rgb + sharpened_bloom * composition_uniforms.bloom_intensity;
    
    // Optional tone mapping for HDR-like effect
    let exposed = final_color * composition_uniforms.exposure;
    let tone_mapped = exposed / (exposed + vec3<f32>(1.0));
    
    return vec4<f32>(tone_mapped, original.a);
}