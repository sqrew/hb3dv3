use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

impl Vertex {
    pub const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x4,
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}


// Vertex structure for cylinder mesh (used for instanced line rendering)
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CylinderVertex {
    pub position: [f32; 3],  // Position on unit cylinder
    pub normal: [f32; 3],    // Normal for lighting
}

impl CylinderVertex {
    pub const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3, // position
        1 => Float32x3, // normal
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

// Per-line instance data for instanced rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct LineInstance {
    pub start_pos: [f32; 3],     // World space start position
    pub thickness: f32,          // Line thickness
    pub end_pos: [f32; 3],       // World space end position
    pub color: [f32; 4],         // Line color with alpha
}

impl LineInstance {
    pub const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
        2 => Float32x3, // start_pos
        3 => Float32,   // thickness
        4 => Float32x3, // end_pos
        5 => Float32x4, // color
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

// Per-primitive instance data for instanced primitive rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct PrimitiveInstance {
    // Transform matrix (4x4) stored as 4 vec4s
    pub transform_0: [f32; 4],
    pub transform_1: [f32; 4], 
    pub transform_2: [f32; 4],
    pub transform_3: [f32; 4],
    pub color: [f32; 4],       // RGBA color
}

impl PrimitiveInstance {
    pub const ATTRIBS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        2 => Float32x4, // transform_0
        3 => Float32x4, // transform_1
        4 => Float32x4, // transform_2
        5 => Float32x4, // transform_3
        6 => Float32x4, // color
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
    
    pub fn new(transform: [[f32; 4]; 4], color: [f32; 4]) -> Self {
        Self {
            transform_0: transform[0],
            transform_1: transform[1],
            transform_2: transform[2],
            transform_3: transform[3],
            color,
        }
    }
}