// Instanced primitive rendering shader
// Renders primitives (tetrahedrons, spheres, etc.) with per-instance transforms and colors

// Camera uniform buffer
struct CameraUniform {
    view_proj: mat4x4<f32>,
    position: vec3<f32>,
    time: f32,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

// Vertex input from primitive mesh
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
}

// Instance input (per-particle data)
struct InstanceInput {
    @location(2) transform_0: vec4<f32>,
    @location(3) transform_1: vec4<f32>,
    @location(4) transform_2: vec4<f32>,
    @location(5) transform_3: vec4<f32>,
    @location(6) instance_color: vec4<f32>,
}

// Vertex shader output
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) world_position: vec3<f32>,
}

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    // Reconstruct transform matrix from instance data
    let transform_matrix = mat4x4<f32>(
        instance.transform_0,
        instance.transform_1,
        instance.transform_2,
        instance.transform_3,
    );
    
    // Transform vertex position to world space
    let world_position = transform_matrix * vec4<f32>(vertex.position, 1.0);
    
    var out: VertexOutput;
    out.clip_position = camera.view_proj * world_position;
    out.world_position = world_position.xyz;
    
    // Blend vertex color with instance color
    out.color = vertex.color * instance.instance_color;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Simple distance-based alpha falloff for particle effect
    let distance_to_camera = length(camera.position - in.world_position);
    let alpha_falloff = 1.0 - smoothstep(500.0, 1000.0, distance_to_camera);
    
    var final_color = in.color;
    final_color.a *= alpha_falloff;
    
    return final_color;
}