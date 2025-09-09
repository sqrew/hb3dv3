/// Simple data structures for submitting primitives to the graphics library

use crate::graphics::{Vec3, Color, PrimitiveType};

/// A single primitive to be rendered
#[derive(Debug, Clone)]
pub struct Primitive {
    /// World position of the primitive
    pub position: Vec3,
    
    /// Rotation in Euler angles (radians)
    pub rotation: Vec3,
    
    /// Scale factors for each axis
    pub scale: Vec3,
    
    /// Type of primitive geometry
    pub primitive_type: PrimitiveType,
    
    /// Color of the primitive
    pub color: Color,
}

impl Primitive {
    /// Create a new primitive at the given position
    pub fn new(primitive_type: PrimitiveType, position: Vec3, color: Color) -> Self {
        Self {
            position,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
            primitive_type,
            color,
        }
    }
    
    /// Set the rotation of this primitive (Euler angles in radians)
    pub fn with_rotation(mut self, rotation: Vec3) -> Self {
        self.rotation = rotation;
        self
    }
    
    /// Set the scale of this primitive
    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }
    
    /// Set uniform scale for this primitive
    pub fn with_uniform_scale(mut self, scale: f32) -> Self {
        self.scale = Vec3::new(scale, scale, scale);
        self
    }
}