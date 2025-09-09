//! Graphics rendering systems
//! 
//! This module handles all graphics-related operations including
//! rendering pipelines, camera management, and primitive generation.

// Core modules
pub mod types;
pub mod primitive_type;
pub mod render_data;

// Rendering systems
pub mod renderer;
pub mod camera;
pub mod vertex;
pub mod primitives;
pub mod line_renderer;
pub mod bloom;
pub mod frustum_culling;
pub mod color;
pub mod constants;
pub mod primitive_cache;
pub mod line_batch;
pub mod collision_compute;
pub mod particles;

// Re-export main types
pub use renderer::GraphicsEngine;
pub use camera::{ThirdPersonCamera, CameraUniform, Projection};
pub use vertex::Vertex;
pub use line_renderer::InstancedLineRenderer;
pub use bloom::BloomRenderer;
pub use frustum_culling::{Frustum, is_visible_sphere};
pub use color::Color;
pub use primitive_type::PrimitiveType;
pub use types::{Vec3, Transform};
pub use render_data::Primitive;
pub use collision_compute::{CollisionCompute, CollisionPair};
pub use particles::ParticleSystem;