// Fractal skybox shader
// Renders animated fractal patterns on a large encompassing sphere

// Camera uniform buffer
struct CameraUniform {
    view_proj: mat4x4<f32>,
    position: vec3<f32>,
    time: f32,  // Add time to camera uniform for skybox animation
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

// Fractal configuration uniforms
struct FractalUniforms {
    max_iterations: u32,
    palette: u32,
    _padding1: u32,
    _padding2: u32,
    zoom: f32,
    offset_x: f32,
    offset_y: f32,
    animation_speed: f32,
    julia_c_real: f32,
    julia_c_imag: f32,
    julia2_c_real: f32,
    julia2_c_imag: f32,
    // Use vec4 instead of array for proper alignment
    fractal_weights: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> fractal: FractalUniforms;

// Vertex input
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
}

// Vertex output
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) camera_position: vec3<f32>,
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Transform to clip space
    out.clip_position = camera.view_proj * vec4<f32>(vertex.position, 1.0);

    // Force skybox to render at maximum depth (classic skybox technique)
    out.clip_position.z = out.clip_position.w * 0.999999; // Just before far plane

    out.world_position = vertex.position;

    // Calculate UV coordinates from sphere position using cube mapping approach
    // This avoids polar distortion and seam artifacts
    let normalized_pos = normalize(vertex.position);

    // Use the normalized 3D position directly for fractal sampling
    // This creates seamless, distortion-free mapping across the entire sphere
    out.uv = vec2<f32>(
        normalized_pos.x * 0.5 + 0.5, // Map [-1, 1] to [0, 1]
        normalized_pos.z * 0.5 + 0.5  // Map [-1, 1] to [0, 1]
    );

    // Pass camera position to fragment shader
    out.camera_position = camera.position;

    return out;
}

// Complex number operations
fn complex_mult(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}

fn complex_mag_squared(z: vec2<f32>) -> f32 {
    return z.x * z.x + z.y * z.y;
}

// Mandelbrot set calculation using uniform parameters
fn mandelbrot(c: vec2<f32>) -> f32 {
    var z = vec2<f32>(0.0, 0.0);
    var iterations = 0u;
    let max_iterations = fractal.max_iterations;

    for (var i = 0u; i < max_iterations; i++) {
        let mag_sq = complex_mag_squared(z);
        if (mag_sq > 4.0 || mag_sq != mag_sq) { // Check for NaN
            break;
        }
        z = complex_mult(z, z) + c;
        iterations++;
    }

    return f32(iterations) / f32(max_iterations);
}

// Julia set calculation using uniform parameters
fn julia(z: vec2<f32>, c: vec2<f32>) -> f32 {
    var current_z = z;
    var iterations = 0u;
    let max_iterations = fractal.max_iterations;

    for (var i = 0u; i < max_iterations; i++) {
        if (complex_mag_squared(current_z) > 4.0) {
            break;
        }
        current_z = complex_mult(current_z, current_z) + c;
        iterations++;
    }

    return f32(iterations) / f32(max_iterations);
}

// Burning ship fractal using uniform parameters
fn burning_ship(c: vec2<f32>) -> f32 {
    var z = vec2<f32>(0.0, 0.0);
    var iterations = 0u;
    let max_iterations = fractal.max_iterations;

    for (var i = 0u; i < max_iterations; i++) {
        if (complex_mag_squared(z) > 4.0) {
            break;
        }
        z = vec2<f32>(abs(z.x), abs(z.y));
        z = complex_mult(z, z) + c;
        iterations++;
    }

    return f32(iterations) / f32(max_iterations);
}

// Tricorn fractal (Mandelbar) (simplified)
fn tricorn(c: vec2<f32>) -> f32 {
    var z = vec2<f32>(0.0, 0.0);
    var iterations = 0u;
    let max_iterations = 64u;

    for (var i = 0u; i < max_iterations; i++) {
        if (complex_mag_squared(z) > 4.0) {
            break;
        }
        // Complex conjugate before squaring
        z = vec2<f32>(z.x, -z.y);
        z = complex_mult(z, z) + c;
        iterations++;
    }

    return f32(iterations) / f32(max_iterations);
}

// Color palette functions
fn palette_electric(t: f32) -> vec3<f32> {
    let r = 0.5 + 0.5 * cos(6.28318 * (t + 0.0));
    let g = 0.5 + 0.5 * cos(6.28318 * (t + 0.33));
    let b = 0.5 + 0.5 * cos(6.28318 * (t + 0.67));
    return vec3<f32>(r, g, b);
}

fn palette_cosmic(t: f32) -> vec3<f32> {
    let purple = vec3<f32>(0.5, 0.0, 1.0);
    let blue = vec3<f32>(0.0, 0.5, 1.0);
    let cyan = vec3<f32>(0.0, 1.0, 1.0);
    let white = vec3<f32>(1.0, 1.0, 1.0);

    if (t < 0.33) {
        return mix(purple, blue, t * 3.0);
    } else if (t < 0.67) {
        return mix(blue, cyan, (t - 0.33) * 3.0);
    } else {
        return mix(cyan, white, (t - 0.67) * 3.0);
    }
}

fn palette_fire(t: f32) -> vec3<f32> {
    let black = vec3<f32>(0.0, 0.0, 0.0);
    let red = vec3<f32>(1.0, 0.0, 0.0);
    let orange = vec3<f32>(1.0, 0.5, 0.0);
    let yellow = vec3<f32>(1.0, 1.0, 0.0);

    if (t < 0.33) {
        return mix(black, red, t * 3.0);
    } else if (t < 0.67) {
        return mix(red, orange, (t - 0.33) * 3.0);
    } else {
        return mix(orange, yellow, (t - 0.67) * 3.0);
    }
}

fn palette_ocean(t: f32) -> vec3<f32> {
    let deep_blue = vec3<f32>(0.0, 0.1, 0.3);
    let blue = vec3<f32>(0.0, 0.3, 0.7);
    let light_blue = vec3<f32>(0.2, 0.7, 1.0);
    let white = vec3<f32>(0.9, 0.95, 1.0);

    if (t < 0.33) {
        return mix(deep_blue, blue, t * 3.0);
    } else if (t < 0.67) {
        return mix(blue, light_blue, (t - 0.33) * 3.0);
    } else {
        return mix(light_blue, white, (t - 0.67) * 3.0);
    }
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Map UV coordinates to complex plane with animated parameters
    let time = camera.time;

    // Dynamic animation parameters using fractal uniforms
    let base_animation_speed = fractal.animation_speed;
    let zoom = fractal.zoom + 0.12 * sin(0.08 * time * base_animation_speed) + 0.08 * cos(0.05 * time * base_animation_speed);
    let offset_x = fractal.offset_x + 0.25 * sin(0.12 * time * base_animation_speed) + 0.15 * cos(0.08 * time * base_animation_speed);
    let offset_y = fractal.offset_y + 0.2 * cos(0.1 * time * base_animation_speed) + 0.12 * sin(0.07 * time * base_animation_speed);

    // Rotation speed controlled by animation speed
    let rotation = time * 0.02 * base_animation_speed;
    let cos_rot = cos(rotation);
    let sin_rot = sin(rotation);

    // Use the 3D world position for seamless fractal mapping
    // This avoids UV distortion and creates uniform fractal density
    let world_pos = normalize(in.world_position);

    // Create 2D coordinates from 3D position with rotation
    // Use X-Z plane projection with Y influence for 3D variation
    let base_coord = vec2<f32>(
        world_pos.x + world_pos.y * 0.3, // Y component adds 3D variation
        world_pos.z + world_pos.y * 0.2  // Different Y influence for variety
    );

    // Apply rotation matrix to the 3D-derived coordinates
    let rotated_uv = vec2<f32>(
        base_coord.x * cos_rot - base_coord.y * sin_rot,
        base_coord.x * sin_rot + base_coord.y * cos_rot
    );

    // Multiple fractal scales with larger fractals
    let base_scale = 1.2; // Reduced from 2.0 to make fractals bigger
    let complex_coord1 = vec2<f32>(
        rotated_uv.x * base_scale * zoom + offset_x,
        rotated_uv.y * base_scale * zoom + offset_y
    );

    // Add a second scale for finer detail
    let detail_scale = 3.5; // Reduced from 6.0 to make detail layer bigger too
    let complex_coord2 = vec2<f32>(
        rotated_uv.x * detail_scale * zoom + offset_x * 2.0,
        rotated_uv.y * detail_scale * zoom + offset_y * 2.0
    );

    // Calculate fractals at different scales using uniform parameters
    let mandel_val1 = mandelbrot(complex_coord1);
    let julia_val1 = julia(complex_coord1, vec2<f32>(fractal.julia_c_real, fractal.julia_c_imag));
    let burning_val1 = burning_ship(complex_coord1 * 0.8);

    // Finer detail layer using second Julia set parameters
    let mandel_val2 = mandelbrot(complex_coord2);
    let julia_val2 = julia(complex_coord2, vec2<f32>(fractal.julia2_c_real, fractal.julia2_c_imag));

    // Enhanced blending with dynamic movement and animation speed
    let blend1 = 0.5 + 0.2 * sin(time * 0.04 * base_animation_speed);
    let blend2 = 0.5 + 0.15 * cos(time * 0.035 * base_animation_speed);
    let blend3 = 0.5 + 0.1 * sin(time * 0.05 * base_animation_speed);

    // Use fractal weights from uniforms for mixing
    let w = fractal.fractal_weights;

    // Weighted mixing of all fractal types
    let mixed_large = mandel_val1 * w.x + julia_val1 * w.y + burning_val1 * w.z;
    let mixed_detail = mandel_val2 * w.x + julia_val2 * w.y;

    // Combine large and detail scales for full sphere coverage
    let final_mixed = mix(mixed_large, mixed_detail, w.w);

    // Create high-contrast monochrome effect with iteration-aware enhancement
    // Higher iteration counts need stronger contrast to show fractal arms
    let iteration_factor = f32(fractal.max_iterations) / 64.0; // Scale factor based on iteration count

    // Adaptive gamma correction - higher iterations get more aggressive contrast
    let gamma = mix(0.6, 0.3, clamp(iteration_factor - 1.0, 0.0, 1.0)); // More aggressive gamma for high iterations
    let contrast_enhanced = pow(final_mixed, gamma);

    // Iteration-aware contrast stretching to pull apart similar values
    let stretch_factor = 1.0 + iteration_factor * 0.5; // More stretching for higher iterations
    let stretched = pow(contrast_enhanced, stretch_factor);

    // High-iteration edge sharpening using derivative-based detection
    let iteration_scaled = final_mixed * iteration_factor;
    let sharpened = stretched + sin(iteration_scaled * 3.14159 * 8.0) * 0.1 * iteration_factor;

    // Calculate fractal boundary detection for edge enhancement
    // Areas near the escape boundary (around 0.3-0.7 range) are the most interesting fractal edges
    let boundary_detection = 1.0 - abs(final_mixed - 0.5) * 2.0; // Peak at 0.5, fall off toward 0 and 1
    let edge_intensity = pow(boundary_detection, 3.0); // Sharp falloff for crisp edges

    // Use the contrast enhanced values directly so fractal patterns are bright
    // (Higher iteration counts = brighter, boundaries/escape areas = darker)

    // Apply iteration-aware edge enhancement using the sharpened values
    let stretched_intensity = sharpened;

    // Enhanced edge detection with iteration-aware scaling
    // Higher iterations reveal more detailed boundary structures
    let iteration_edge_scale = 1.0 + iteration_factor * 0.8; // Scale edge detection with iterations

    // Multi-frequency edge detection adapted for high iteration counts
    let fine_edge = 1.0 - abs(final_mixed - 0.25) * (3.33 * iteration_edge_scale); // Fine detail edges
    let coarse_edge = 1.0 - abs(final_mixed - 0.75) * (1.43 * iteration_edge_scale); // Coarse structure edges
    let mid_edge = 1.0 - abs(final_mixed - 0.5) * (2.0 * iteration_edge_scale); // Mid-range structures
    let combined_edge = max(max(pow(fine_edge, 2.0), pow(coarse_edge, 3.0)), pow(mid_edge, 2.5)) * (0.6 * iteration_factor);

    // Iteration-scaled boundary detection for fractal arms
    let boundary_1 = 1.0 - abs(final_mixed - 0.35) * (2.5 * iteration_edge_scale);
    let boundary_2 = 1.0 - abs(final_mixed - 0.65) * (2.5 * iteration_edge_scale);
    let enhanced_edge = max(pow(boundary_1, 2.5), pow(boundary_2, 2.5)) * (0.4 * iteration_factor);

    // Combine all edge detection methods with iteration awareness
    let total_edge_boost = combined_edge + enhanced_edge + edge_intensity * (0.3 * iteration_factor);

    // Final intensity with iteration-aware boost
    let edge_boosted = stretched_intensity + total_edge_boost;
    let intensity = edge_boosted * 0.35; // Keep base brightness but with iteration-enhanced detail

    // Ultra high contrast palette - true black background with gray highlights
    let base_color = vec3<f32>(0.0, 0.0, 0.0); // Pure black background
    let highlight_color = vec3<f32>(0.3, 0.32, 0.35); // Medium gray instead of bright white

    // Create high-contrast monochrome gradient
    let color = mix(base_color, highlight_color, intensity);

    // Reduced shimmer for darker atmosphere
    let sparkle = 0.015 * sin(contrast_enhanced * 30.0 + time * 1.0);
    let final_color = color + vec3<f32>(sparkle, sparkle, sparkle);

    return vec4<f32>(final_color, 1.0);
}