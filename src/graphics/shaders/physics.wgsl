// Physics compute shaders for gravitational simulation

// Gravitational body structure
struct GravitationalBody {
    position: vec3<f32>,
    mass: f32,
    velocity: vec3<f32>,
    radius: f32,
}

// Gravity-affected object structure
struct AffectedObject {
    position: vec3<f32>,
    mass: f32,
    force: vec3<f32>,      // Output: computed gravitational force
    padding: f32,
}

// Constants
const GRAVITATIONAL_CONSTANT: f32 = 6.674e-1; // Scaled for game physics
const MIN_DISTANCE_SQUARED: f32 = 0.01; // Prevent singularities

// Gravity force computation bindings
@group(0) @binding(0) var<storage, read> gravitational_bodies: array<GravitationalBody>;
@group(0) @binding(1) var<storage, read_write> affected_objects: array<AffectedObject>;
@group(0) @binding(2) var<uniform> body_count: u32;
@group(0) @binding(3) var<uniform> affected_count: u32;

// N-body simulation bindings  
@group(0) @binding(0) var<storage, read_write> nbody_bodies: array<GravitationalBody>;
@group(0) @binding(1) var<uniform> nbody_body_count: u32;
@group(0) @binding(2) var<uniform> delta_time: f32;

// Compute gravitational forces acting on affected objects
@compute @workgroup_size(64)
fn compute_gravity_forces(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let obj_index = global_id.x;
    if (obj_index >= affected_count) { return; }
    
    var total_force = vec3<f32>(0.0, 0.0, 0.0);
    let obj_pos = affected_objects[obj_index].position;
    let obj_mass = affected_objects[obj_index].mass;
    
    // Calculate force from each gravitational body
    for (var body_idx = 0u; body_idx < body_count; body_idx++) {
        let body = gravitational_bodies[body_idx];
        let displacement = body.position - obj_pos;
        let distance_squared = dot(displacement, displacement);
        
        // Avoid singularities and unrealistic forces at very close distances
        if (distance_squared < MIN_DISTANCE_SQUARED) {
            continue;
        }
        
        let distance = sqrt(distance_squared);
        let force_magnitude = (GRAVITATIONAL_CONSTANT * body.mass * obj_mass) / distance_squared;
        let force_direction = displacement / distance;
        
        total_force += force_direction * force_magnitude;
    }
    
    // Store computed force
    affected_objects[obj_index].force = total_force;
}

// N-body simulation for gravitational bodies affecting each other
@compute @workgroup_size(64)
fn update_gravitational_bodies(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let body_index = global_id.x;
    if (body_index >= nbody_body_count) { return; }
    
    var total_force = vec3<f32>(0.0, 0.0, 0.0);
    let current_body = nbody_bodies[body_index];
    
    // Calculate forces from all other gravitational bodies
    for (var other_idx = 0u; other_idx < nbody_body_count; other_idx++) {
        if (other_idx == body_index) { continue; } // Don't apply force to self
        
        let other_body = nbody_bodies[other_idx];
        let displacement = other_body.position - current_body.position;
        let distance_squared = dot(displacement, displacement);
        
        // Avoid singularities
        if (distance_squared < MIN_DISTANCE_SQUARED) {
            continue;
        }
        
        let distance = sqrt(distance_squared);
        let force_magnitude = (GRAVITATIONAL_CONSTANT * other_body.mass * current_body.mass) / distance_squared;
        let force_direction = displacement / distance;
        
        total_force += force_direction * force_magnitude;
    }
    
    // Apply force to update velocity (F = ma, so a = F/m)
    let acceleration = total_force / current_body.mass;
    nbody_bodies[body_index].velocity += acceleration * delta_time;
    
    // Update position using velocity
    nbody_bodies[body_index].position += nbody_bodies[body_index].velocity * delta_time;
    
    // Optional: Apply some damping to prevent runaway velocities
    nbody_bodies[body_index].velocity *= 0.999;
}