// Line Rendering Shader - Instanced cylindrical lines for wireframes

struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

// Vertex data for the base cylinder mesh
struct CylinderVertex {
    @location(0) position: vec3<f32>,  // Position on unit cylinder
    @location(1) normal: vec3<f32>,    // Normal for lighting (optional)
}

// Per-line instance data
struct LineInstance {
    @location(2) start_pos: vec3<f32>,     // World space start position
    @location(3) thickness: f32,           // Line thickness
    @location(4) end_pos: vec3<f32>,       // World space end position  
    @location(5) color: vec4<f32>,         // Line color with alpha
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) glow_factor: f32,
}

@vertex
fn vs_main(
    vertex: CylinderVertex,
    instance: LineInstance,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Calculate line direction and length
    let line_dir = instance.end_pos - instance.start_pos;
    let line_length = length(line_dir);
    let line_normalized = normalize(line_dir);
    
    // Create basis vectors for the cylinder transformation
    // We need to rotate the cylinder to align with the line direction
    let up = select(
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(1.0, 0.0, 0.0),
        abs(line_normalized.y) > 0.99
    );
    
    let right = normalize(cross(up, line_normalized));
    let forward = normalize(cross(line_normalized, right));
    
    // Scale cylinder vertex by thickness and length
    var cylinder_pos = vertex.position;
    cylinder_pos.x *= instance.thickness * 0.5;  // Radius = thickness * 0.5 (thicker lines)
    cylinder_pos.z *= instance.thickness * 0.5;  // Radius = thickness * 0.5 (thicker lines)
    cylinder_pos.y *= line_length;  // Stretch along Y axis (triangular prism is Y-oriented)
    
    // Transform cylinder vertex to world space
    // Build rotation matrix from basis vectors
    // Y-axis oriented triangular prism: line_normalized maps to Y column
    let rotation_matrix = mat3x3<f32>(
        right,
        line_normalized,  // Y-axis column gets line direction
        forward
    );
    
    let rotated_pos = rotation_matrix * cylinder_pos;
    // Offset so line starts at start_pos instead of being centered there
    // Triangular prism goes from -0.5 to +0.5, so shift by +0.5 to start at 0
    let line_center = instance.start_pos + line_normalized * (line_length * 0.5);
    let world_pos = line_center + rotated_pos;
    
    // Transform normal for lighting
    let world_normal = rotation_matrix * vertex.normal;
    
    // Project to clip space
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 1.0);
    out.world_position = world_pos;
    out.color = instance.color;
    out.normal = world_normal;
    
    // Glow intensity based on line thickness (thicker lines glow more)
    out.glow_factor = smoothstep(0.5, 2.0, instance.thickness);
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Basic lighting (optional - can be removed for pure emissive look)
    let light_dir = normalize(vec3<f32>(0.3, 1.0, 0.5));
    let ndotl = max(dot(in.normal, light_dir), 0.0);
    let ambient = 0.3;
    let diffuse = 0.7 * ndotl;
    
    // Emissive glow effect - balanced for neon tubes
    let emission = in.color.rgb * (1.5 + in.glow_factor * 3.0);
    
    // Combine lighting and emission - more emission heavy
    let final_color = in.color.rgb * (ambient + diffuse) * 0.4 + emission * 0.7;
    
    // Add slight bloom hint by boosting bright colors
    let brightness = dot(final_color, vec3<f32>(0.299, 0.587, 0.114));
    let bloom_boost = smoothstep(0.5, 1.0, brightness) * 0.3;
    
    return vec4<f32>(final_color * (1.0 + bloom_boost), in.color.a);
}

// Alternative fragment shader for pure emissive wireframes (Geometry Wars style)
@fragment  
fn fs_emissive(in: VertexOutput) -> @location(0) vec4<f32> {
    // Pure emissive glow, no lighting - balanced for neon effect
    let emission = in.color.rgb * (2.2 + in.glow_factor * 4.0);
    
    // Edge glow effect - brighter at cylinder edges for neon tube effect
    let edge_factor = 1.0 - abs(in.normal.z);  // Assuming cylinder aligned with Z
    let edge_glow = edge_factor * 0.8;  // Balanced edge glow
    
    return vec4<f32>(emission * (1.0 + edge_glow), in.color.a);
}
