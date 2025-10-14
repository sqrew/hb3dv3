// Particle system compute and render shaders
// GPU-only particle system for collision effects

struct Particle {
    position: vec3<f32>,
    velocity: vec3<f32>,
    life: f32,
    max_life: f32,
    color: vec4<f32>, // Store collision event color
}

struct SpawnRequest {
    position: vec3<f32>,
    count: u32,
    velocity: vec3<f32>,
    lifetime: f32,
    color: vec4<f32>, // RGBA
    spawn_mode: u32, // 0 = point spawn, 1 = burst spawn
    pad1: f32,
    pad2: f32,
    pad3: f32,
}

struct GravitationalBody {
    position: vec3<f32>,
    _pad1: f32,
    velocity: vec3<f32>,
    _pad2: f32,
    radius: f32,
    mass: f32,
    angular_velocity: f32,
    ergosphere_radius: f32,
    frame_dragging_strength: f32,
    _pad3: f32,
    _pad4: f32,
    _pad5: f32,
}

struct Explosion {
    position: vec3<f32>,
    current_radius: f32,
    force_strength: f32,
    falloff_type: u32,  // 0=Linear, 1=Quadratic, 2=Constant, 3=InverseLinear, 4=InverseQuadratic
    _pad1: f32,
    _pad2: f32,
}

// Particle buffers (for compute shaders)
@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<storage, read_write> alive_count: atomic<u32>;
@group(0) @binding(2) var<storage, read> spawn_queue: array<SpawnRequest>;
@group(0) @binding(3) var<storage, read> spawn_count: u32;
@group(0) @binding(4) var<uniform> delta_time: f32;
@group(0) @binding(5) var<storage, read> gravitational_bodies: array<GravitationalBody>;
@group(0) @binding(6) var<storage, read> gravity_body_count: u32;
@group(0) @binding(7) var<storage, read> explosions: array<Explosion>;
@group(0) @binding(8) var<uniform> explosion_count: u32;

// Constants
const MAX_PARTICLES: u32 = 524288u;
const GRAVITATIONAL_CONSTANT: f32 = 6.674e-1; // Match physics system constant
const MAX_DISTANCE_FROM_ORIGIN: f32 = 3000.0; // Maximum distance allowed from spawn location
const MAX_PARTICLE_VELOCITY: f32 = 300.0; // Prevent particles from being yeeted too far
const AIR_RESISTANCE: f32 = 0.995;

// Simple random function
fn random(seed: u32) -> f32 {
    var s = seed;
    s = ((s >> 16u) ^ s) * 0x45d9f3bu;
    s = ((s >> 16u) ^ s) * 0x45d9f3bu;
    s = (s >> 16u) ^ s;
    return f32(s & 0xFFFFu) / 65535.0;
}

// Generate evenly distributed point on sphere using Fibonacci sphere algorithm
fn fibonacci_sphere(i: u32, n: u32) -> vec3<f32> {
    // Safety check: avoid division by zero
    if (n <= 1u) {
        return vec3<f32>(0.0, 1.0, 0.0); // Return up vector for edge case
    }

    let phi = 3.141592653589793 * (sqrt(5.0) - 1.0); // Golden angle in radians
    let y = 1.0 - (f32(i) / f32(n - 1u)) * 2.0;  // y goes from 1 to -1
    let radius = sqrt(1.0 - y * y);  // radius at y

    let theta = phi * f32(i);

    let x = cos(theta) * radius;
    let z = sin(theta) * radius;

    return normalize(vec3<f32>(x, y, z));
}

@compute @workgroup_size(64)
fn update_particles(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= MAX_PARTICLES) { return; }
    
    let dt = delta_time;
    
    // Update existing particles
    if (particles[index].life > 0.0) {
        particles[index].life -= dt;

        // Check distance from origin for culling (same as bullet system)
        let distance_from_origin = length(particles[index].position);

        if (particles[index].life <= 0.0 || distance_from_origin >= MAX_DISTANCE_FROM_ORIGIN) {
            // Particle died or too far from origin
            particles[index].life = 0.0;
            atomicSub(&alive_count, 1u);
        } else {
            // Apply gravitational forces from all large bodies
            var gravitational_force = vec3<f32>(0.0, 0.0, 0.0);
            let particle_mass = 0.1; // Small mass for particles

            for (var i = 0u; i < gravity_body_count; i++) {
                let body = gravitational_bodies[i];
                let displacement = body.position - particles[index].position;
                let distance_squared = dot(displacement, displacement);

                // Avoid singularities and very close interactions
                if (distance_squared > 0.1) {
                    let distance = sqrt(distance_squared);

                    // Apply frame-dragging (ergosphere) effect for spinning bodies - EXACT COPY FROM PHYSICS.WGSL
                    if (body.angular_velocity != 0.0 && distance <= body.ergosphere_radius) {
                        // Frame-dragging twists spacetime - objects get carried along with rotation
                        let spin_axis = vec3<f32>(0.0, 1.0, 0.0); // Spin around Y-axis for better visibility
                        let radial_vector = displacement / distance;

                        // Calculate tangential direction (perpendicular to both spin axis and radial direction)
                        let tangential_direction = normalize(cross(spin_axis, radial_vector));

                        // Frame-dragging strength falls off with distance (realistic physics)
                        let ergosphere_factor = 1.0 - (distance / body.ergosphere_radius);
                        let ergosphere_factor_squared = ergosphere_factor * ergosphere_factor;

                        // Realistic frame-dragging - gentle acceleration that builds up over time
                        // Scale down the enormous frame-dragging values for gentle orbital effects
                        let scaled_frame_dragging = body.frame_dragging_strength * 0.00001; // Scale down by 100,000x
                        let orbital_acceleration = body.angular_velocity * scaled_frame_dragging * ergosphere_factor_squared;

                        // Apply gentle tangential force that creates gradual spiraling motion
                        let frame_drag_force = tangential_direction * orbital_acceleration;
                        gravitational_force += frame_drag_force;

                        // Very subtle inward component - frame-dragging should mostly be tangential
                        let spiral_factor = 0.02 * ergosphere_factor_squared;
                        let inward_spiral_force = -radial_vector * orbital_acceleration * spiral_factor;
                        gravitational_force += inward_spiral_force;
                    }

                    let force_magnitude = (GRAVITATIONAL_CONSTANT * body.mass * particle_mass) / distance_squared;
                    let force_direction = displacement / distance;

                    // Scale down gravitational effect for particles to prevent them from being sucked in too quickly
                    gravitational_force += force_direction * force_magnitude * 0.1;
                }
            }

            // Apply explosion forces
            var explosion_force = vec3<f32>(0.0, 0.0, 0.0);
            for (var explosion_idx = 0u; explosion_idx < explosion_count; explosion_idx++) {
                let explosion = explosions[explosion_idx];
                let explosion_displacement = particles[index].position - explosion.position;
                let explosion_distance = length(explosion_displacement);

                // Check if particle is within explosion radius
                if (explosion_distance <= explosion.current_radius && explosion_distance > 0.001) {
                    // Calculate explosion force direction (radial outward from explosion center)
                    let explosion_direction = explosion_displacement / explosion_distance;

                    // Calculate force magnitude based on falloff type (same logic as physics.wgsl)
                    var force_magnitude = 0.0;
                    if (explosion.falloff_type == 0u) {
                        // Linear falloff
                        force_magnitude = explosion.force_strength * (1.0 - (explosion_distance / explosion.current_radius));
                    } else if (explosion.falloff_type == 1u) {
                        // Quadratic falloff
                        let normalized_distance = explosion_distance / explosion.current_radius;
                        force_magnitude = explosion.force_strength * (1.0 - normalized_distance * normalized_distance);
                    } else if (explosion.falloff_type == 2u) {
                        // Constant falloff
                        force_magnitude = explosion.force_strength;
                    } else if (explosion.falloff_type == 3u) {
                        // Inverse linear falloff (stronger at edges)
                        force_magnitude = explosion.force_strength * (explosion_distance / explosion.current_radius);
                    } else if (explosion.falloff_type == 4u) {
                        // Inverse quadratic falloff (much stronger at edges)
                        let normalized_distance = explosion_distance / explosion.current_radius;
                        force_magnitude = explosion.force_strength * (normalized_distance * normalized_distance);
                    }

                    // Apply explosion force (outward from explosion center, scaled for particles)
                    let explosion_force_vec = explosion_direction * force_magnitude * 0.05; // Scale down for particles
                    explosion_force += explosion_force_vec;
                }
            }

            // Apply total acceleration (gravitational + explosion forces)
            let total_acceleration = (gravitational_force + explosion_force) / particle_mass;
            particles[index].velocity += total_acceleration * dt;

            // Clamp velocity to prevent particles from being yeeted too far
            let velocity_magnitude = length(particles[index].velocity);
            if (velocity_magnitude > MAX_PARTICLE_VELOCITY) {
                particles[index].velocity = normalize(particles[index].velocity) * MAX_PARTICLE_VELOCITY;
            }

            // Update position
            particles[index].position += particles[index].velocity * dt;

            // Apply air resistance
            particles[index].velocity *= AIR_RESISTANCE;
        }
    }
}

@compute @workgroup_size(64)
fn spawn_particles(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let spawn_index = global_id.x;
    if (spawn_index >= spawn_count) { return; }
    
    let spawn_req = spawn_queue[spawn_index];
    let base_seed = spawn_index * 1000u;
    
    // Spawn particles for this collision using event data
    for (var i = 0u; i < spawn_req.count; i++) {
        let particle_seed = base_seed + i;
        // Better slot distribution using prime numbers to avoid clustering
        let primary_index = (spawn_index * 113u + i * 79u + global_id.x * 31u) % MAX_PARTICLES;
        
        // Search for free slot with fallback (try up to 512 slots)
        var particle_index = primary_index;
        var found_slot = false;
        for (var search = 0u; search < 512u; search++) {
            if (particles[particle_index].life <= 0.0) {
                found_slot = true;
                break;
            }
            particle_index = (particle_index + 97u) % MAX_PARTICLES; // Another prime for search step
        }
        
        // Only spawn if we found a free slot
        if (found_slot) {
            var final_velocity: vec3<f32>;
            let rand4 = random(particle_seed + 4u);

            // Check spawn mode (explicit comparison for safety)
            if (spawn_req.spawn_mode == 1u && spawn_req.count > 0u) {
                // Burst mode - evenly distributed spherical pattern
                let direction = fibonacci_sphere(i, spawn_req.count);
                let speed = spawn_req.velocity.x; // Speed stored in velocity.x for burst mode
                let speed_variation = 0.8 + rand4 * 0.4; // Small speed variation
                final_velocity = direction * speed * speed_variation;
            } else {
                // Point spawn mode - random velocity (original behavior, default)
                let rand1 = random(particle_seed + 1u) * 2.0 - 1.0;
                let rand2 = random(particle_seed + 2u) * 2.0 - 1.0;
                let rand3 = random(particle_seed + 3u) * 2.0 - 1.0;

                // Generate random velocity with much higher speeds for proper delta time
                let speed = 1.0 + rand4 * 9.0; // Random speed between 30-90

                // Create a more dramatic burst with wider spread
                let random_direction = normalize(vec3<f32>(
                    rand1 * 1.5,           // Wider horizontal spread
                    abs(rand2) * 0.8 + 0.3, // Mix of upward and horizontal (0.3 to 1.1 range)
                    rand3 * 1.5            // Wider horizontal spread
                ));

                // Apply the speed to the direction
                final_velocity = random_direction * speed;
            }

            particles[particle_index] = Particle(
                spawn_req.position,
                final_velocity,
                spawn_req.lifetime * (0.8 + rand4 * 0.4), // Vary lifetime slightly
                spawn_req.lifetime,
                spawn_req.color // Use collision event color
            );

            atomicAdd(&alive_count, 1u);
        }
    }
}

// Vertex shader for particle rendering
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

// Render shader bindings
@group(0) @binding(0) var<uniform> view_proj: mat4x4<f32>;
@group(1) @binding(0) var<storage, read> render_particles: array<Particle>;

@vertex
fn vs_main(@builtin(instance_index) instance_index: u32) -> VertexOutput {
    let particle = render_particles[instance_index];
    
    var output: VertexOutput;
    
    // Only render alive particles
    if (particle.life > 0.0) {
        let world_pos = vec4<f32>(particle.position, 1.0);
        output.position = view_proj * world_pos;
        
        // Fade out over lifetime using collision event color with slower fade curve
        let life_ratio = particle.life / particle.max_life;

        // Use a power curve for slower fade - particles stay visible longer
        // Square the life_ratio to create a curve that fades slowly at first, then quickly at the end
        // let fade_curve = life_ratio * life_ratio;
        // Alternative: even slower fade using cubic curve
        let fade_curve = life_ratio * life_ratio * life_ratio;

        output.color = vec4<f32>(particle.color.rgb, particle.color.a * fade_curve);
    } else {
        // Dead particle - render offscreen
        output.position = vec4<f32>(-10.0, -10.0, -10.0, 1.0);
        output.color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
