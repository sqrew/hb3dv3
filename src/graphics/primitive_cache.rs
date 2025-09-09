//! Primitive line cache for high-performance wireframe rendering
//!
//! Pre-computes line segments for each primitive type to eliminate
//! runtime vertex generation and reduce allocations.

use super::constants::*;
use super::vertex::LineInstance;
use crate::graphics::{
    PrimitiveType, Color, Vec3,
    constants::{TORUS_MAJOR_RADIUS_RATIO, TORUS_MINOR_RADIUS_RATIO},
};
use rayon::prelude::*;

/// Pre-computed line segments for a primitive type
#[derive(Debug, Clone)]
pub struct PrimitiveLineData {
    /// Line segments as (start, end) position pairs in local space
    pub line_segments: Vec<([f32; 3], [f32; 3])>,
    /// Number of line segments (for capacity pre-allocation)
    pub line_count: usize,
}

impl PrimitiveLineData {
    pub fn new(line_segments: Vec<([f32; 3], [f32; 3])>) -> Self {
        let line_count = line_segments.len();
        Self {
            line_segments,
            line_count,
        }
    }
    
    /// Generate line instances with full transform support (position, rotation, scale)
    pub fn generate_instances_with_rotation(
        &self,
        world_position: Vec3,
        rotation: Vec3,
        scale: Vec3,
        color: Color,
        thickness: f32,
    ) -> Vec<LineInstance> {
        use nalgebra::{Matrix4, Vector3};
        
        let color_array = color.to_array4();
        let mut instances = Vec::with_capacity(self.line_count);
        
        // Create transform matrix: Translation * Rotation * Scale - optimized with direct Vector3 creation
        let translation_vec = Vector3::new(world_position.x, world_position.y, world_position.z);
        let scale_vec = Vector3::new(scale.x, scale.y, scale.z);
        let transform_matrix = Matrix4::new_translation(&translation_vec) 
            * Matrix4::from_euler_angles(rotation.x, rotation.y, rotation.z) 
            * Matrix4::new_nonuniform_scaling(&scale_vec);
        
        // Parallelize matrix transformations for large primitives (sphere, torus, icosahedron)
        if self.line_segments.len() > 50 {
            // Use parallel processing for complex primitives
            let parallel_instances: Vec<LineInstance> = self.line_segments
                .par_iter()
                .map(|(start_local, end_local)| {
                    // Apply full matrix transformation to each point - parallelized
                    let start_world = transform_matrix.transform_point(&nalgebra::Point3::new(start_local[0], start_local[1], start_local[2]));
                    let end_world = transform_matrix.transform_point(&nalgebra::Point3::new(end_local[0], end_local[1], end_local[2]));
                    
                    LineInstance {
                        start_pos: [start_world.x, start_world.y, start_world.z],
                        end_pos: [end_world.x, end_world.y, end_world.z],
                        thickness,
                        color: color_array,
                    }
                })
                .collect();
            instances.extend(parallel_instances);
        } else {
            // Use sequential processing for simple primitives to avoid Rayon overhead
            for (start_local, end_local) in &self.line_segments {
                // Apply full matrix transformation to each point - optimized to avoid intermediate objects
                let start_world = transform_matrix.transform_point(&nalgebra::Point3::new(start_local[0], start_local[1], start_local[2]));
                let end_world = transform_matrix.transform_point(&nalgebra::Point3::new(end_local[0], end_local[1], end_local[2]));
                
                instances.push(LineInstance {
                    start_pos: [start_world.x, start_world.y, start_world.z],
                    end_pos: [end_world.x, end_world.y, end_world.z],
                    thickness,
                    color: color_array,
                });
            }
        }
        
        instances
    }
    
    /// Generate line instances with only position and scale (legacy method)
    pub fn generate_instances(
        &self,
        world_position: Vec3,
        scale: Vec3,
        color: Color,
        thickness: f32,
    ) -> Vec<LineInstance> {
        // Use the rotation-aware method with zero rotation
        self.generate_instances_with_rotation(
            world_position,
            Vec3::zeros(), // No rotation
            scale,
            color,
            thickness,
        )
    }
}

/// Cache for all primitive line data
pub struct PrimitiveCache {
    cube: PrimitiveLineData,
    sphere: PrimitiveLineData,
    cylinder: PrimitiveLineData,
    pyramid: PrimitiveLineData,
    tetrahedron: PrimitiveLineData,
    cone: PrimitiveLineData,
    torus: PrimitiveLineData,
    octahedron: PrimitiveLineData,
    icosahedron: PrimitiveLineData,
    dodecahedron: PrimitiveLineData,
    capsule: PrimitiveLineData,
    plane: PrimitiveLineData,
    hemisphere: PrimitiveLineData,
    // 2D Primitives
    circle2d: PrimitiveLineData,
    square2d: PrimitiveLineData,
    triangle2d: PrimitiveLineData,
    pentagon2d: PrimitiveLineData,
    hexagon2d: PrimitiveLineData,
    diamond2d: PrimitiveLineData,
    cross2d: PrimitiveLineData,
    star2d: PrimitiveLineData,
    arrow2d: PrimitiveLineData,
}

impl PrimitiveCache {
    /// Initialize the cache with all primitive data
    pub fn new() -> Self {
        Self {
            cube: Self::generate_cube_data(),
            sphere: Self::generate_sphere_data(),
            cylinder: Self::generate_cylinder_data(),
            pyramid: Self::generate_pyramid_data(),
            tetrahedron: Self::generate_tetrahedron_data(),
            cone: Self::generate_cone_data(),
            torus: Self::generate_torus_data(),
            octahedron: Self::generate_octahedron_data(),
            icosahedron: Self::generate_icosahedron_data(),
            dodecahedron: Self::generate_dodecahedron_data(),
            capsule: Self::generate_capsule_data(),
            plane: Self::generate_plane_data(),
            hemisphere: Self::generate_hemisphere_data(),
            // 2D Primitives
            circle2d: Self::generate_circle2d_data(),
            square2d: Self::generate_square2d_data(),
            triangle2d: Self::generate_triangle2d_data(),
            pentagon2d: Self::generate_pentagon2d_data(),
            hexagon2d: Self::generate_hexagon2d_data(),
            diamond2d: Self::generate_diamond2d_data(),
            cross2d: Self::generate_cross2d_data(),
            star2d: Self::generate_star2d_data(),
            arrow2d: Self::generate_arrow2d_data(),
        }
    }
    
    /// Get line data for a specific primitive type
    pub fn get_primitive_data(&self, primitive_type: PrimitiveType) -> &PrimitiveLineData {
        let result = match primitive_type {
            PrimitiveType::Cube => &self.cube,
            PrimitiveType::Sphere => &self.sphere,
            PrimitiveType::Cylinder => &self.cylinder,
            PrimitiveType::Pyramid => &self.pyramid,
            PrimitiveType::Tetrahedron => &self.tetrahedron,
            PrimitiveType::Cone => &self.cone,
            PrimitiveType::Torus => &self.torus,
            PrimitiveType::Ellipsoid => &self.sphere, // Ellipsoid uses sphere with scaling
            PrimitiveType::Octahedron => &self.octahedron,
            PrimitiveType::Icosahedron => &self.icosahedron,
            PrimitiveType::Dodecahedron => &self.dodecahedron,
            PrimitiveType::Capsule => &self.capsule,
            PrimitiveType::Plane => &self.plane,
            PrimitiveType::Hemisphere => &self.hemisphere,
            // 2D Primitives
            PrimitiveType::Circle2D => &self.circle2d,
            PrimitiveType::Square2D => &self.square2d,
            PrimitiveType::Triangle2D => &self.triangle2d,
            PrimitiveType::Pentagon2D => &self.pentagon2d,
            PrimitiveType::Hexagon2D => &self.hexagon2d,
            PrimitiveType::Diamond2D => &self.diamond2d,
            PrimitiveType::Cross2D => &self.cross2d,
            PrimitiveType::Star2D => &self.star2d,
            PrimitiveType::Arrow2D => &self.arrow2d,
        };
        result
    }
    
    /// Generate line instances using cached data with rotation support
    pub fn generate_line_instances_with_rotation(
        &self,
        primitive_type: PrimitiveType,
        world_position: Vec3,
        rotation: Vec3,
        scale: Vec3,
        color: Color,
        thickness: f32,
    ) -> Vec<LineInstance> {
        self.get_primitive_data(primitive_type)
            .generate_instances_with_rotation(world_position, rotation, scale, color, thickness)
    }
    
    /// Generate line instances using cached data (legacy method without rotation)
    pub fn generate_line_instances(
        &self,
        primitive_type: PrimitiveType,
        world_position: Vec3,
        scale: Vec3,
        color: Color,
        thickness: f32,
    ) -> Vec<LineInstance> {
        self.get_primitive_data(primitive_type)
            .generate_instances(world_position, scale, color, thickness)
    }

    // === PRIMITIVE DATA GENERATORS ===

    fn generate_cube_data() -> PrimitiveLineData {
        let vertices = [
            [-0.5, -0.5,  0.5], // 0: front bottom left
            [ 0.5, -0.5,  0.5], // 1: front bottom right
            [ 0.5,  0.5,  0.5], // 2: front top right
            [-0.5,  0.5,  0.5], // 3: front top left
            [-0.5, -0.5, -0.5], // 4: back bottom left
            [ 0.5, -0.5, -0.5], // 5: back bottom right
            [ 0.5,  0.5, -0.5], // 6: back top right
            [-0.5,  0.5, -0.5], // 7: back top left
        ];
        
        let edges = [
            (0, 1), (1, 2), (2, 3), (3, 0), // Front face
            (4, 5), (5, 6), (6, 7), (7, 4), // Back face
            (0, 4), (1, 5), (2, 6), (3, 7), // Connecting edges
        ];
        
        let line_segments = edges.iter()
            .map(|(start_idx, end_idx)| (vertices[*start_idx], vertices[*end_idx]))
            .collect();
        
        PrimitiveLineData::new(line_segments)
    }

    fn generate_sphere_data() -> PrimitiveLineData {
        let mut line_segments = Vec::with_capacity(SPHERE_RINGS * SPHERE_SEGMENTS * 2);
        
        // Generate latitude rings
        for ring in 0..=SPHERE_RINGS {
            let theta = std::f32::consts::PI * (ring as f32) / (SPHERE_RINGS as f32);
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();
            let y = cos_theta * 0.5;
            
            for segment in 0..SPHERE_SEGMENTS {
                let phi = 2.0 * std::f32::consts::PI * (segment as f32) / (SPHERE_SEGMENTS as f32);
                let next_phi = 2.0 * std::f32::consts::PI * ((segment + 1) % SPHERE_SEGMENTS) as f32 / (SPHERE_SEGMENTS as f32);
                
                let x1 = sin_theta * phi.cos() * 0.5;
                let z1 = sin_theta * phi.sin() * 0.5;
                let x2 = sin_theta * next_phi.cos() * 0.5;
                let z2 = sin_theta * next_phi.sin() * 0.5;
                
                // Skip degenerate lines at poles
                if ring > 0 && ring < SPHERE_RINGS {
                    line_segments.push(([x1, y, z1], [x2, y, z2]));
                }
            }
        }
        
        // Generate longitude lines
        for segment in 0..SPHERE_SEGMENTS {
            let phi = 2.0 * std::f32::consts::PI * (segment as f32) / (SPHERE_SEGMENTS as f32);
            
            for ring in 0..SPHERE_RINGS {
                let theta1 = std::f32::consts::PI * (ring as f32) / (SPHERE_RINGS as f32);
                let theta2 = std::f32::consts::PI * ((ring + 1) as f32) / (SPHERE_RINGS as f32);
                
                let sin_theta1 = theta1.sin();
                let cos_theta1 = theta1.cos();
                let sin_theta2 = theta2.sin();
                let cos_theta2 = theta2.cos();
                
                let x1 = sin_theta1 * phi.cos() * 0.5;
                let y1 = cos_theta1 * 0.5;
                let z1 = sin_theta1 * phi.sin() * 0.5;
                
                let x2 = sin_theta2 * phi.cos() * 0.5;
                let y2 = cos_theta2 * 0.5;
                let z2 = sin_theta2 * phi.sin() * 0.5;
                
                line_segments.push(([x1, y1, z1], [x2, y2, z2]));
            }
        }
        
        PrimitiveLineData::new(line_segments)
    }

    fn generate_cylinder_data() -> PrimitiveLineData {
        let mut line_segments = Vec::with_capacity(CYLINDER_SEGMENTS * 3); // bottom + top + vertical
        
        // Generate bottom and top circle edges + vertical connections
        for i in 0..CYLINDER_SEGMENTS {
            let angle = 2.0 * std::f32::consts::PI * (i as f32) / (CYLINDER_SEGMENTS as f32);
            let next_angle = 2.0 * std::f32::consts::PI * ((i + 1) % CYLINDER_SEGMENTS) as f32 / (CYLINDER_SEGMENTS as f32);
            
            let x1 = angle.cos() * 0.5;
            let z1 = angle.sin() * 0.5;
            let x2 = next_angle.cos() * 0.5;
            let z2 = next_angle.sin() * 0.5;
            
            // Bottom circle edge
            line_segments.push(([x1, -0.5, z1], [x2, -0.5, z2]));
            // Top circle edge  
            line_segments.push(([x1, 0.5, z1], [x2, 0.5, z2]));
            // Vertical edge
            line_segments.push(([x1, -0.5, z1], [x1, 0.5, z1]));
        }
        
        PrimitiveLineData::new(line_segments)
    }

    fn generate_pyramid_data() -> PrimitiveLineData {
        let vertices = [
            [ 0.0,  0.5,  0.0], // 0: top
            [-0.5, -0.5,  0.5], // 1: base front left
            [ 0.5, -0.5,  0.5], // 2: base front right
            [ 0.5, -0.5, -0.5], // 3: base back right
            [-0.5, -0.5, -0.5], // 4: base back left
        ];
        
        let edges = [
            // Base edges
            (1, 2), (2, 3), (3, 4), (4, 1),
            // Edges to top
            (0, 1), (0, 2), (0, 3), (0, 4),
        ];
        
        let line_segments = edges.iter()
            .map(|(start_idx, end_idx)| (vertices[*start_idx], vertices[*end_idx]))
            .collect();
        
        PrimitiveLineData::new(line_segments)
    }

    fn generate_tetrahedron_data() -> PrimitiveLineData {
        let vertices = [
            [ 0.0,  TETRAHEDRON_HEIGHT,  0.0], // 0: top
            [-TETRAHEDRON_BASE_X_OFFSET, TETRAHEDRON_BASE_Y,  TETRAHEDRON_BASE_FRONT], // 1: base front left
            [ TETRAHEDRON_BASE_X_OFFSET, TETRAHEDRON_BASE_Y,  TETRAHEDRON_BASE_FRONT], // 2: base front right  
            [ 0.0, TETRAHEDRON_BASE_Y, TETRAHEDRON_BASE_BACK], // 3: base back
        ];
        
        let edges = [
            // Base triangle
            (1, 2), (2, 3), (3, 1),
            // Edges to top
            (0, 1), (0, 2), (0, 3),
        ];
        
        let line_segments = edges.iter()
            .map(|(start_idx, end_idx)| (vertices[*start_idx], vertices[*end_idx]))
            .collect();
        
        PrimitiveLineData::new(line_segments)
    }

    fn generate_cone_data() -> PrimitiveLineData {
        let mut line_segments = Vec::with_capacity(CONE_SEGMENTS * 2); // base circle + lines to apex
        
        let apex = [0.0, 0.5, 0.0];
        
        for i in 0..CONE_SEGMENTS {
            let angle = 2.0 * std::f32::consts::PI * (i as f32) / (CONE_SEGMENTS as f32);
            let next_angle = 2.0 * std::f32::consts::PI * ((i + 1) % CONE_SEGMENTS) as f32 / (CONE_SEGMENTS as f32);
            
            let x1 = angle.cos() * 0.5;
            let z1 = angle.sin() * 0.5;
            let x2 = next_angle.cos() * 0.5;
            let z2 = next_angle.sin() * 0.5;
            
            let base_point1 = [x1, -0.5, z1];
            let base_point2 = [x2, -0.5, z2];
            
            // Base circle edge
            line_segments.push((base_point1, base_point2));
            // Line to apex
            line_segments.push((base_point1, apex));
        }
        
        PrimitiveLineData::new(line_segments)
    }

    fn generate_torus_data() -> PrimitiveLineData {
        let major_radius = 0.5 * TORUS_MAJOR_RADIUS_RATIO;
        let minor_radius = 0.5 * TORUS_MINOR_RADIUS_RATIO;
        let mut line_segments = Vec::with_capacity(TORUS_MAJOR_SEGMENTS * TORUS_MINOR_SEGMENTS * 2);
        
        for i in 0..TORUS_MAJOR_SEGMENTS {
            let major_angle = (i as f32 / TORUS_MAJOR_SEGMENTS as f32) * 2.0 * std::f32::consts::PI;
            let next_major_angle = ((i + 1) % TORUS_MAJOR_SEGMENTS) as f32 / TORUS_MAJOR_SEGMENTS as f32 * 2.0 * std::f32::consts::PI;
            let major_cos = major_angle.cos();
            let major_sin = major_angle.sin();
            let next_major_cos = next_major_angle.cos();
            let next_major_sin = next_major_angle.sin();
            
            for j in 0..TORUS_MINOR_SEGMENTS {
                let minor_angle = (j as f32 / TORUS_MINOR_SEGMENTS as f32) * 2.0 * std::f32::consts::PI;
                let next_minor_angle = ((j + 1) % TORUS_MINOR_SEGMENTS) as f32 / TORUS_MINOR_SEGMENTS as f32 * 2.0 * std::f32::consts::PI;
                let minor_cos = minor_angle.cos();
                let minor_sin = minor_angle.sin();
                let next_minor_cos = next_minor_angle.cos();
                let next_minor_sin = next_minor_angle.sin();
                
                // Current point
                let x = (major_radius + minor_radius * minor_cos) * major_cos;
                let y = minor_radius * minor_sin;
                let z = (major_radius + minor_radius * minor_cos) * major_sin;
                
                // Next minor point (around tube)
                let x_next_minor = (major_radius + minor_radius * next_minor_cos) * major_cos;
                let y_next_minor = minor_radius * next_minor_sin;
                let z_next_minor = (major_radius + minor_radius * next_minor_cos) * major_sin;
                
                // Next major point (around main ring)
                let x_next_major = (major_radius + minor_radius * minor_cos) * next_major_cos;
                let y_next_major = minor_radius * minor_sin;
                let z_next_major = (major_radius + minor_radius * minor_cos) * next_major_sin;
                
                // Lines around the minor circles (tube cross-sections)
                line_segments.push(([x, y, z], [x_next_minor, y_next_minor, z_next_minor]));
                
                // Lines connecting major segments (along the main ring)
                line_segments.push(([x, y, z], [x_next_major, y_next_major, z_next_major]));
            }
        }
        
        PrimitiveLineData::new(line_segments)
    }

    fn generate_octahedron_data() -> PrimitiveLineData {
        let vertices = [
            // Top and bottom points
            [ 0.0,  0.5,  0.0], // 0: top
            [ 0.0, -0.5,  0.0], // 1: bottom
            
            // Middle ring - 4 points forming a diamond in XZ plane
            [ 0.5,  0.0,  0.0], // 2: +X
            [ 0.0,  0.0,  0.5], // 3: +Z
            [-0.5,  0.0,  0.0], // 4: -X
            [ 0.0,  0.0, -0.5], // 5: -Z
        ];
        
        let edges = [
            // Edges from top to middle ring
            (0, 2), (0, 3), (0, 4), (0, 5),
            // Edges from bottom to middle ring  
            (1, 2), (1, 3), (1, 4), (1, 5),
            // Edges around middle ring
            (2, 3), (3, 4), (4, 5), (5, 2),
        ];
        
        let line_segments = edges.iter()
            .map(|(start_idx, end_idx)| (vertices[*start_idx], vertices[*end_idx]))
            .collect();
        
        PrimitiveLineData::new(line_segments)
    }

    fn generate_icosahedron_data() -> PrimitiveLineData {
        let phi = GOLDEN_RATIO;
        let scale = 0.5;
        
        let vertices = [
            // 4 vertices on XZ plane
            [scale, 0.0, scale * phi],     // 0
            [-scale, 0.0, scale * phi],    // 1
            [scale, 0.0, -scale * phi],    // 2
            [-scale, 0.0, -scale * phi],   // 3
            
            // 4 vertices on YZ plane  
            [0.0, scale * phi, scale],     // 4
            [0.0, -scale * phi, scale],    // 5
            [0.0, scale * phi, -scale],    // 6
            [0.0, -scale * phi, -scale],   // 7
            
            // 4 vertices on XY plane
            [scale * phi, scale, 0.0],     // 8
            [-scale * phi, scale, 0.0],    // 9
            [scale * phi, -scale, 0.0],    // 10
            [-scale * phi, -scale, 0.0],   // 11
        ];
        
        let edges = [
            // Pentagonal faces around vertex 0
            (0, 1), (0, 4), (0, 5), (0, 8), (0, 10),
            // Pentagonal faces around vertex 3  
            (3, 2), (3, 6), (3, 7), (3, 9), (3, 11),
            // Triangular faces connecting the two pentagons
            (1, 4), (4, 6), (6, 2), (2, 10), (10, 8),
            (1, 5), (5, 7), (7, 11), (11, 9), (9, 6),
            // Remaining edges
            (1, 9), (4, 8), (5, 10), (7, 2), (8, 6), (11, 5),
        ];
        
        let line_segments = edges.iter()
            .map(|(start_idx, end_idx)| (vertices[*start_idx], vertices[*end_idx]))
            .collect();
        
        PrimitiveLineData::new(line_segments)
    }
    
    fn generate_dodecahedron_data() -> PrimitiveLineData {
        // Regular dodecahedron with 12 pentagonal faces
        let phi = GOLDEN_RATIO;
        let scale = 0.3;
        
        // 20 vertices of a dodecahedron using golden ratio relationships
        let vertices = [
            // 8 vertices of a cube
            [scale, scale, scale],
            [scale, scale, -scale],
            [scale, -scale, scale],
            [scale, -scale, -scale],
            [-scale, scale, scale],
            [-scale, scale, -scale],
            [-scale, -scale, scale],
            [-scale, -scale, -scale],
            
            // 12 vertices on faces of cube using golden ratio
            // XY faces (Z = 0)
            [0.0, scale * phi, scale / phi],
            [0.0, scale * phi, -scale / phi],
            [0.0, -scale * phi, scale / phi],
            [0.0, -scale * phi, -scale / phi],
            
            // XZ faces (Y = 0)  
            [scale * phi, scale / phi, 0.0],
            [scale * phi, -scale / phi, 0.0],
            [-scale * phi, scale / phi, 0.0],
            [-scale * phi, -scale / phi, 0.0],
            
            // YZ faces (X = 0)
            [scale / phi, 0.0, scale * phi],
            [-scale / phi, 0.0, scale * phi],
            [scale / phi, 0.0, -scale * phi],
            [-scale / phi, 0.0, -scale * phi],
        ];
        
        // 30 edges of the dodecahedron connecting adjacent vertices
        let edges = [
            // Connect cube vertices to golden ratio points
            (0, 8), (0, 12), (0, 16),   // vertex 0 connections
            (1, 9), (1, 12), (1, 18),   // vertex 1 connections  
            (2, 10), (2, 13), (2, 16),  // vertex 2 connections
            (3, 11), (3, 13), (3, 18),  // vertex 3 connections
            (4, 8), (4, 14), (4, 17),   // vertex 4 connections
            (5, 9), (5, 14), (5, 19),   // vertex 5 connections
            (6, 10), (6, 15), (6, 17),  // vertex 6 connections
            (7, 11), (7, 15), (7, 19),  // vertex 7 connections
            
            // Connect golden ratio points to form pentagonal faces
            (8, 9), (9, 14), (14, 15), (15, 10), (10, 8),     // Pentagon face
            (12, 13), (13, 18), (18, 19), (19, 16), (16, 12), // Pentagon face  
        ];
        
        let line_segments = edges.iter()
            .map(|(start_idx, end_idx)| (vertices[*start_idx], vertices[*end_idx]))
            .collect();
        
        PrimitiveLineData::new(line_segments)
    }
    
    fn generate_capsule_data() -> PrimitiveLineData {
        // Capsule: cylinder with hemispherical caps
        let cylinder_height = 0.4;
        let radius = 0.25;
        let segments = 8;
        let hemisphere_rings = 4;
        
        let mut line_segments = Vec::new();
        
        // Main cylinder part
        for i in 0..segments {
            let angle = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
            let next_angle = 2.0 * std::f32::consts::PI * ((i + 1) % segments) as f32 / segments as f32;
            
            let (x1, z1) = (angle.cos() * radius, angle.sin() * radius);
            let (x2, z2) = (next_angle.cos() * radius, next_angle.sin() * radius);
            
            // Cylinder body vertical lines
            line_segments.push(([x1, -cylinder_height, z1], [x1, cylinder_height, z1]));
            
            // Top and bottom circles of cylinder
            line_segments.push(([x1, cylinder_height, z1], [x2, cylinder_height, z2]));
            line_segments.push(([x1, -cylinder_height, z1], [x2, -cylinder_height, z2]));
        }
        
        // Hemispherical caps
        for ring in 1..hemisphere_rings {
            let phi = std::f32::consts::PI * ring as f32 / (hemisphere_rings * 2) as f32; // 0 to π/2
            let ring_radius = radius * phi.sin();
            let ring_y = radius * phi.cos();
            
            for i in 0..segments {
                let angle = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
                let next_angle = 2.0 * std::f32::consts::PI * ((i + 1) % segments) as f32 / segments as f32;
                
                let (x1, z1) = (angle.cos() * ring_radius, angle.sin() * ring_radius);
                let (x2, z2) = (next_angle.cos() * ring_radius, next_angle.sin() * ring_radius);
                
                // Top hemisphere rings
                line_segments.push(([x1, cylinder_height + ring_y, z1], [x2, cylinder_height + ring_y, z2]));
                // Bottom hemisphere rings  
                line_segments.push(([x1, -cylinder_height - ring_y, z1], [x2, -cylinder_height - ring_y, z2]));
            }
        }
        
        // Meridian lines for hemisphere structure
        for i in 0..segments/2 { // Only need half the meridians to avoid clutter
            let angle = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
            let (x, z) = (angle.cos() * radius, angle.sin() * radius);
            
            // Top hemisphere meridian
            line_segments.push(([x, cylinder_height, z], [0.0, cylinder_height + radius, 0.0]));
            // Bottom hemisphere meridian  
            line_segments.push(([x, -cylinder_height, z], [0.0, -cylinder_height - radius, 0.0]));
        }
        
        PrimitiveLineData::new(line_segments)
    }
    
    fn generate_plane_data() -> PrimitiveLineData {
        // Simple grid plane in XZ plane
        let size = 0.8;
        let divisions = 4;
        let step = size / divisions as f32;
        let half = size / 2.0;
        
        let mut line_segments = Vec::new();
        
        // Horizontal lines (along X axis)
        for i in 0..=divisions {
            let z = -half + i as f32 * step;
            line_segments.push(([-half, 0.0, z], [half, 0.0, z]));
        }
        
        // Vertical lines (along Z axis)
        for i in 0..=divisions {
            let x = -half + i as f32 * step;
            line_segments.push(([x, 0.0, -half], [x, 0.0, half]));
        }
        
        PrimitiveLineData::new(line_segments)
    }
    
    fn generate_hemisphere_data() -> PrimitiveLineData {
        // Hemisphere: upper half of a sphere with base circle
        let mut line_segments = Vec::with_capacity(SPHERE_RINGS * SPHERE_SEGMENTS);
        
        // Generate only the upper half of the sphere (latitude rings)
        for ring in 0..=(SPHERE_RINGS/2) {
            let theta = std::f32::consts::PI * (ring as f32) / (SPHERE_RINGS as f32);
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();
            let y = cos_theta * 0.5;
            
            for segment in 0..SPHERE_SEGMENTS {
                let phi = 2.0 * std::f32::consts::PI * (segment as f32) / (SPHERE_SEGMENTS as f32);
                let next_phi = 2.0 * std::f32::consts::PI * ((segment + 1) % SPHERE_SEGMENTS) as f32 / (SPHERE_SEGMENTS as f32);
                
                let x1 = sin_theta * phi.cos() * 0.5;
                let z1 = sin_theta * phi.sin() * 0.5;
                let x2 = sin_theta * next_phi.cos() * 0.5;
                let z2 = sin_theta * next_phi.sin() * 0.5;
                
                // Horizontal ring (latitude lines)
                line_segments.push(([x1, y, z1], [x2, y, z2]));
                
                // Vertical lines (longitude lines) - only if not the top point
                if ring < SPHERE_RINGS/2 {
                    let next_theta = std::f32::consts::PI * ((ring + 1) as f32) / (SPHERE_RINGS as f32);
                    let next_sin_theta = next_theta.sin();
                    let next_cos_theta = next_theta.cos();
                    let next_y = next_cos_theta * 0.5;
                    let next_x = next_sin_theta * phi.cos() * 0.5;
                    let next_z = next_sin_theta * phi.sin() * 0.5;
                    
                    line_segments.push(([x1, y, z1], [next_x, next_y, next_z]));
                }
            }
        }
        
        // Base circle at y=0 (bottom edge of hemisphere)
        for segment in 0..SPHERE_SEGMENTS {
            let phi = 2.0 * std::f32::consts::PI * (segment as f32) / (SPHERE_SEGMENTS as f32);
            let next_phi = 2.0 * std::f32::consts::PI * ((segment + 1) % SPHERE_SEGMENTS) as f32 / (SPHERE_SEGMENTS as f32);
            
            let x1 = phi.cos() * 0.5;
            let z1 = phi.sin() * 0.5;
            let x2 = next_phi.cos() * 0.5;
            let z2 = next_phi.sin() * 0.5;
            
            // Base circle
            line_segments.push(([x1, 0.0, z1], [x2, 0.0, z2]));
        }
        
        PrimitiveLineData::new(line_segments)
    }
    
    // 2D Primitive generation functions
    
    fn generate_circle2d_data() -> PrimitiveLineData {
        // 2D circle outline in XY plane
        let segments = 16;
        let radius = 0.5;
        let mut line_segments = Vec::with_capacity(segments);
        
        for i in 0..segments {
            let angle = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
            let next_angle = 2.0 * std::f32::consts::PI * ((i + 1) % segments) as f32 / segments as f32;
            
            let x1 = angle.cos() * radius;
            let y1 = angle.sin() * radius;
            let x2 = next_angle.cos() * radius;
            let y2 = next_angle.sin() * radius;
            
            line_segments.push(([x1, y1, 0.0], [x2, y2, 0.0]));
        }
        
        PrimitiveLineData::new(line_segments)
    }
    
    fn generate_square2d_data() -> PrimitiveLineData {
        // 2D square outline in XY plane
        let half_size = 0.5;
        
        let line_segments = vec![
            // Square outline
            ([-half_size, -half_size, 0.0], [half_size, -half_size, 0.0]),  // bottom
            ([half_size, -half_size, 0.0], [half_size, half_size, 0.0]),    // right
            ([half_size, half_size, 0.0], [-half_size, half_size, 0.0]),    // top
            ([-half_size, half_size, 0.0], [-half_size, -half_size, 0.0]),  // left
        ];
        
        PrimitiveLineData::new(line_segments)
    }
    
    fn generate_triangle2d_data() -> PrimitiveLineData {
        // 2D equilateral triangle outline in XY plane
        let height = 0.866; // sqrt(3)/2 for equilateral triangle
        let half_base = 0.5;
        
        let vertices = [
            [0.0, height * 2.0 / 3.0, 0.0],        // top (centered)
            [-half_base, -height / 3.0, 0.0],      // bottom left
            [half_base, -height / 3.0, 0.0],       // bottom right
        ];
        
        let line_segments = vec![
            (vertices[0], vertices[1]),  // top to left
            (vertices[1], vertices[2]),  // left to right
            (vertices[2], vertices[0]),  // right to top
        ];
        
        PrimitiveLineData::new(line_segments)
    }

    fn generate_pentagon2d_data() -> PrimitiveLineData {
        // 2D regular pentagon outline in XY plane
        let radius = 0.5;
        let mut line_segments = Vec::new();
        let mut vertices = Vec::new();
        
        // Generate pentagon vertices
        for i in 0..5 {
            let angle = (i as f32) * 2.0 * std::f32::consts::PI / 5.0 - std::f32::consts::PI / 2.0;
            vertices.push([radius * angle.cos(), radius * angle.sin(), 0.0]);
        }
        
        // Connect vertices to form pentagon outline
        for i in 0..5 {
            let next = (i + 1) % 5;
            line_segments.push((vertices[i], vertices[next]));
        }
        
        PrimitiveLineData::new(line_segments)
    }

    fn generate_hexagon2d_data() -> PrimitiveLineData {
        // 2D regular hexagon outline in XY plane
        let radius = 0.5;
        let mut line_segments = Vec::new();
        let mut vertices = Vec::new();
        
        // Generate hexagon vertices
        for i in 0..6 {
            let angle = (i as f32) * 2.0 * std::f32::consts::PI / 6.0;
            vertices.push([radius * angle.cos(), radius * angle.sin(), 0.0]);
        }
        
        // Connect vertices to form hexagon outline
        for i in 0..6 {
            let next = (i + 1) % 6;
            line_segments.push((vertices[i], vertices[next]));
        }
        
        PrimitiveLineData::new(line_segments)
    }

    fn generate_diamond2d_data() -> PrimitiveLineData {
        // 2D diamond (rotated square) outline in XY plane
        let half_diagonal = 0.5;
        let vertices = [
            [0.0, half_diagonal, 0.0],      // top
            [half_diagonal, 0.0, 0.0],      // right
            [0.0, -half_diagonal, 0.0],     // bottom
            [-half_diagonal, 0.0, 0.0],     // left
        ];
        
        let line_segments = vec![
            (vertices[0], vertices[1]),  // top to right
            (vertices[1], vertices[2]),  // right to bottom
            (vertices[2], vertices[3]),  // bottom to left
            (vertices[3], vertices[0]),  // left to top
        ];
        
        PrimitiveLineData::new(line_segments)
    }

    fn generate_cross2d_data() -> PrimitiveLineData {
        // 2D cross outline in XY plane
        let arm_length = 0.4;
        let arm_thickness = 0.15;
        
        // Cross has 12 vertices forming the outline
        let vertices = [
            // Top arm
            [-arm_thickness, arm_length, 0.0],
            [arm_thickness, arm_length, 0.0],
            [arm_thickness, arm_thickness, 0.0],
            // Right arm
            [arm_length, arm_thickness, 0.0],
            [arm_length, -arm_thickness, 0.0],
            [arm_thickness, -arm_thickness, 0.0],
            // Bottom arm
            [arm_thickness, -arm_length, 0.0],
            [-arm_thickness, -arm_length, 0.0],
            [-arm_thickness, -arm_thickness, 0.0],
            // Left arm
            [-arm_length, -arm_thickness, 0.0],
            [-arm_length, arm_thickness, 0.0],
            [-arm_thickness, arm_thickness, 0.0],
        ];
        
        let mut line_segments = Vec::new();
        
        // Connect vertices to form cross outline
        for i in 0..12 {
            let next = (i + 1) % 12;
            line_segments.push((vertices[i], vertices[next]));
        }
        
        PrimitiveLineData::new(line_segments)
    }

    fn generate_star2d_data() -> PrimitiveLineData {
        // 2D 5-point star outline in XY plane
        let outer_radius = 0.5;
        let inner_radius = 0.2;
        let mut line_segments = Vec::new();
        let mut vertices = Vec::new();
        
        // Generate star vertices (10 total - 5 outer, 5 inner)
        for i in 0..10 {
            let angle = (i as f32) * std::f32::consts::PI / 5.0 - std::f32::consts::PI / 2.0;
            let radius = if i % 2 == 0 { outer_radius } else { inner_radius };
            vertices.push([radius * angle.cos(), radius * angle.sin(), 0.0]);
        }
        
        // Connect vertices to form star outline
        for i in 0..10 {
            let next = (i + 1) % 10;
            line_segments.push((vertices[i], vertices[next]));
        }
        
        PrimitiveLineData::new(line_segments)
    }

    fn generate_arrow2d_data() -> PrimitiveLineData {
        // 2D arrow outline in XY plane (pointing up)
        let shaft_width = 0.1;
        let head_width = 0.25;
        let head_height = 0.2;
        let _shaft_height = 0.3;
        
        let vertices = [
            // Arrow head
            [0.0, 0.5, 0.0],                        // tip
            [-head_width, 0.5 - head_height, 0.0], // left head
            [-shaft_width, 0.5 - head_height, 0.0], // left shaft top
            // Left shaft
            [-shaft_width, -0.5, 0.0],              // left shaft bottom
            [shaft_width, -0.5, 0.0],               // right shaft bottom
            // Right shaft
            [shaft_width, 0.5 - head_height, 0.0],  // right shaft top
            [head_width, 0.5 - head_height, 0.0],   // right head
        ];
        
        let mut line_segments = Vec::new();
        
        // Connect vertices to form arrow outline
        for i in 0..7 {
            let next = (i + 1) % 7;
            line_segments.push((vertices[i], vertices[next]));
        }
        
        PrimitiveLineData::new(line_segments)
    }
}

impl Default for PrimitiveCache {
    fn default() -> Self {
        Self::new()
    }
}