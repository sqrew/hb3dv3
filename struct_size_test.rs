// Quick test to verify struct sizes
use std::mem;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct GpuParticle {
    position: [f32; 3],      // 12 bytes
    mass: f32,               // 4 bytes
    velocity: [f32; 3],      // 12 bytes
    _padding: f32,           // 4 bytes
    life: f32,               // 4 bytes
    max_life: f32,           // 4 bytes
    _padding2: [f32; 2],     // 8 bytes
    color: [f32; 4],         // 16 bytes
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct SpawnRequest {
    position: [f32; 3],      // 12 bytes
    count: u32,              // 4 bytes
    velocity: [f32; 3],      // 12 bytes
    lifetime: f32,           // 4 bytes
    color: [f32; 4],         // 16 bytes
    mass: f32,               // 4 bytes
    _padding: [f32; 3],      // 12 bytes
}

fn main() {
    println!("GpuParticle size: {} bytes", mem::size_of::<GpuParticle>());
    println!("SpawnRequest size: {} bytes", mem::size_of::<SpawnRequest>());
    
    // Expected sizes for WGSL alignment:
    // Particle: position(16) + mass(4) + velocity(16) + padding(4) + life(4) + max_life(4) + padding2(8) + color(16) = 72 bytes
    // SpawnRequest: position(16) + count(4) + velocity(16) + lifetime(4) + color(16) + mass(4) + padding(12) = 72 bytes
    
    println!("Expected WGSL Particle size: 72 bytes");
    println!("Expected WGSL SpawnRequest size: 72 bytes");
}