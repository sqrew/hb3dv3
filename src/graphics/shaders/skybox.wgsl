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

// Complex division: a / b = (a * conjugate(b)) / |b|^2
fn complex_div(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    let b_mag_sq = complex_mag_squared(b);
    if (b_mag_sq < 0.0001) {
        return vec2<f32>(0.0, 0.0); // Avoid division by zero
    }
    let b_conj = vec2<f32>(b.x, -b.y); // Complex conjugate
    let numerator = complex_mult(a, b_conj);
    return numerator / b_mag_sq;
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

// Newton fractal functions - showing convergence basins for polynomial roots
// Complex polynomial evaluation: f(z) = z^3 + a*z^2 + b*z + c
fn complex_poly3(z: vec2<f32>, a: vec2<f32>, b: vec2<f32>, c: vec2<f32>) -> vec2<f32> {
    let z2 = complex_mult(z, z);
    let z3 = complex_mult(z2, z);
    return z3 + complex_mult(a, z2) + complex_mult(b, z) + c;
}

// Derivative: f'(z) = 3*z^2 + 2*a*z + b
fn complex_poly3_derivative(z: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    let z2 = complex_mult(z, z);
    return vec2<f32>(3.0, 0.0) * z2 + vec2<f32>(2.0, 0.0) * complex_mult(a, z) + b;
}

// Newton method iteration: z_new = z - f(z)/f'(z)
fn newton_iteration(z: vec2<f32>, a: vec2<f32>, b: vec2<f32>, c: vec2<f32>) -> vec2<f32> {
    let f_z = complex_poly3(z, a, b, c);
    let df_z = complex_poly3_derivative(z, a, b);

    // Avoid division by zero
    if (complex_mag_squared(df_z) < 0.0001) {
        return z;
    }

    return z - complex_div(f_z, df_z);
}

// Newton fractal - returns convergence basin information
fn newton_fractal(start_z: vec2<f32>, a: vec2<f32>, b: vec2<f32>, c: vec2<f32>) -> f32 {
    var z = start_z;
    var iterations = 0u;
    let max_iterations = 48u; // Higher iterations for sharper Newton boundaries

    for (var i = 0u; i < max_iterations; i++) {
        let z_new = newton_iteration(z, a, b, c);

        // Check for convergence (tighter threshold for sharper boundaries)
        if (complex_mag_squared(z_new - z) < 0.0001) {
            break;
        }

        z = z_new;
        iterations++;

        // Prevent divergence
        if (complex_mag_squared(z) > 100.0) {
            break;
        }
    }

    // Return normalized convergence info
    // The final z value determines which root basin we're in
    let basin_id = atan2(z.y, z.x) / (2.0 * 3.14159) + 0.5; // Convert to 0-1 range
    return fract(basin_id + f32(iterations) * 0.1); // Add iteration info for visual variety
}

// Flowing Newton fractal with morphing polynomial coefficients
fn flowing_newton_fractal(coord: vec2<f32>, time_factor: f32) -> f32 {
    // Morphing polynomial coefficients create flowing convergence basins
    let flow_speed = time_factor * 0.3;

    // Dynamic coefficients that flow and change
    let a = vec2<f32>(
        0.5 + 0.3 * sin(flow_speed * 0.7 + coord.x * 2.0),
        0.2 + 0.4 * cos(flow_speed * 0.9 + coord.y * 1.8)
    );

    let b = vec2<f32>(
        -0.8 + 0.6 * cos(flow_speed * 1.1 + coord.x * 1.5),
        0.3 + 0.5 * sin(flow_speed * 0.8 + coord.y * 2.2)
    );

    let c = vec2<f32>(
        0.1 + 0.2 * sin(flow_speed * 1.3 + coord.x * 0.8),
        -0.4 + 0.3 * cos(flow_speed * 0.6 + coord.y * 1.2)
    );

    return newton_fractal(coord, a, b, c);
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
    let white = vec3<f32>(0.85, 0.9, 0.95); // Softer white to prevent wash-out

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
    let white = vec3<f32>(0.75, 0.8, 0.85); // Softer white to prevent wash-out

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

    // Perspective distortion - make fractals feel wrapped around a sphere
    let distance_from_center = length(world_pos);
    let normalized_world_pos = normalize(world_pos);

    // Create spherical mapping distortion
    let sphere_phi = atan2(normalized_world_pos.z, normalized_world_pos.x);
    let sphere_theta = acos(clamp(normalized_world_pos.y, -1.0, 1.0));

    // Convert spherical coordinates to distorted UV
    let sphere_u = sphere_phi / (2.0 * 3.14159) + 0.5;
    let sphere_v = sphere_theta / 3.14159;

    // Blend between flat projection and spherical mapping based on distance
    let sphere_blend = smoothstep(0.5, 2.0, distance_from_center);
    let sphere_coord = vec2<f32>(sphere_u * 4.0 - 2.0, sphere_v * 4.0 - 2.0);

    // Create perspective-aware coordinates
    let perspective_coord = mix(rotated_uv, sphere_coord, sphere_blend * 0.3);

    // Add subtle fish-eye distortion for depth
    let distort_strength = 0.15 * sphere_blend;
    let coord_distance = length(perspective_coord);
    let distortion_factor = 1.0 + distort_strength * coord_distance * coord_distance;
    let distorted_coord = perspective_coord * distortion_factor;

    // Multiple fractal scales with larger fractals (now using distorted coordinates)
    let base_scale = 1.2; // Reduced from 2.0 to make fractals bigger
    let complex_coord1 = vec2<f32>(
        distorted_coord.x * base_scale * zoom + offset_x,
        distorted_coord.y * base_scale * zoom + offset_y
    );

    // Add a second scale for finer detail (also using distorted coordinates)
    let detail_scale = 3.5; // Reduced from 6.0 to make detail layer bigger too
    let complex_coord2 = vec2<f32>(
        distorted_coord.x * detail_scale * zoom + offset_x * 2.0,
        distorted_coord.y * detail_scale * zoom + offset_y * 2.0
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

    // Add Newton fractals with flowing convergence basins
    let newton_val1 = flowing_newton_fractal(complex_coord1 * 0.7, time * morph_speed);
    let newton_val2 = flowing_newton_fractal(complex_coord1 * 1.3, time * morph_speed * 1.4);

    // Finer detail layer using morphing Julia set parameters
    let mandel_val2 = mandelbrot(complex_coord2);
    let julia_val2 = julia(complex_coord2, julia_morph_c2);

    // DYNAMIC FRACTAL TRANSFORMATION SYSTEM - Flowing, branching, metamorphosis
    // Create transformation zones that flow across the space
    let transform_time = time * morph_speed * 2.0; // Faster transformation waves

    // Multiple flowing transformation waves with different patterns
    let transform_wave1 = sin(complex_coord1.x * 3.0 + transform_time * 0.7) * cos(complex_coord1.y * 2.8 + transform_time * 0.5);
    let transform_wave2 = cos(complex_coord1.x * 4.2 + transform_time * 0.9) * sin(complex_coord1.y * 3.6 + transform_time * 0.6);
    let transform_wave3 = sin(complex_coord1.x * 2.1 + transform_time * 1.1) * cos(complex_coord1.y * 4.3 + transform_time * 0.8);

    // Create branching transformation fields
    let branch_field = normalize(vec2<f32>(transform_wave1, transform_wave2));
    let branch_strength = abs(transform_wave3) * smoothstep(0.55, 0.65, abs(transform_wave1 + transform_wave2));

    // METAMORPHOSIS ZONES - Where fractals transform into each other
    let metamorphosis_phase = 0.5 + 0.5 * sin(transform_time * 0.4 + length(complex_coord1) * 2.0);

    // MULTI-JUNCTURE TRANSFORMATION NETWORK - Dynamic non-linear fractal evolution!
    // Create spatial influence zones for different transformation types
    let spatial_x = complex_coord1.x * 2.0 + time * 0.1;
    let spatial_y = complex_coord1.y * 1.8 + time * 0.08;
    let spatial_influence = sin(spatial_x) * cos(spatial_y) + 0.5 * cos(spatial_x * 1.3) * sin(spatial_y * 0.7);

    // Time-varying transformation probabilities (each fractal can go to ANY other fractal!)
    let time_phase1 = time * morph_speed * 0.8 + spatial_influence;
    let time_phase2 = time * morph_speed * 1.2 + spatial_influence * 0.7;
    let time_phase3 = time * morph_speed * 0.6 + spatial_influence * 1.3;

    // MANDELBROT JUNCTURES - Can transform to Julia, Newton, OR Burning Ship
    let mandel_to_julia_prob = 0.5 + 0.5 * sin(time_phase1 + transform_wave1 * 0.4);
    let mandel_to_newton_prob = 0.5 + 0.5 * cos(time_phase2 + transform_wave2 * 0.3);
    let mandel_to_burning_prob = 0.5 + 0.5 * sin(time_phase3 + transform_wave3 * 0.5);

    // JULIA JUNCTURES - Can transform to Mandelbrot, Burning Ship, OR Newton
    let julia_to_mandel_prob = 0.5 + 0.5 * cos(time_phase1 + transform_wave2 * 0.3);
    let julia_to_burning_prob = 0.5 + 0.5 * sin(time_phase2 + transform_wave3 * 0.4);
    let julia_to_newton_prob = 0.5 + 0.5 * cos(time_phase3 + transform_wave1 * 0.3);

    // BURNING SHIP JUNCTURES - Can transform to Newton, Mandelbrot, OR Julia
    let burning_to_newton_prob = 0.5 + 0.5 * sin(time_phase1 + transform_wave3 * 0.4);
    let burning_to_mandel_prob = 0.5 + 0.5 * cos(time_phase2 + transform_wave1 * 0.3);
    let burning_to_julia_prob = 0.5 + 0.5 * sin(time_phase3 + transform_wave2 * 0.5);

    // NEWTON JUNCTURES - Can transform to Mandelbrot, Julia, OR Burning Ship
    let newton_to_mandel_prob = 0.5 + 0.5 * cos(time_phase1 + transform_wave1 * 0.3);
    let newton_to_julia_prob = 0.5 + 0.5 * sin(time_phase2 + transform_wave2 * 0.4);
    let newton_to_burning_prob = 0.5 + 0.5 * cos(time_phase3 + transform_wave3 * 0.3);

    // Create sharp transition thresholds for each juncture
    let mandel_to_julia = smoothstep(0.48, 0.52, metamorphosis_phase + mandel_to_julia_prob * 0.3);
    let mandel_to_newton = smoothstep(0.45, 0.49, metamorphosis_phase + mandel_to_newton_prob * 0.3);
    let mandel_to_burning = smoothstep(0.51, 0.55, metamorphosis_phase + mandel_to_burning_prob * 0.3);

    let julia_to_mandel = smoothstep(0.47, 0.51, metamorphosis_phase + julia_to_mandel_prob * 0.3);
    let julia_to_burning = smoothstep(0.49, 0.53, metamorphosis_phase + julia_to_burning_prob * 0.3);
    let julia_to_newton = smoothstep(0.46, 0.50, metamorphosis_phase + julia_to_newton_prob * 0.3);

    let burning_to_newton = smoothstep(0.50, 0.54, metamorphosis_phase + burning_to_newton_prob * 0.3);
    let burning_to_mandel = smoothstep(0.48, 0.52, metamorphosis_phase + burning_to_mandel_prob * 0.3);
    let burning_to_julia = smoothstep(0.46, 0.50, metamorphosis_phase + burning_to_julia_prob * 0.3);

    let newton_to_mandel = smoothstep(0.49, 0.53, metamorphosis_phase + newton_to_mandel_prob * 0.3);
    let newton_to_julia = smoothstep(0.47, 0.51, metamorphosis_phase + newton_to_julia_prob * 0.3);
    let newton_to_burning = smoothstep(0.51, 0.55, metamorphosis_phase + newton_to_burning_prob * 0.3);

    // BRANCHING INFINITY EFFECTS - Fractals split and multiply
    let branch_coords = complex_coord1 + branch_field * branch_strength * 0.3;
    let split_coords = complex_coord1 * (1.0 + branch_strength * 0.5);

    // Calculate branched versions of fractals
    let mandel_branch = mandelbrot(branch_coords);
    let julia_branch = julia(split_coords, julia_morph_c1 + vec2<f32>(transform_wave1, transform_wave2) * 0.1);
    let burning_branch = burning_ship(branch_coords * 0.9);
    let newton_branch = flowing_newton_fractal(branch_coords * 1.1, transform_time + length(branch_coords));

    // MULTI-JUNCTURE TRANSFORMATION - Dynamic metamorphosis using probability network!
    // Each fractal can transform to any other fractal based on spatial zones and time
    let transformed_mandel = mix(
        mandel_val1,
        mix(
            mix(julia_branch, newton_branch, mandel_to_julia / (mandel_to_julia + mandel_to_newton + 0.001)),
            burning_branch,
            mandel_to_burning
        ),
        branch_strength
    );

    let transformed_julia = mix(
        julia_val1,
        mix(
            mix(mandel_branch, burning_branch, julia_to_mandel / (julia_to_mandel + julia_to_burning + 0.001)),
            newton_branch,
            julia_to_newton
        ),
        branch_strength
    );

    let transformed_burning = mix(
        burning_val1,
        mix(
            mix(newton_branch, mandel_branch, burning_to_newton / (burning_to_newton + burning_to_mandel + 0.001)),
            julia_branch,
            burning_to_julia
        ),
        branch_strength
    );

    // Newton fractal multi-juncture transformation
    let transformed_newton = mix(
        newton_val1,
        mix(
            mix(mandel_branch, julia_branch, newton_to_mandel / (newton_to_mandel + newton_to_julia + 0.001)),
            burning_branch,
            newton_to_burning
        ),
        branch_strength
    );

    // SUDDEN METAMORPHOSIS - Instant transformations in certain regions
    let sudden_transform_threshold = 0.7 + 0.2 * sin(transform_time * 1.3);
    let sudden_zones = step(sudden_transform_threshold, abs(transform_wave1 * transform_wave2));

    // Apply sudden transformations using multi-juncture probability network!
    // Each fractal can suddenly transform to its highest probability destination
    let final_mandel = mix(transformed_mandel,
        select(select(julia_branch, newton_branch, mandel_to_newton > mandel_to_julia),
               burning_branch, mandel_to_burning > max(mandel_to_julia, mandel_to_newton)),
        sudden_zones * max(max(mandel_to_julia, mandel_to_newton), mandel_to_burning));

    let final_julia = mix(transformed_julia,
        select(select(mandel_branch, burning_branch, julia_to_burning > julia_to_mandel),
               newton_branch, julia_to_newton > max(julia_to_mandel, julia_to_burning)),
        sudden_zones * max(max(julia_to_mandel, julia_to_burning), julia_to_newton));

    let final_burning = mix(transformed_burning,
        select(select(newton_branch, mandel_branch, burning_to_mandel > burning_to_newton),
               julia_branch, burning_to_julia > max(burning_to_newton, burning_to_mandel)),
        sudden_zones * max(max(burning_to_newton, burning_to_mandel), burning_to_julia));

    let final_newton = mix(transformed_newton,
        select(select(mandel_branch, julia_branch, newton_to_julia > newton_to_mandel),
               burning_branch, newton_to_burning > max(newton_to_mandel, newton_to_julia)),
        sudden_zones * max(max(newton_to_mandel, newton_to_julia), newton_to_burning));


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

    // Mix 2D fractals using TRANSFORMED fractals (flowing, branching, metamorphosis) + NEWTON!
    // Add Newton fractal as a dynamic fourth component
    let newton_weight = 0.2 * (0.5 + 0.5 * sin(time * morph_speed * 0.4 + length(complex_coord1)));

    let mixed_large = final_mandel * w_normalized.x + final_julia * w_normalized.y +
                     final_burning * w_normalized.z + final_newton * newton_weight;

    let mixed_detail = mandel_val2 * w_normalized.x + julia_val2 * w_normalized.y +
                      newton_val2 * newton_weight * 0.7;

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
    let color_wave2 = cos(fractal_phase * 1.4 + time * 0.049); // 0.07 * 0.7
    let color_wave3 = sin(fractal_phase * 0.7 + time * 0.084); // 0.12 * 0.7

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

    // SHARP edge detection for crisp fractal boundary beauty
    let edge_detection1 = 1.0 - abs(fractal_with_filaments - 0.5) * 4.0; // Much sharper primary detection
    let edge_detection2 = 1.0 - abs(final_mixed - 0.2) * 8.0; // Detect shallow iteration boundaries
    let edge_detection3 = 1.0 - abs(final_mixed - 0.8) * 6.0; // Detect deep iteration boundaries
    let edge_detection4 = 1.0 - abs(final_mixed - 0.6) * 10.0; // Ultra-sharp mid-boundary detection

    // Create CRISP glow boundaries that highlight fractal tendrils
    let soft_edge_glow = pow(max(0.0, edge_detection1), 4.0) * 0.15;
    let sharp_edge_glow = pow(max(0.0, edge_detection2), 6.0) * 0.2;
    let deep_edge_glow = pow(max(0.0, edge_detection3), 5.0) * 0.18;
    let ultra_edge_glow = pow(max(0.0, edge_detection4), 8.0) * 0.12;

    // Gentle pulsing effect for edge glow (very subtle)
    let edge_pulse = 0.7 + 0.2 * sin(time * 1.8);
    let total_edge_glow = (soft_edge_glow + sharp_edge_glow + deep_edge_glow + ultra_edge_glow) * edge_pulse;

    // FRACTAL INTERFERENCE PATTERNS - Mathematical wave physics where fractal universes collide!
    // Calculate fractal purity - how dominant each type is at this location
    let mandel_purity = final_mandel / (final_mandel + final_julia + final_burning + final_newton + 0.001);
    let julia_purity = final_julia / (final_mandel + final_julia + final_burning + final_newton + 0.001);
    let burning_purity = final_burning / (final_mandel + final_julia + final_burning + final_newton + 0.001);
    let newton_purity = final_newton / (final_mandel + final_julia + final_burning + final_newton + 0.001);

    // Detect interference zones - where multiple fractals have significant presence (softened)
    let mandel_julia_interference = mandel_purity * julia_purity * 2.5; // Reduced for muted pastels
    let julia_burning_interference = julia_purity * burning_purity * 2.5;
    let burning_newton_interference = burning_purity * newton_purity * 2.5;
    let newton_mandel_interference = newton_purity * mandel_purity * 2.5;

    // Create wave interference patterns using mathematical wave equations
    let interference_frequency = 15.0; // Wave frequency for standing patterns
    let interference_distance = length(complex_coord1);

    // Standing wave patterns in interference zones (slowed by 30%)
    let wave_phase1 = interference_frequency * interference_distance + time * 1.4; // 2.0 * 0.7
    let wave_phase2 = interference_frequency * interference_distance * 1.4 + time * 1.19; // 1.7 * 0.7
    let wave_phase3 = interference_frequency * interference_distance * 0.8 + time * 1.61; // 2.3 * 0.7
    let wave_phase4 = interference_frequency * interference_distance * 1.2 + time * 1.33; // 1.9 * 0.7

    // Mathematical wave interference - creates geometric patterns
    let standing_wave1 = sin(wave_phase1) * cos(wave_phase1 * 0.7) * mandel_julia_interference;
    let standing_wave2 = cos(wave_phase2) * sin(wave_phase2 * 1.3) * julia_burning_interference;
    let standing_wave3 = sin(wave_phase3) * cos(wave_phase3 * 0.9) * burning_newton_interference;
    let standing_wave4 = cos(wave_phase4) * sin(wave_phase4 * 1.1) * newton_mandel_interference;

    // Golden ratio spiral interference - emerges at high-energy interference nodes
    let golden_ratio = 1.618033988749895;
    let spiral_angle = atan2(complex_coord1.y, complex_coord1.x);
    let spiral_radius = length(complex_coord1);
    let golden_spiral_phase = spiral_angle * golden_ratio + log(spiral_radius + 0.1) * golden_ratio + time * 0.35; // 0.5 * 0.7

    // Golden spiral emerges where multiple interferences combine (softened)
    let interference_energy = abs(standing_wave1) + abs(standing_wave2) + abs(standing_wave3) + abs(standing_wave4);
    let golden_spiral_intensity = sin(golden_spiral_phase) * cos(golden_spiral_phase * golden_ratio) *
                                 smoothstep(0.3, 0.8, interference_energy) * 0.08; // Reduced intensity

    // Combine all interference patterns
    let total_interference = standing_wave1 + standing_wave2 + standing_wave3 + standing_wave4 + golden_spiral_intensity;

    // Apply interference to edge glow for geometric enhancement (softened)
    let interference_enhanced_glow = total_edge_glow + abs(total_interference) * 0.15; // Reduced enhancement

    // Apply colors to different iteration bands with interference-enhanced glow (muted pastels)
    let colored_deep = deep_color * (deep_areas * 1.5 + interference_enhanced_glow * 0.3); // Reduced multiplier to prevent white-out
    let colored_mid = mid_color * (mid_areas * 0.9 + interference_enhanced_glow * 0.2); // Subtle mid areas
    let colored_edge = edge_color * (edge_areas * 0.7 + interference_enhanced_glow * 0.4); // Gentle edge areas

    // Apply filament coloring - ethereal cosmic threads
    let colored_filaments = filament_color * filament_areas * 1.5; // Enhanced filament visibility

    // INTERFERENCE PATTERN COLORING - Geometric wave patterns
    // Create specific colors for interference phenomena
    let interference_cyan = vec3<f32>(0.15, 0.25, 0.35); // Cool interference waves
    let interference_gold = vec3<f32>(0.35, 0.25, 0.10); // Warm interference nodes
    let golden_spiral_color = vec3<f32>(0.40, 0.30, 0.15); // Golden ratio spirals

    // Color interference patterns based on their mathematical nature (muted)
    let wave_interference_color = mix(interference_cyan, interference_gold,
                                    0.5 + 0.5 * sin(time * 2.1 + total_interference * 8.0)); // 3.0 * 0.7
    let golden_interference_color = golden_spiral_color * abs(golden_spiral_intensity) * 2.0; // Reduced intensity

    // Combine interference colors (softened for pastels)
    let colored_interference = wave_interference_color * abs(total_interference) * 0.8 + golden_interference_color;

    // Combine all colored areas including filaments and interference patterns
    let total_colored_areas = colored_deep + colored_mid + colored_edge + colored_filaments + colored_interference;

    // Base gray for remaining areas, softly reduced where deep colors, filaments, and interference are applied
    let interference_coverage = abs(total_interference) * 0.4; // Reduced interference impact
    let color_coverage = clamp(deep_areas * 1.5 + mid_areas * 0.7 + edge_areas * 0.4 + filament_areas * 0.8 + interference_coverage, 0.0, 1.0);
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

    // Sparkle color that matches the area colors (softened to prevent white-out)
    let sparkle_color = mix(
        vec3<f32>(0.55, 0.6, 0.65), // Softer base sparkle (was too bright white)
        (deep_color + mid_color + edge_color) * 0.4, // Tinted by area colors
        total_edge_glow + edge_intensity * 0.5 // More color on edges
    );

    // Additional edge highlight sparkles (very subtle)
    let edge_highlight = total_edge_glow * 0.08 * abs(sin(time * 3.5));

    let final_color = color + sparkle_color * total_sparkle + bright_lavender * edge_highlight * 0.6; // Reduced edge highlight

    return vec4<f32>(final_color, 1.0);
}
