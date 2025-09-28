// Physics compute shaders for gravitational simulation

// Gravitational body structure
struct GravitationalBody {
    position: vec3<f32>,
    _pad1: f32,                    // Explicit padding after vec3
    velocity: vec3<f32>,
    _pad2: f32,                    // Explicit padding after vec3
    radius: f32,
    mass: f32,
    angular_velocity: f32,
    ergosphere_radius: f32,        // Radius of frame-dragging effect
    frame_dragging_strength: f32,  // Strength of frame-dragging effect
    _pad3: f32,                    // Padding
    _pad4: f32,                    // Padding to 64-byte boundary
    _pad5: f32,                    // Final padding to 64-byte boundary
}

// Gravity-affected object structure
struct AffectedObject {
    position: vec3<f32>,
    mass: f32,
    force: vec3<f32>,      // Output: computed gravitational force
    padding: f32,
}

// Explosion structure
struct Explosion {
    position: vec3<f32>,
    current_radius: f32,
    force_strength: f32,
    falloff_type: u32,  // 0=Linear, 1=Quadratic, 2=Constant
    _pad1: f32,
    _pad2: f32,
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
@group(0) @binding(4) var<storage, read> explosions: array<Explosion>;
@group(0) @binding(5) var<uniform> explosion_count: u32;

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
        
        // For negative mass objects, force should be repulsive (away from gravitational body)
        // Since force_magnitude is already negative for negative mass, we need to flip direction
        let actual_force = force_direction * force_magnitude;
        if (obj_mass < 0.0) {
            total_force += -actual_force; // Flip direction for negative mass
        } else {
            total_force += actual_force;  // Normal direction for positive mass
        }
        
        // Apply frame-dragging (ergosphere) effect for spinning bodies
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
            total_force += frame_drag_force;
            
            // Very subtle inward component - frame-dragging should mostly be tangential
            let spiral_factor = 0.02 * ergosphere_factor_squared;
            let inward_spiral_force = -radial_vector * orbital_acceleration * spiral_factor;
            total_force += inward_spiral_force;
        }
    }
    
    // Apply explosion forces
    for (var explosion_idx = 0u; explosion_idx < explosion_count; explosion_idx++) {
        let explosion = explosions[explosion_idx];
        let explosion_displacement = obj_pos - explosion.position;
        let explosion_distance = length(explosion_displacement);
        
        // Check if object is within explosion radius
        if (explosion_distance <= explosion.current_radius && explosion_distance > 0.001) {
            // Calculate explosion force direction (radial outward from explosion center)
            let explosion_direction = explosion_displacement / explosion_distance;
            
            // Calculate force magnitude based on falloff type
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
            }
            
            // Apply explosion force (outward from explosion center)
            let explosion_force = explosion_direction * force_magnitude;
            total_force += explosion_force;
        }
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
        
        // Check for collision (overlapping bodies) - reduced collision distance for closer visual contact
        let collision_distance = (current_body.radius + other_body.radius) * 0.9;
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
            
            // === ENHANCED ANGULAR MOMENTUM CONSERVATION ===

            // 1. Spin transfer (existing logic)
            let spin_difference = current_body.angular_velocity - other_body.angular_velocity;
            let angular_impulse_factor = collision_strength * 0.15; // Slightly stronger for more effect

            // Conservation of angular momentum - transfer spin based on mass ratios
            let current_moment_of_inertia = current_body.mass * current_body.radius * current_body.radius;
            let other_moment_of_inertia = other_body.mass * other_body.radius * other_body.radius;
            let total_moment = current_moment_of_inertia + other_moment_of_inertia;

            // Calculate angular momentum transfer for spin
            let angular_transfer = spin_difference * angular_impulse_factor;
            let current_angular_change = -angular_transfer * (other_moment_of_inertia / total_moment);
            let other_angular_change = angular_transfer * (current_moment_of_inertia / total_moment);

            // Apply spin changes
            nbody_bodies[body_index].angular_velocity += current_angular_change;
            nbody_bodies[other_idx].angular_velocity += other_angular_change;

            // 2. Orbital angular momentum from impact parameter (NEW!)
            // Calculate impact parameter (how "off-center" the collision is)
            let collision_center = (current_body.position + other_body.position) * 0.5;
            let impact_arm_current = current_body.position - collision_center;
            let impact_arm_other = other_body.position - collision_center;

            // Calculate tangential component of impact (creates orbital motion)
            let tangent_direction = normalize(cross(collision_normal, vec3<f32>(0.0, 1.0, 0.0)));
            let impact_strength = length(relative_velocity) * collision_strength * tangential_ratio * 0.2;

            // Apply sideways kicks based on impact parameter and mass ratios
            let tangential_kick_current = tangent_direction * impact_strength * (other_body.mass / total_mass);
            let tangential_kick_other = -tangent_direction * impact_strength * (current_body.mass / total_mass);

            // Add orbital angular momentum to velocities
            nbody_bodies[body_index].velocity += tangential_kick_current;
            nbody_bodies[other_idx].velocity += tangential_kick_other;
            
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
    
    // Add soft binding force to keep bodies near origin (including negative mass bodies)
    let distance_from_origin = length(current_body.position);
    var gravitational_acceleration = total_force / current_body.mass; // Normal physics for gravity
    
    if (distance_from_origin > BINDING_DISTANCE_THRESHOLD) {
        let binding_direction = -normalize(current_body.position); // Toward origin
        let excess_distance = distance_from_origin - BINDING_DISTANCE_THRESHOLD;
        let binding_force_magnitude = BINDING_FORCE_STRENGTH * excess_distance;
        let binding_force = binding_direction * binding_force_magnitude;
        
        // Use absolute mass for binding force only to prevent negative mass bodies from escaping
        let binding_acceleration = binding_force / abs(current_body.mass);
        gravitational_acceleration += binding_acceleration;
    }

    // Apply combined acceleration
    let acceleration = gravitational_acceleration;
    nbody_bodies[body_index].velocity += acceleration * delta_time;
    
    // Update position using velocity
    nbody_bodies[body_index].position += nbody_bodies[body_index].velocity * delta_time;
    
    // Optional: Apply some damping to prevent runaway velocities
    nbody_bodies[body_index].velocity *= 0.999;
}
