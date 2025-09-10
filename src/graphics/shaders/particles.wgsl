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
    padding: vec4<f32>, // For alignment
}

// Particle buffers (for compute shaders)
@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<storage, read_write> alive_count: atomic<u32>;
@group(0) @binding(2) var<storage, read> spawn_queue: array<SpawnRequest>;
@group(0) @binding(3) var<storage, read> spawn_count: u32;
@group(0) @binding(4) var<uniform> delta_time: f32;

// Constants
const MAX_PARTICLES: u32 = 8192u;

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
    
    let dt = delta_time;
    
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
    
    // Spawn particles for this collision using event data
    for (var i = 0u; i < spawn_req.count; i++) {
        let particle_seed = base_seed + i;
        // Better slot distribution using prime numbers to avoid clustering
        let primary_index = (spawn_index * 113u + i * 79u + global_id.x * 31u) % MAX_PARTICLES;
        
        // Search for free slot with fallback (try up to 8 slots)
        var particle_index = primary_index;
        var found_slot = false;
        for (var search = 0u; search < 8u; search++) {
            if (particles[particle_index].life <= 0.0) {
                found_slot = true;
                break;
            }
            particle_index = (particle_index + 97u) % MAX_PARTICLES; // Another prime for search step
        }
        
        // Only spawn if we found a free slot
        if (found_slot) {
            // Generate random velocity based on collision direction
            let rand1 = random(particle_seed + 1u) * 2.0 - 1.0;
            let rand2 = random(particle_seed + 2u) * 2.0 - 1.0;
            let rand3 = random(particle_seed + 3u) * 2.0 - 1.0;
            let rand4 = random(particle_seed + 4u);
            
            // Generate random velocity with much higher speeds for proper delta time
            let speed = 30.0 + rand4 * 60.0; // Random speed between 30-90 (4x+ increase for dramatic effect)
            
            // Create a more dramatic burst with wider spread
            let random_direction = normalize(vec3<f32>(
                rand1 * 1.5,           // Wider horizontal spread
                abs(rand2) * 0.8 + 0.3, // Mix of upward and horizontal (0.3 to 1.1 range)  
                rand3 * 1.5            // Wider horizontal spread
            ));
            
            // Apply the speed to the direction
            let final_velocity = random_direction * speed;
            
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
        
        // Fade out over lifetime using collision event color
        let life_ratio = particle.life / particle.max_life;
        output.color = vec4<f32>(particle.color.rgb, particle.color.a * life_ratio);
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
