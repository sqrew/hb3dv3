use nalgebra::{Matrix4, Vector3};
use crate::graphics::{Vec3, Transform};

#[derive(Debug, Clone)]
pub struct Plane {
    pub normal: Vector3<f32>,
    pub distance: f32,
}

impl Plane {
    pub fn new(normal: Vector3<f32>, distance: f32) -> Self {
        Self { normal, distance }
    }
    
    pub fn from_coefficients(a: f32, b: f32, c: f32, d: f32) -> Self {
        let normal = Vector3::new(a, b, c);
        let length = normal.magnitude();
        Self {
            normal: normal / length,
            distance: d / length,
        }
    }
    
    pub fn distance_to_point(&self, point: &Vector3<f32>) -> f32 {
        self.normal.dot(point) + self.distance
    }
}

#[derive(Debug, Clone)]
pub struct Frustum {
    pub planes: [Plane; 6], // left, right, bottom, top, near, far
}

impl Frustum {
    pub fn from_view_proj_matrix(view_proj: &Matrix4<f32>) -> Self {
        let m = view_proj;
        
        // Extract frustum planes from view-projection matrix
        // Based on "Fast Extraction of Viewing Frustum Planes from the WorldView-Projection Matrix"
        let planes = [
            // Left plane: m[3] + m[0]
            Plane::from_coefficients(
                m[3] + m[0], m[7] + m[4], m[11] + m[8], m[15] + m[12]
            ),
            // Right plane: m[3] - m[0] 
            Plane::from_coefficients(
                m[3] - m[0], m[7] - m[4], m[11] - m[8], m[15] - m[12]
            ),
            // Bottom plane: m[3] + m[1]
            Plane::from_coefficients(
                m[3] + m[1], m[7] + m[5], m[11] + m[9], m[15] + m[13]
            ),
            // Top plane: m[3] - m[1]
            Plane::from_coefficients(
                m[3] - m[1], m[7] - m[5], m[11] - m[9], m[15] - m[13]
            ),
            // Near plane: m[3] + m[2]
            Plane::from_coefficients(
                m[3] + m[2], m[7] + m[6], m[11] + m[10], m[15] + m[14]
            ),
            // Far plane: m[3] - m[2]
            Plane::from_coefficients(
                m[3] - m[2], m[7] - m[6], m[11] - m[10], m[15] - m[14]
            ),
        ];
        
        Self { planes }
    }
    
    pub fn contains_point(&self, point: &Vector3<f32>) -> bool {
        for plane in &self.planes {
            if plane.distance_to_point(point) < 0.0 {
                return false;
            }
        }
        true
    }
    
    pub fn contains_sphere(&self, center: &Vector3<f32>, radius: f32) -> bool {
        for plane in &self.planes {
            if plane.distance_to_point(center) < -radius {
                return false;
            }
        }
        true
    }
    
    pub fn contains_aabb(&self, min: &Vector3<f32>, max: &Vector3<f32>) -> bool {
        for plane in &self.planes {
            // Test all 8 corners of the AABB
            let corners = [
                Vector3::new(min.x, min.y, min.z),
                Vector3::new(max.x, min.y, min.z),
                Vector3::new(min.x, max.y, min.z),
                Vector3::new(max.x, max.y, min.z),
                Vector3::new(min.x, min.y, max.z),
                Vector3::new(max.x, min.y, max.z),
                Vector3::new(min.x, max.y, max.z),
                Vector3::new(max.x, max.y, max.z),
            ];
            
            // If all corners are behind this plane, the box is outside
            let mut all_behind = true;
            for corner in &corners {
                if plane.distance_to_point(corner) >= 0.0 {
                    all_behind = false;
                    break;
                }
            }
            
            if all_behind {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Clone)]
pub struct BoundingSphere {
    pub center: Vector3<f32>,
    pub radius: f32,
}

impl BoundingSphere {
    pub fn new(center: Vector3<f32>, radius: f32) -> Self {
        Self { center, radius }
    }
    
    pub fn from_transform_and_scale(transform: &Transform, base_radius: f32) -> Self {
        let center = transform.position;
        
        // Use maximum scale component to determine radius
        let max_scale = transform.scale.x
            .max(transform.scale.y)
            .max(transform.scale.z);
        
        let radius = base_radius * max_scale;
        
        Self { center, radius }
    }
}

#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub min: Vector3<f32>,
    pub max: Vector3<f32>,
}

impl BoundingBox {
    pub fn new(min: Vector3<f32>, max: Vector3<f32>) -> Self {
        Self { min, max }
    }
    
    pub fn from_transform_and_extents(transform: &Transform, half_extents: Vec3) -> Self {
        let center = transform.position;
        
        let scaled_extents = Vector3::new(
            half_extents.x * transform.scale.x,
            half_extents.y * transform.scale.y,
            half_extents.z * transform.scale.z,
        );
        
        Self {
            min: center - scaled_extents,
            max: center + scaled_extents,
        }
    }
    
    pub fn center(&self) -> Vector3<f32> {
        (self.min + self.max) * 0.5
    }
    
    pub fn extents(&self) -> Vector3<f32> {
        (self.max - self.min) * 0.5
    }
}

pub fn is_visible_sphere(frustum: &Frustum, transform: &Transform, base_radius: f32) -> bool {
    let bounding_sphere = BoundingSphere::from_transform_and_scale(transform, base_radius);
    frustum.contains_sphere(&bounding_sphere.center, bounding_sphere.radius)
}

pub fn is_visible_aabb(frustum: &Frustum, transform: &Transform, half_extents: Vec3) -> bool {
    let bounding_box = BoundingBox::from_transform_and_extents(transform, half_extents);
    frustum.contains_aabb(&bounding_box.min, &bounding_box.max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::{Matrix4, Point3};
    use crate::graphics::Transform;
    
    #[test]
    fn test_frustum_point_containment() {
        // Create a simple frustum (identity view-proj for testing)
        let view_proj = Matrix4::identity();
        let frustum = Frustum::from_view_proj_matrix(&view_proj);
        
        // Test points
        let origin = Vector3::new(0.0, 0.0, 0.0);
        assert!(frustum.contains_point(&origin));
    }
    
    #[test]
    fn test_bounding_sphere_creation() {
        let transform = Transform {
            position: Vec3::new(1.0, 2.0, 3.0),
            rotation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(2.0, 2.0, 2.0),
        };
        
        let sphere = BoundingSphere::from_transform_and_scale(&transform, 1.0);
        
        assert_eq!(sphere.center, Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(sphere.radius, 2.0); // base_radius * max_scale
    }
    
    #[test]
    fn test_bounding_box_creation() {
        let transform = Transform {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Vec3::new(0.0, 0.0, 0.0),  
            scale: Vec3::new(2.0, 1.0, 3.0),
        };
        
        let half_extents = Vec3::new(1.0, 1.0, 1.0);
        let bbox = BoundingBox::from_transform_and_extents(&transform, half_extents);
        
        assert_eq!(bbox.min, Vector3::new(-2.0, -1.0, -3.0));
        assert_eq!(bbox.max, Vector3::new(2.0, 1.0, 3.0));
    }
}