//! Primitive mesh generation for rendering
//! 
//! This module contains functions for generating basic primitive meshes
//! like cubes, spheres, cylinders, etc. for wireframe rendering.
//! 
//! Note: Most wireframe generation is now handled by primitive_cache.rs
//! for better performance with instanced rendering.

use super::vertex::{LineInstance, CylinderVertex};
use crate::graphics::{PrimitiveType, Color, Vec3};

/// Generate line instances directly for a primitive type (optimized)
/// This function is currently unused but kept for potential future use.
pub fn generate_primitive_line_instances(
    primitive_type: PrimitiveType,
    world_position: Vec3,
    scale: Vec3,
    color: Color,
    thickness: f32,
) -> Vec<LineInstance> {
    match primitive_type {
        PrimitiveType::Cube => generate_cube_line_instances(world_position, scale, color, thickness),
        PrimitiveType::Sphere => generate_sphere_line_instances(world_position, scale, color, thickness),
        PrimitiveType::Cylinder => generate_cylinder_line_instances(world_position, scale, color, thickness),
        PrimitiveType::Pyramid => generate_pyramid_line_instances(world_position, scale, color, thickness),
        PrimitiveType::Tetrahedron => generate_tetrahedron_line_instances(world_position, scale, color, thickness),
        PrimitiveType::Cone => generate_cone_line_instances(world_position, scale, color, thickness),
        PrimitiveType::Torus => generate_torus_line_instances(world_position, scale, color, thickness),
        PrimitiveType::Ellipsoid => generate_sphere_line_instances(world_position, scale, color, thickness), // Ellipsoid uses sphere with scaling
        PrimitiveType::Octahedron => generate_octahedron_line_instances(world_position, scale, color, thickness),
        PrimitiveType::Icosahedron => generate_icosahedron_line_instances(world_position, scale, color, thickness),
        PrimitiveType::Dodecahedron => generate_dodecahedron_line_instances(world_position, scale, color, thickness),
        PrimitiveType::Capsule => generate_capsule_line_instances(world_position, scale, color, thickness),
        PrimitiveType::Plane => generate_plane_line_instances(world_position, scale, color, thickness),
        PrimitiveType::Hemisphere => generate_hemisphere_line_instances(world_position, scale, color, thickness),
        // 2D Primitives
        PrimitiveType::Circle2D => generate_circle2d_line_instances(world_position, scale, color, thickness),
        PrimitiveType::Square2D => generate_square2d_line_instances(world_position, scale, color, thickness),
        PrimitiveType::Triangle2D => generate_triangle2d_line_instances(world_position, scale, color, thickness),
        PrimitiveType::Pentagon2D => generate_pentagon2d_line_instances(world_position, scale, color, thickness),
        PrimitiveType::Hexagon2D => generate_hexagon2d_line_instances(world_position, scale, color, thickness),
        PrimitiveType::Diamond2D => generate_diamond2d_line_instances(world_position, scale, color, thickness),
        PrimitiveType::Cross2D => generate_cross2d_line_instances(world_position, scale, color, thickness),
        PrimitiveType::Star2D => generate_star2d_line_instances(world_position, scale, color, thickness),
        PrimitiveType::Arrow2D => generate_arrow2d_line_instances(world_position, scale, color, thickness),
    }
}

// Stub implementations - these functions are referenced above but not actually used
// They're kept as stubs to avoid compilation errors, but the real wireframe 
// generation now happens in primitive_cache.rs for better performance

fn generate_cube_line_instances(_world_pos: Vec3, _scale: Vec3, _color: Color, _thickness: f32) -> Vec<LineInstance> {
    Vec::new()
}

fn generate_sphere_line_instances(_world_pos: Vec3, _scale: Vec3, _color: Color, _thickness: f32) -> Vec<LineInstance> {
    Vec::new()
}

fn generate_cylinder_line_instances(_world_pos: Vec3, _scale: Vec3, _color: Color, _thickness: f32) -> Vec<LineInstance> {
    Vec::new()
}

fn generate_pyramid_line_instances(_world_pos: Vec3, _scale: Vec3, _color: Color, _thickness: f32) -> Vec<LineInstance> {
    Vec::new()
}

fn generate_tetrahedron_line_instances(_world_pos: Vec3, _scale: Vec3, _color: Color, _thickness: f32) -> Vec<LineInstance> {
    Vec::new()
}

fn generate_cone_line_instances(_world_pos: Vec3, _scale: Vec3, _color: Color, _thickness: f32) -> Vec<LineInstance> {
    Vec::new()
}

fn generate_torus_line_instances(_world_pos: Vec3, _scale: Vec3, _color: Color, _thickness: f32) -> Vec<LineInstance> {
    Vec::new()
}

fn generate_octahedron_line_instances(_world_pos: Vec3, _scale: Vec3, _color: Color, _thickness: f32) -> Vec<LineInstance> {
    Vec::new()
}

fn generate_icosahedron_line_instances(_world_pos: Vec3, _scale: Vec3, _color: Color, _thickness: f32) -> Vec<LineInstance> {
    Vec::new()
}

fn generate_dodecahedron_line_instances(_world_pos: Vec3, _scale: Vec3, _color: Color, _thickness: f32) -> Vec<LineInstance> {
    Vec::new()
}

fn generate_capsule_line_instances(_world_pos: Vec3, _scale: Vec3, _color: Color, _thickness: f32) -> Vec<LineInstance> {
    Vec::new()
}

fn generate_plane_line_instances(_world_pos: Vec3, _scale: Vec3, _color: Color, _thickness: f32) -> Vec<LineInstance> {
    Vec::new()
}

pub fn generate_hemisphere_line_instances(_world_position: Vec3, _scale: Vec3, _color: Color, _thickness: f32) -> Vec<LineInstance> {
    Vec::new()
}

pub fn generate_circle2d_line_instances(_world_position: Vec3, _scale: Vec3, _color: Color, _thickness: f32) -> Vec<LineInstance> {
    Vec::new()
}

pub fn generate_square2d_line_instances(_world_position: Vec3, _scale: Vec3, _color: Color, _thickness: f32) -> Vec<LineInstance> {
    Vec::new()
}

pub fn generate_triangle2d_line_instances(_world_position: Vec3, _scale: Vec3, _color: Color, _thickness: f32) -> Vec<LineInstance> {
    Vec::new()
}

pub fn generate_pentagon2d_line_instances(_world_position: Vec3, _scale: Vec3, _color: Color, _thickness: f32) -> Vec<LineInstance> {
    Vec::new()
}

pub fn generate_hexagon2d_line_instances(_world_position: Vec3, _scale: Vec3, _color: Color, _thickness: f32) -> Vec<LineInstance> {
    Vec::new()
}

pub fn generate_diamond2d_line_instances(_world_position: Vec3, _scale: Vec3, _color: Color, _thickness: f32) -> Vec<LineInstance> {
    Vec::new()
}

pub fn generate_cross2d_line_instances(_world_position: Vec3, _scale: Vec3, _color: Color, _thickness: f32) -> Vec<LineInstance> {
    Vec::new()
}

pub fn generate_star2d_line_instances(_world_position: Vec3, _scale: Vec3, _color: Color, _thickness: f32) -> Vec<LineInstance> {
    Vec::new()
}

pub fn generate_arrow2d_line_instances(_world_position: Vec3, _scale: Vec3, _color: Color, _thickness: f32) -> Vec<LineInstance> {
    Vec::new()
}

/// Generate triangular prism mesh for line rendering (optimized)
/// This is used by the line renderer to create cylindrical-looking lines
/// Uses only 3 sides instead of 8+ for better performance with minimal visual difference
pub fn generate_line_cylinder() -> (Vec<CylinderVertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    // Define 3 points around a circle for triangular cross-section
    let angles: [f32; 3] = [0.0, 2.094395, 4.188787]; // 0°, 120°, 240° in radians
    
    // Generate vertices for bottom and top triangles
    for &theta in &angles {
        let x = theta.cos();
        let z = theta.sin();
        
        // Bottom vertex (y = -0.5)
        vertices.push(CylinderVertex {
            position: [x, -0.5, z],
            normal: [x, 0.0, z], // Radial normal pointing outward
        });
        
        // Top vertex (y = 0.5)  
        vertices.push(CylinderVertex {
            position: [x, 0.5, z],
            normal: [x, 0.0, z], // Radial normal pointing outward
        });
    }
    
    // Generate indices for the 3 rectangular faces of the triangular prism
    for side in 0..3 {
        let next_side = (side + 1) % 3;
        
        // Current side vertices
        let bottom_current = (side * 2) as u16;
        let top_current = (side * 2 + 1) as u16;
        
        // Next side vertices  
        let bottom_next = (next_side * 2) as u16;
        let top_next = (next_side * 2 + 1) as u16;
        
        // Two triangles per face (rectangular face split into 2 triangles)
        // Triangle 1: bottom_current, top_current, bottom_next
        indices.push(bottom_current);
        indices.push(top_current);  
        indices.push(bottom_next);
        
        // Triangle 2: top_current, top_next, bottom_next
        indices.push(top_current);
        indices.push(top_next);
        indices.push(bottom_next);
    }
    
    (vertices, indices)
}