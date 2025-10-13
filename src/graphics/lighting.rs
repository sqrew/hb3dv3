//! Dynamic lighting system for celestial bodies
//!
//! Supports positive and negative light sources that affect the skybox
//! and create atmospheric lighting effects.

use crate::engine::Vec3;
use crate::graphics::Color;

/// Maximum number of lights supported by the system
pub const MAX_LIGHTS: usize = 32;

/// A single light source in the scene
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Light {
    /// Position of the light in world space
    pub position: [f32; 3],
    /// Light intensity (positive = brightness, negative = darkness)
    pub intensity: f32,
    /// Light color (RGB)
    pub color: [f32; 3],
    /// Influence radius - how far the light affects the scene
    pub radius: f32,
}

impl Light {
    /// Create a new light source
    pub fn new(position: Vec3, intensity: f32, color: Color, radius: f32) -> Self {
        Self {
            position: [position.x, position.y, position.z],
            intensity,
            color: [color.r, color.g, color.b],
            radius,
        }
    }

    /// Create a positive light (brightness source)
    pub fn positive(position: Vec3, intensity: f32, color: Color, radius: f32) -> Self {
        Self::new(position, intensity.abs(), color, radius)
    }

    /// Create a negative light (darkness source)
    pub fn negative(position: Vec3, intensity: f32, color: Color, radius: f32) -> Self {
        Self::new(position, -intensity.abs(), color, radius)
    }
}

/// Lighting uniform data sent to GPU
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightingUniforms {
    /// Number of active lights (0-32)
    pub light_count: u32,
    /// Padding for alignment
    pub _padding1: u32,
    pub _padding2: u32,
    pub _padding3: u32,
    /// Array of lights (only first light_count are active)
    pub lights: [Light; MAX_LIGHTS],
}

impl Default for LightingUniforms {
    fn default() -> Self {
        Self {
            light_count: 0,
            _padding1: 0,
            _padding2: 0,
            _padding3: 0,
            lights: [Light {
                position: [0.0, 0.0, 0.0],
                intensity: 0.0,
                color: [0.0, 0.0, 0.0],
                radius: 0.0,
            }; MAX_LIGHTS],
        }
    }
}

impl LightingUniforms {
    /// Create empty lighting uniforms
    pub fn new() -> Self {
        Self::default()
    }

    /// Create from a collection of lights
    pub fn from_lights(lights: &[Light]) -> Self {
        let mut uniforms = Self::new();
        let count = lights.len().min(MAX_LIGHTS);
        uniforms.light_count = count as u32;
        uniforms.lights[..count].copy_from_slice(&lights[..count]);
        uniforms
    }

    /// Add a light to the uniforms (returns false if full)
    pub fn add_light(&mut self, light: Light) -> bool {
        if self.light_count >= MAX_LIGHTS as u32 {
            return false;
        }
        self.lights[self.light_count as usize] = light;
        self.light_count += 1;
        true
    }

    /// Clear all lights
    pub fn clear(&mut self) {
        self.light_count = 0;
    }

    /// Get slice of active lights
    pub fn active_lights(&self) -> &[Light] {
        &self.lights[..self.light_count as usize]
    }
}
