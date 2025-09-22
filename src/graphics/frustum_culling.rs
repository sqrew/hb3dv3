use crate::graphics::Transform;
use nalgebra::{Matrix4, Vector3};

#[derive(Debug, Clone)]
pub struct Plane {
    pub normal: Vector3<f32>,
    pub distance: f32,
}

impl Plane {
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
            Plane::from_coefficients(m[3] + m[0], m[7] + m[4], m[11] + m[8], m[15] + m[12]),
            // Right plane: m[3] - m[0]
            Plane::from_coefficients(m[3] - m[0], m[7] - m[4], m[11] - m[8], m[15] - m[12]),
            // Bottom plane: m[3] + m[1]
            Plane::from_coefficients(m[3] + m[1], m[7] + m[5], m[11] + m[9], m[15] + m[13]),
            // Top plane: m[3] - m[1]
            Plane::from_coefficients(m[3] - m[1], m[7] - m[5], m[11] - m[9], m[15] - m[13]),
            // Near plane: m[3] + m[2]
            Plane::from_coefficients(m[3] + m[2], m[7] + m[6], m[11] + m[10], m[15] + m[14]),
            // Far plane: m[3] - m[2]
            Plane::from_coefficients(m[3] - m[2], m[7] - m[6], m[11] - m[10], m[15] - m[14]),
        ];

        Self { planes }
    }

    pub fn contains_sphere(&self, center: &Vector3<f32>, radius: f32) -> bool {
        for plane in &self.planes {
            if plane.distance_to_point(center) < -radius {
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
    pub fn from_transform_and_scale(transform: &Transform, base_radius: f32) -> Self {
        let center = transform.position;

        // Use maximum scale component to determine radius
        let max_scale = transform
            .scale
            .x
            .max(transform.scale.y)
            .max(transform.scale.z);

        let radius = base_radius * max_scale;

        Self { center, radius }
    }
}

pub fn is_visible_sphere(frustum: &Frustum, transform: &Transform, base_radius: f32) -> bool {
    let bounding_sphere = BoundingSphere::from_transform_and_scale(transform, base_radius);
    frustum.contains_sphere(&bounding_sphere.center, bounding_sphere.radius)
}
