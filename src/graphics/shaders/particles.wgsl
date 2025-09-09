// Particle system compute and render shaders
// GPU-only particle system for collision effects

struct Particle {
    position: vec3<f32>,
    velocity: vec3<f32>,
    life: f32,
    max_life: f32,
}

struct SpawnRequest {
    position: vec3<f32>,
    count: u32,
}

// Particle buffers (for compute shaders)
@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<storage, read_write> alive_count: atomic<u32>;
@group(0) @binding(2) var<storage, read> spawn_queue: array<SpawnRequest>;
@group(0) @binding(3) var<storage, read> spawn_count: u32;

// Constants
const MAX_PARTICLES: u32 = 8192u;
const PARTICLES_PER_COLLISION: u32 = 20u;
const PARTICLE_LIFETIME: f32 = 2.0;

// Simple random function
fn random(seed: u32) -> f32 {
    var s = seed;
    s = ((s >> 16u) ^ s) * 0x45d9f3bu;
    s = ((s >> 16u) ^ s) * 0x45d9f3bu;
    s = (s >> 16u) ^ s;
    return f32(s & 0xFFFFu) / 65535.0;
}

@compute @workgroup_size(64)
fn update_particles(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= MAX_PARTICLES) { return; }
    
    let dt = 0.016; // ~60 FPS delta time
    
    // Update existing particles
    if (particles[index].life > 0.0) {
        particles[index].life -= dt;
        if (particles[index].life <= 0.0) {
            // Particle died
            particles[index].life = 0.0;
            atomicSub(&alive_count, 1u);
        } else {
            // Update physics
            particles[index].position += particles[index].velocity * dt;
            particles[index].velocity.y -= 9.8 * dt; // Gravity
            particles[index].velocity *= 0.98; // Air resistance
        }
    }
}

@compute @workgroup_size(64)
fn spawn_particles(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let spawn_index = global_id.x;
    if (spawn_index >= spawn_count) { return; }
    
    let spawn_req = spawn_queue[spawn_index];
    let base_seed = spawn_index * 1000u;
    
    // Spawn particles for this collision
    for (var i = 0u; i < PARTICLES_PER_COLLISION; i++) {
        let particle_seed = base_seed + i;
        let particle_index = (spawn_index * PARTICLES_PER_COLLISION + i) % MAX_PARTICLES;
        
        // Only spawn if slot is free
        if (particles[particle_index].life <= 0.0) {
            // Generate random velocity
            let rand1 = random(particle_seed + 1u) * 2.0 - 1.0;
            let rand2 = random(particle_seed + 2u) * 2.0 - 1.0;
            let rand3 = random(particle_seed + 3u) * 2.0 - 1.0;
            let rand4 = random(particle_seed + 4u);
            
            let speed = 5.0 + rand4 * 10.0;
            let velocity = normalize(vec3<f32>(rand1, abs(rand2), rand3)) * speed;
            
            particles[particle_index] = Particle(
                spawn_req.position,
                velocity,
                PARTICLE_LIFETIME,
                PARTICLE_LIFETIME
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
        
        // Fade out over lifetime
        let life_ratio = particle.life / particle.max_life;
        output.color = vec4<f32>(1.0, 0.8, 0.2, life_ratio); // Orange fade
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