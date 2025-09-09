/// Types of geometric primitives that can be rendered
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    Cube,
    Sphere,
    Cylinder,
    Pyramid,
    Tetrahedron,
    Cone,
    Torus,
    Ellipsoid,
    Octahedron,
    Icosahedron,
    Dodecahedron,
    Capsule,
    Plane,
    Hemisphere,
    // 2D Primitives (flat shapes in 3D space)
    Circle2D,
    Square2D,
    Triangle2D,
    Pentagon2D,
    Hexagon2D,
    Diamond2D,
    Cross2D,
    Star2D,
    Arrow2D,
}