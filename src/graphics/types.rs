/// Common types used throughout the graphics module
use nalgebra::{Vector3, Matrix4, Point3};

pub type Vec3 = Vector3<f32>;
pub type Mat4 = Matrix4<f32>;
pub type Point3f = Point3<f32>;

/// Transform component for positioning objects in 3D space
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3, // Euler angles in radians
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }
}

/// Simple entity identifier for particle systems
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub u64);

impl EntityId {
    pub fn new(id: u64) -> Self {
        EntityId(id)
    }
}

