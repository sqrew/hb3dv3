//! Fractal skybox rendering system
//!
//! Renders an animated fractal pattern on a large encompassing sphere
//! to create a mesmerizing background for the game world.

use crate::graphics::{Color, Primitive, PrimitiveType, Vec3};

/// Configuration for fractal animation
#[derive(Debug, Clone)]
pub struct FractalConfig {
    /// Maximum iterations for fractal calculation
    pub max_iterations: u32,
    /// Zoom level into the fractal
    pub zoom: f32,
    /// X offset in fractal space
    pub offset_x: f32,
    /// Y offset in fractal space
    pub offset_y: f32,
    /// Animation time for evolving parameters
    pub time: f32,
    /// Color palette selection (0-3 for different palettes)
    pub palette: u32,
}

impl Default for FractalConfig {
    fn default() -> Self {
        Self {
            max_iterations: 64,
            zoom: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            time: 0.0,
            palette: 0,
        }
    }
}

/// Skybox rendering system
pub struct SkyboxRenderer {
    config: FractalConfig,
}

impl SkyboxRenderer {
    pub fn new() -> Self {
        Self {
            config: FractalConfig::default(),
        }
    }

    /// Update fractal animation parameters
    pub fn update(&mut self, delta_time: f32) {
        self.config.time += delta_time;

        // Slowly animate fractal parameters for a mesmerizing effect
        self.config.offset_x = (self.config.time * 0.1).sin() * 0.5;
        self.config.offset_y = (self.config.time * 0.07).cos() * 0.3;
        self.config.zoom = 1.0 + (self.config.time * 0.05).sin() * 0.2;
    }

    /// Generate skybox primitive to be rendered by the main pipeline
    pub fn get_skybox_primitive(&self) -> Primitive {
        // Create a large sphere that encompasses the entire scene
        let position = Vec3::new(0.0, 0.0, 0.0); // Centered at origin
        let scale = 1000.0; // Large enough to encompass the game world

        // Use a color that changes over time for animation
        let t = self.config.time * 0.5;
        let color = Color {
            r: 0.5 + 0.3 * (t).sin(),
            g: 0.5 + 0.3 * (t + 2.0).sin(),
            b: 0.8 + 0.2 * (t + 4.0).sin(),
            a: 1.0,
        };

        Primitive::new(PrimitiveType::Sphere, position, color)
            .with_scale(Vec3::new(scale, scale, scale))
    }

    /// Get mutable reference to config for external modification
    pub fn config_mut(&mut self) -> &mut FractalConfig {
        &mut self.config
    }

    /// Get reference to config
    pub fn config(&self) -> &FractalConfig {
        &self.config
    }
}
