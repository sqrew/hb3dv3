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

    // Enhanced morphing Julia set parameters - slowly evolve over time
    let morph_speed = base_animation_speed * 0.08; // Very slow morphing for subtle evolution

    // Create morphing Julia parameters using multiple frequency waves
    let julia_morph_c1 = vec2<f32>(
        fractal.julia_c_real + 0.15 * sin(time * morph_speed * 0.7) + 0.08 * cos(time * morph_speed * 1.3),
        fractal.julia_c_imag + 0.12 * cos(time * morph_speed * 0.9) + 0.06 * sin(time * morph_speed * 1.1)
    );

    let julia_morph_c2 = vec2<f32>(
        fractal.julia2_c_real + 0.18 * cos(time * morph_speed * 0.6) + 0.09 * sin(time * morph_speed * 1.5),
        fractal.julia2_c_imag + 0.14 * sin(time * morph_speed * 0.8) + 0.07 * cos(time * morph_speed * 1.2)
    );

    // Calculate fractals at different scales with morphing parameters
    let mandel_val1 = mandelbrot(complex_coord1);
    let julia_val1 = julia(complex_coord1, julia_morph_c1);
    let burning_val1 = burning_ship(complex_coord1 * 0.8);

    // Additional fractal type with morphing parameters
    let tricorn_val1 = tricorn(complex_coord1 * 1.1);

    // Finer detail layer using morphing Julia set parameters
    let mandel_val2 = mandelbrot(complex_coord2);
    let julia_val2 = julia(complex_coord2, julia_morph_c2);


    // Enhanced blending with dynamic movement and animation speed
    let blend1 = 0.5 + 0.2 * sin(time * 0.04 * base_animation_speed);
    let blend2 = 0.5 + 0.15 * cos(time * 0.035 * base_animation_speed);
    let blend3 = 0.5 + 0.1 * sin(time * 0.05 * base_animation_speed);

    // Dynamic fractal weight morphing - slowly shift emphasis between fractal types
    let weight_morph_speed = morph_speed * 0.5; // Even slower weight changes
    let w_base = fractal.fractal_weights;

    // Create morphing weights that smoothly transition between different fractal dominance
    let weight_phase1 = time * weight_morph_speed;
    let weight_phase2 = time * weight_morph_speed * 0.7 + 1.57; // 90 degree phase offset
    let weight_phase3 = time * weight_morph_speed * 1.3 + 3.14; // 180 degree phase offset

    let w_morph = vec4<f32>(
        w_base.x * (0.8 + 0.4 * sin(weight_phase1)), // Mandelbrot weight morphing
        w_base.y * (0.8 + 0.4 * sin(weight_phase2)), // Julia weight morphing
        w_base.z * (0.8 + 0.4 * sin(weight_phase3)), // Burning ship weight morphing
        w_base.w * (0.9 + 0.2 * cos(weight_phase1))  // Detail weight morphing (more stable)
    );

    // Normalize morphed weights to maintain total intensity
    let total_weight = w_morph.x + w_morph.y + w_morph.z;
    let w_normalized = vec4<f32>(
        w_morph.x / total_weight,
        w_morph.y / total_weight,
        w_morph.z / total_weight,
        w_morph.w
    );

    // Enhanced mixing with tricorn influence
    let tricorn_influence = 0.15 * (0.5 + 0.5 * sin(time * morph_speed * 1.4));

    // Mix 2D fractals
    let mixed_large = mandel_val1 * w_normalized.x + julia_val1 * w_normalized.y +
                     burning_val1 * w_normalized.z + tricorn_val1 * tricorn_influence;

    let mixed_detail = mandel_val2 * w_normalized.x + julia_val2 * w_normalized.y;

    // Combine large and detail scales with morphing balance
    let detail_morph = w_normalized.w * (0.8 + 0.3 * cos(time * morph_speed * 0.9));
    let final_mixed = mix(mixed_large, mixed_detail, detail_morph);

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

    // Final intensity with iteration-aware boost (reduced for muted pastels)
    let edge_boosted = stretched_intensity + total_edge_boost;
    let intensity = edge_boosted * 0.25; // Reduced base brightness for muted pastels

    // Enhanced color palette with pastel hues for better visibility
    let base_color = vec3<f32>(0.0, 0.0, 0.0); // Pure black background

    // Lighter pastel color palette - emphasizing deep iterations with brighter colors
    let bright_lavender = vec3<f32>(0.45, 0.35, 0.55); // Bright lavender for deep areas
    let bright_blue = vec3<f32>(0.30, 0.40, 0.65); // Bright blue for deep areas
    let bright_mint = vec3<f32>(0.35, 0.55, 0.45); // Bright mint green for deep areas
    let bright_rose = vec3<f32>(0.55, 0.35, 0.40); // Bright dusty rose for deep areas
    let mid_sage = vec3<f32>(0.25, 0.35, 0.22); // Medium sage green for mid areas
    let mid_steel = vec3<f32>(0.25, 0.30, 0.35); // Medium steel blue for mid areas
    let soft_gray = vec3<f32>(0.15, 0.15, 0.17); // Soft gray for edge areas

    // Cosmic filament colors - ethereal and thread-like
    let filament_cyan = vec3<f32>(0.20, 0.35, 0.40); // Cosmic cyan threads
    let filament_violet = vec3<f32>(0.35, 0.20, 0.45); // Ethereal violet threads
    let filament_gold = vec3<f32>(0.45, 0.35, 0.15); // Golden cosmic threads

    // Color selection based on fractal position and time for smooth transitions
    let fractal_phase = final_mixed * 8.0 + time * 0.1;
    let color_wave1 = sin(fractal_phase);
    let color_wave2 = cos(fractal_phase * 1.4 + time * 0.07);
    let color_wave3 = sin(fractal_phase * 0.7 + time * 0.12);

    // Select colors based on waves for smooth gradients
    var deep_color: vec3<f32>;
    var mid_color: vec3<f32>;
    var edge_color: vec3<f32>;

    // Deep areas get the brightest, most prominent colors
    if (color_wave1 > 0.33) {
        deep_color = mix(bright_lavender, bright_blue, abs(color_wave2));
    } else if (color_wave1 > -0.33) {
        deep_color = mix(bright_blue, bright_mint, abs(color_wave2));
    } else {
        deep_color = mix(bright_mint, bright_rose, abs(color_wave2));
    }

    // Mid areas get medium brightness colors
    if (color_wave2 > 0.33) {
        mid_color = mix(mid_sage, mid_steel, abs(color_wave3));
    } else if (color_wave2 > -0.33) {
        mid_color = mix(mid_steel, bright_lavender * 0.6, abs(color_wave3));
    } else {
        mid_color = mix(bright_rose * 0.6, mid_sage, abs(color_wave3));
    }

    // Edge areas get the most subtle colors
    if (color_wave3 > 0.0) {
        edge_color = mix(soft_gray, mid_steel * 0.5, abs(color_wave1));
    } else {
        edge_color = mix(soft_gray, mid_sage * 0.5, abs(color_wave1));
    }

    // Filament color selection - flowing through different hues
    var filament_color: vec3<f32>;
    let filament_phase = time * 0.05 + world_pos.x * 0.3 + world_pos.y * 0.4 + world_pos.z * 0.2;
    let filament_wave = sin(filament_phase);

    if (filament_wave > 0.33) {
        filament_color = mix(filament_cyan, filament_violet, abs(color_wave2));
    } else if (filament_wave > -0.33) {
        filament_color = mix(filament_violet, filament_gold, abs(color_wave3));
    } else {
        filament_color = mix(filament_gold, filament_cyan, abs(color_wave1));
    }

    // Neural filament network - chaotic, tendrily cosmic connective fibers
    // Multi-scale noise for organic chaos
    let filament_coord = world_pos * 0.8;

    // Primary chaotic noise layers with different frequencies
    let noise1 = sin(filament_coord.x * 3.2 + time * 0.05) * cos(filament_coord.y * 2.8 + time * 0.07) * sin(filament_coord.z * 4.1 + time * 0.03);
    let noise2 = cos(filament_coord.x * 6.7 + time * 0.08) * sin(filament_coord.y * 5.3 + time * 0.04) * cos(filament_coord.z * 7.9 + time * 0.06);
    let noise3 = sin(filament_coord.x * 12.1 + time * 0.12) * cos(filament_coord.y * 9.8 + time * 0.09) * sin(filament_coord.z * 11.3 + time * 0.11);

    // Create chaotic directional flows with turbulence
    let chaos_flow1 = vec3<f32>(
        noise1 + 0.5 * noise2 + 0.25 * noise3,
        noise2 + 0.5 * noise3 + 0.25 * noise1,
        noise3 + 0.5 * noise1 + 0.25 * noise2
    );

    let chaos_flow2 = vec3<f32>(
        cos(filament_coord.x * 4.7 + time * 0.06) + 0.3 * sin(filament_coord.y * 8.2),
        sin(filament_coord.y * 5.9 + time * 0.08) + 0.3 * cos(filament_coord.z * 7.4),
        cos(filament_coord.z * 6.3 + time * 0.04) + 0.3 * sin(filament_coord.x * 9.1)
    );

    // Neural fiber patterns - irregular, branching tendrils
    let fiber_density1 = length(chaos_flow1 - normalize(filament_coord));
    let fiber_density2 = length(chaos_flow2 - normalize(filament_coord.zyx));
    let fiber_density3 = length(mix(chaos_flow1, chaos_flow2, 0.6) - normalize(filament_coord.yxz));

    // Highly variable thickness for organic feel
    let thickness_chaos1 = 0.08 + 0.15 * abs(noise1) + 0.05 * abs(noise3);
    let thickness_chaos2 = 0.06 + 0.12 * abs(noise2) + 0.08 * abs(noise1);
    let thickness_chaos3 = 0.10 + 0.18 * abs(noise3) + 0.06 * abs(noise2);

    // Branching intensity with sharp falloffs for fiber-like appearance
    let branch_intensity1 = pow(max(0.0, 1.0 - fiber_density1 / thickness_chaos1), 3.5);
    let branch_intensity2 = pow(max(0.0, 1.0 - fiber_density2 / thickness_chaos2), 4.2);
    let branch_intensity3 = pow(max(0.0, 1.0 - fiber_density3 / thickness_chaos3), 3.8);

    // Add secondary tendrils that branch off main fibers
    let secondary_chaos = sin(filament_coord.x * 15.7) * cos(filament_coord.y * 18.3) * sin(filament_coord.z * 14.2);
    let tertiary_chaos = cos(filament_coord.x * 22.1) * sin(filament_coord.y * 25.8) * cos(filament_coord.z * 19.6);

    let tendril_factor = pow(max(0.0, abs(secondary_chaos) - 0.7), 2.0) *
                        pow(max(0.0, abs(tertiary_chaos) - 0.6), 2.5);

    // Combine all chaotic neural elements
    let neural_base = max(branch_intensity1, max(branch_intensity2 * 0.7, branch_intensity3 * 0.5));
    let combined_filaments = neural_base + tendril_factor * 0.4;

    // Filaments are stronger in void regions (where fractals are weak)
    let void_regions = 1.0 - smoothstep(0.15, 0.4, final_mixed);
    let filament_strength = combined_filaments * void_regions * 0.3;

    // Blend filaments with fractal base
    let fractal_with_filaments = final_mixed + filament_strength;

    // Create iteration ranges for coloring (now using filament-enhanced fractal)
    let color_iteration_factor = 1.0 - fractal_with_filaments; // Invert: high iterations = low fractal_with_filaments value

    // Define three iteration bands for different color treatments
    let deep_areas = pow(max(0.0, color_iteration_factor - 0.6), 1.5); // Deepest 40%
    let mid_areas = pow(max(0.0, color_iteration_factor - 0.3) * (1.0 - step(0.6, color_iteration_factor)), 1.2); // Mid range
    let edge_areas = pow(max(0.0, color_iteration_factor - 0.1) * (1.0 - step(0.3, color_iteration_factor)), 1.0); // Outer edges

    // Filament-specific coloring - subtle cosmic threads
    let filament_areas = filament_strength * 2.0; // Enhance visibility

    // Enhanced edge detection for subtle glow effects
    let edge_detection1 = 1.0 - abs(fractal_with_filaments - 0.5) * 2.0; // Primary edge detection
    let edge_detection2 = 1.0 - abs(final_mixed - 0.3) * 3.33; // Secondary edge detection
    let edge_detection3 = 1.0 - abs(final_mixed - 0.7) * 1.43; // Tertiary edge detection

    // Create soft glow boundaries (subtle, not overwhelming)
    let soft_edge_glow = pow(max(0.0, edge_detection1), 2.2) * 0.25;
    let sharp_edge_glow = pow(max(0.0, edge_detection2), 3.5) * 0.15;
    let subtle_edge_glow = pow(max(0.0, edge_detection3), 2.8) * 0.1;

    // Gentle pulsing effect for edge glow (very subtle)
    let edge_pulse = 0.7 + 0.2 * sin(time * 1.8);
    let total_edge_glow = (soft_edge_glow + sharp_edge_glow + subtle_edge_glow) * edge_pulse;

    // Apply colors to different iteration bands with subtle edge glow enhancement (muted pastels)
    let colored_deep = deep_color * (deep_areas * 2.0 + total_edge_glow * 0.3); // Muted deep areas
    let colored_mid = mid_color * (mid_areas * 0.9 + total_edge_glow * 0.2); // Subtle mid areas
    let colored_edge = edge_color * (edge_areas * 0.7 + total_edge_glow * 0.4); // Gentle edge areas

    // Apply filament coloring - ethereal cosmic threads
    let colored_filaments = filament_color * filament_areas * 1.5; // Enhanced filament visibility

    // Combine all colored areas including filaments
    let total_colored_areas = colored_deep + colored_mid + colored_edge + colored_filaments;

    // Base gray for remaining areas, heavily reduced where deep colors and filaments are applied
    let color_coverage = clamp(deep_areas * 1.5 + mid_areas * 0.7 + edge_areas * 0.4 + filament_areas * 0.8, 0.0, 1.0);
    let gray_intensity = intensity * (1.0 - color_coverage * 0.85); // Stronger reduction
    let gray_component = vec3<f32>(0.12, 0.12, 0.14) * gray_intensity; // Darker gray

    // Final color mixing
    let color = mix(base_color, gray_component, gray_intensity) + total_colored_areas;

    // Enhanced edge-focused sparkle shimmer
    let sparkle_base = 0.020 * sin(contrast_enhanced * 28.0 + time * 1.1);
    let edge_sparkle = 0.035 * sin(total_edge_glow * 15.0 + time * 2.2); // Sparkles follow edges
    let boundary_sparkle = 0.030 * cos(edge_intensity * 20.0 + time * 1.6); // Sparkles on fractal boundaries

    // Combine sparkle effects - strongest on edges
    let total_sparkle = sparkle_base + edge_sparkle * total_edge_glow + boundary_sparkle * edge_intensity;

    // Sparkle color that matches the area colors
    let sparkle_color = mix(
        vec3<f32>(0.85, 0.9, 0.95), // Base white sparkle
        (deep_color + mid_color + edge_color) * 0.4, // Tinted by area colors
        total_edge_glow + edge_intensity * 0.5 // More color on edges
    );

    // Additional edge highlight sparkles (very subtle)
    let edge_highlight = total_edge_glow * 0.04 * abs(sin(time * 3.5));

    let final_color = color + sparkle_color * total_sparkle + bright_lavender * edge_highlight;

    return vec4<f32>(final_color, 1.0);
}
