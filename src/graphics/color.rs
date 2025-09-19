//! Color representation and manipulation for graphics rendering
//!
//! This module provides RGBA color support with common color constants
//! and utility functions for graphics operations.

/// RGBA color representation
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// Create a new color with RGBA components (0.0 to 1.0)
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create a new color with RGB components and full opacity
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::new(r, g, b, 1.0)
    }

    /// Convert to 3-component array (RGB)
    pub fn to_array3(&self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }

    /// Convert to 4-component array (RGBA)
    pub fn to_array4(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn random_rgb_values() -> Color {
        let r = rand::random_range(0.0..1.0);
        let g = rand::random_range(0.0..1.0);
        let b = rand::random_range(0.0..1.0);
        let a = 1.0;

        Color { r, g, b, a }
    }

    pub fn random_color() -> Color {
        const COLORLIST: [Color; 17] = [
            Color::WHITE,
            Color::GRAY,
            Color::BLACK,
            Color::RED,
            Color::GREEN,
            Color::BLUE,
            Color::CYAN,
            Color::TEAL,
            Color::NAVY,
            Color::MAGENTA,
            Color::PURPLE,
            Color::MAROON,
            Color::PINK,
            Color::ORANGE,
            Color::YELLOW,
            Color::OLIVE,
            Color::FOREST,
        ];
        let index = rand::random_range(0..COLORLIST.len());
        COLORLIST[index]
    }

    // Individual color constants for convenience
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const GRAY: Color = Color {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        a: 1.0,
    };
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const RED: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const GREEN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const BLUE: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    pub const CYAN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const TEAL: Color = Color {
        r: 0.0,
        g: 0.5,
        b: 0.5,
        a: 1.0,
    };
    pub const NAVY: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.5,
        a: 1.0,
    };
    pub const MAGENTA: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    pub const PURPLE: Color = Color {
        r: 0.5,
        g: 0.0,
        b: 0.5,
        a: 1.0,
    };
    pub const MAROON: Color = Color {
        r: 0.5,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    pub const PINK: Color = Color {
        r: 1.0,
        g: 0.4,
        b: 0.7,
        a: 1.0,
    };
    pub const ORANGE: Color = Color {
        r: 1.0,
        g: 0.6,
        b: 0.0,
        a: 1.0,
    };
    pub const YELLOW: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const OLIVE: Color = Color {
        r: 0.5,
        g: 0.5,
        b: 0.0,
        a: 1.0,
    };
    pub const FOREST: Color = Color {
        r: 0.0,
        g: 0.5,
        b: 0.0,
        a: 1.0,
    };
}

impl From<[f32; 3]> for Color {
    fn from(arr: [f32; 3]) -> Self {
        Self::rgb(arr[0], arr[1], arr[2])
    }
}

impl From<[f32; 4]> for Color {
    fn from(arr: [f32; 4]) -> Self {
        Self::new(arr[0], arr[1], arr[2], arr[3])
    }
}
