//! Graphics rendering systems
//!
//! This module handles all graphics-related operations including
//! rendering pipelines, camera management, and primitive generation.

// Core modules
pub mod primitive_type;
pub mod render_data;
pub mod types;

// Rendering systems
pub mod bloom;
pub mod camera;
pub mod collision_compute;
pub mod color;
pub mod constants;
pub mod frustum_culling;
pub mod line_batch;
pub mod line_renderer;
pub mod lightning;
pub mod particles;
pub mod primitive_cache;
pub mod primitives;
pub mod renderer;
pub mod shape_particles;
pub mod skybox;
pub mod vertex;

// Re-export main types
pub use bloom::BloomRenderer;
pub use camera::{CameraUniform, Projection, ThirdPersonCamera};
pub use collision_compute::{CollisionCompute, CollisionPair};
pub use color::Color;
pub use frustum_culling::{Frustum, is_visible_sphere};
pub use line_renderer::InstancedLineRenderer;
pub use lightning::{LightningConfig, LightningEffectManager};
pub use particles::ParticleSystem;
pub use primitive_type::PrimitiveType;
pub use render_data::Primitive;
pub use renderer::GraphicsEngine;
pub use shape_particles::ShapeParticleSystem;
pub use skybox::SkyboxRenderer;
pub use types::{Transform, Vec3};
pub use vertex::Vertex;
