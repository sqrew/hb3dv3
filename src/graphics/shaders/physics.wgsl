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
const BINDING_FORCE_STRENGTH: f32 = 50000.0; // Soft binding to origin
const BINDING_DISTANCE_THRESHOLD: f32 = 1.0; // Start binding beyond this distance

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
        let distance = sqrt(distance_squared);
        
        // Check for collision (overlapping bodies)
        let collision_distance = current_body.radius + other_body.radius;
        if (distance < collision_distance && distance > 0.001) {
            // Collision detected - apply elastic collision response
            let collision_normal = displacement / distance;
            
            // Calculate relative velocity along collision normal
            let relative_velocity = other_body.velocity - current_body.velocity;
            let velocity_along_normal = dot(relative_velocity, collision_normal);
            
            // Don't resolve if objects are separating
            if (velocity_along_normal > 0.0) {
                continue;
            }
            
            // Calculate collision impulse with gentler response for orbital motion
            let total_mass = current_body.mass + other_body.mass;
            
            // Check if this is more of a grazing/orbital collision vs head-on
            let relative_speed = length(relative_velocity);
            let normal_speed = abs(velocity_along_normal);
            let tangential_ratio = 1.0 - (normal_speed / max(relative_speed, 0.001));
            
            // Reduce collision strength for orbital/grazing collisions
            let collision_strength = mix(1.0, 0.3, tangential_ratio); // 30% strength for pure tangential
            let impulse_magnitude = -2.0 * velocity_along_normal * collision_strength / total_mass;
            let impulse = collision_normal * impulse_magnitude;
            
            // Apply impulse to both bodies (Newton's third law)
            nbody_bodies[body_index].velocity -= impulse * other_body.mass;
            nbody_bodies[other_idx].velocity += impulse * current_body.mass;
            
            // Also apply position correction to prevent overlap
            let overlap = collision_distance - distance;
            let mass_ratio_current = current_body.mass / total_mass;
            let mass_ratio_other = other_body.mass / total_mass;
            let separation_current = collision_normal * (overlap * mass_ratio_other);
            let separation_other = collision_normal * (overlap * mass_ratio_current);
            
            nbody_bodies[body_index].position -= separation_current;
            nbody_bodies[other_idx].position += separation_other;
        } else if (distance_squared < MIN_DISTANCE_SQUARED) {
            // Avoid singularities for very close but non-colliding bodies
            continue;
        } else {
            // Normal gravitational force calculation
            let force_magnitude = (GRAVITATIONAL_CONSTANT * other_body.mass * current_body.mass) / distance_squared;
            let force_direction = displacement / distance;
            
            total_force += force_direction * force_magnitude;
        }
    }
    
    // Add soft binding force to keep bodies near origin
    let distance_from_origin = length(current_body.position);
    if (distance_from_origin > BINDING_DISTANCE_THRESHOLD) {
        let binding_direction = -normalize(current_body.position); // Toward origin
        let excess_distance = distance_from_origin - BINDING_DISTANCE_THRESHOLD;
        let binding_force_magnitude = BINDING_FORCE_STRENGTH * excess_distance;
        let binding_force = binding_direction * binding_force_magnitude;
        total_force += binding_force;
    }

    // Apply force to update velocity (F = ma, so a = F/m)
    let acceleration = total_force / current_body.mass;
    nbody_bodies[body_index].velocity += acceleration * delta_time;
    
    // Update position using velocity
    nbody_bodies[body_index].position += nbody_bodies[body_index].velocity * delta_time;
    
    // Optional: Apply some damping to prevent runaway velocities
    nbody_bodies[body_index].velocity *= 0.999;
}
