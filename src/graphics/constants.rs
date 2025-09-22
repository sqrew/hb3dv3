//! Graphics rendering constants
//!
//! This module contains constants used throughout the graphics system
//! for primitive generation, rendering parameters, and visual effects.

// Primitive bounding radii (for frustum culling) - calculated from actual geometry
pub const CUBE_BOUNDING_RADIUS: f32 = 0.866; // sqrt(3)/2 for cube diagonal ±0.5
pub const SPHERE_BOUNDING_RADIUS: f32 = 0.5; // Unit sphere radius 0.5
pub const PYRAMID_BOUNDING_RADIUS: f32 = 0.866; // sqrt(0.5² + 0.5² + 1.0²) for base corner to apex
pub const TETRAHEDRON_BOUNDING_RADIUS: f32 = 0.612; // Distance from center to vertex (0.5 height, 0.433 base)
pub const CONE_BOUNDING_RADIUS: f32 = 0.707; // sqrt(0.5² + 0.5²) base radius to apex
pub const CYLINDER_BOUNDING_RADIUS: f32 = 0.707; // sqrt(height²/4 + radius²) = sqrt(0.5² + 0.5²)
pub const OCTAHEDRON_BOUNDING_RADIUS: f32 = 0.5; // Distance from center to vertex ±0.5
pub const TORUS_BOUNDING_RADIUS: f32 = 0.55; // Major radius (0.4) + minor radius (0.15)
pub const ELLIPSOID_BOUNDING_RADIUS: f32 = 0.5; // Uses sphere geometry, scaled by transform
pub const ICOSAHEDRON_BOUNDING_RADIUS: f32 = 0.809; // Golden ratio scale: 0.5 * φ ≈ 0.809
pub const DODECAHEDRON_BOUNDING_RADIUS: f32 = 0.485; // scale * φ = 0.3 * 1.618 ≈ 0.485
pub const CAPSULE_BOUNDING_RADIUS: f32 = 0.516; // sqrt((height/2 + radius)² + radius²)
pub const PLANE_BOUNDING_RADIUS: f32 = 0.566; // sqrt(0.4² + 0.4²) diagonal of 0.8 square
pub const HEMISPHERE_BOUNDING_RADIUS: f32 = 0.5; // Same as sphere, just half

// 2D Primitive bounding radii (flat shapes in 3D space)
pub const CIRCLE2D_BOUNDING_RADIUS: f32 = 0.5; // Circle radius
pub const SQUARE2D_BOUNDING_RADIUS: f32 = 0.707; // sqrt(2)/2 for diagonal
pub const TRIANGLE2D_BOUNDING_RADIUS: f32 = 0.577; // Equilateral triangle circumradius

// Primitive generation parameters (optimized for performance)
pub const SPHERE_RINGS: usize = 6; // Number of horizontal rings (latitude lines) - reduced from 8
pub const SPHERE_SEGMENTS: usize = 12; // Number of segments per ring (longitude lines) - reduced from 16
pub const CYLINDER_SEGMENTS: usize = 8; // Number of segments around cylinder - reduced from 16 (50% reduction)
pub const CONE_SEGMENTS: usize = 8; // Number of segments around cone base - reduced from 16 (50% reduction)
pub const TORUS_MAJOR_SEGMENTS: usize = 12; // Segments around the main ring - reduced from 16 (25% reduction)
pub const TORUS_MINOR_SEGMENTS: usize = 6; // Segments around the tube cross-section - reduced from 8 (25% reduction)
pub const LINE_CYLINDER_SEGMENTS: u32 = 8; // Segments for instanced line rendering

// Line rendering parameters
pub const DEFAULT_LINE_THICKNESS: f32 = 0.10; // Default line thickness for wireframes
pub const ARENA_LINE_THICKNESS: f32 = 2.0; // Thicker lines for arena boundaries

// Torus geometry constants
pub const TORUS_MAJOR_RADIUS_RATIO: f32 = 0.8;
pub const TORUS_MINOR_RADIUS_RATIO: f32 = 0.3;

// Golden ratio for icosahedron generation
pub const GOLDEN_RATIO: f32 = 1.618033988749; // (1 + sqrt(5)) / 2

// Additional 2D Primitive Bounding Radii
pub const PENTAGON2D_BOUNDING_RADIUS: f32 = 0.5; // Regular pentagon inscribed in circle
pub const HEXAGON2D_BOUNDING_RADIUS: f32 = 0.5; // Regular hexagon inscribed in circle
pub const DIAMOND2D_BOUNDING_RADIUS: f32 = 0.707; // Diamond (rotated square)
pub const CROSS2D_BOUNDING_RADIUS: f32 = 0.707; // Cross shape
pub const STAR2D_BOUNDING_RADIUS: f32 = 0.707; // 5-point star
pub const ARROW2D_BOUNDING_RADIUS: f32 = 0.6; // Arrow shape

// Tetrahedron vertex positions (for proper regular tetrahedron)
pub const TETRAHEDRON_HEIGHT: f32 = 0.5;
pub const TETRAHEDRON_BASE_Y: f32 = -0.25;
pub const TETRAHEDRON_BASE_FRONT: f32 = 0.25;
pub const TETRAHEDRON_BASE_BACK: f32 = -0.5;
pub const TETRAHEDRON_BASE_X_OFFSET: f32 = 0.433; // sqrt(3)/4 for equilateral triangle
