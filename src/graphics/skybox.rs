//! Fractal skybox rendering system
//!
//! Renders an animated fractal pattern on a large encompassing sphere
//! to create a mesmerizing background for the game world.

use crate::graphics::{Color, Primitive, PrimitiveType, Vec3};
use rand::Rng;

/// Configuration for fractal animation and parameters
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
    /// Julia set parameter - real part
    pub julia_c_real: f32,
    /// Julia set parameter - imaginary part
    pub julia_c_imag: f32,
    /// Second Julia set parameter - real part
    pub julia2_c_real: f32,
    /// Second Julia set parameter - imaginary part
    pub julia2_c_imag: f32,
    /// Animation speed multiplier
    pub animation_speed: f32,
    /// Fractal mixing weights (4 values for different fractal types)
    pub fractal_weights: [f32; 4],
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
            julia_c_real: -0.7,
            julia_c_imag: 0.27015,
            julia2_c_real: -0.4,
            julia2_c_imag: 0.6,
            animation_speed: 1.0,
            fractal_weights: [0.4, 0.3, 0.2, 0.1], // Mandelbrot, Julia1, Burning Ship, Julia2
        }
    }
}

impl FractalConfig {
    /// Generate a random fractal configuration for unique playthroughs
    pub fn random() -> Self {
        let mut rng = rand::rng();

        // Generate interesting Julia set parameters
        // These ranges are chosen to create visually appealing fractals
        let julia_c_real = rng.random_range(-1.5..1.5);
        let julia_c_imag = rng.random_range(-1.5..1.5);
        let julia2_c_real = rng.random_range(-1.0..1.0);
        let julia2_c_imag = rng.random_range(-1.0..1.0);

        // Random animation speed (0.5x to 2.0x normal speed)
        let animation_speed = rng.random_range(0.5..2.0);

        // Random fractal mixing weights (normalized to sum to 1.0)
        let w1: f32 = rng.random_range(0.1..0.6);
        let w2: f32 = rng.random_range(0.1..0.5);
        let w3: f32 = rng.random_range(0.1..0.4);
        let w4: f32 = rng.random_range(0.1..0.3);
        let total = w1 + w2 + w3 + w4;
        let fractal_weights = [w1/total, w2/total, w3/total, w4/total];

        // Random iteration count for different detail levels
        let max_iterations = rng.random_range(48..96);

        // Random palette selection
        let palette = rng.random_range(0..4);

        Self {
            max_iterations,
            zoom: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            time: 0.0,
            palette,
            julia_c_real,
            julia_c_imag,
            julia2_c_real,
            julia2_c_imag,
            animation_speed,
            fractal_weights,
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
            config: FractalConfig::random(), // Use random fractals for unique playthroughs!
        }
    }

    /// Create a skybox with a specific fractal configuration
    pub fn with_config(config: FractalConfig) -> Self {
        Self { config }
    }

    /// Create a skybox with default (non-random) fractals
    pub fn with_default_fractals() -> Self {
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
