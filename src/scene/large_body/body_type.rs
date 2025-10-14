use crate::graphics::{Color, PrimitiveType};

/// Death sequence state for large bodies
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeathState {
    Alive,
    DeathSequence { timer: f32 }, // Death sequence in progress, timer = time remaining
    ReadyForRemoval,              // Death sequence complete, ready to be removed
}

/// Types of large gravitational bodies in the game
#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LargeBodyType {
    /// Massive gravitational body with extreme pull
    BlackHole,
    BlackHoleLarge,
    /// Massive gravitational body with extreme repulsion (negative mass)
    WhiteHole,
    /// Large rocky body with moderate gravity
    NeutronStar,
    ExoticMatter,
    Star,
    /// Habitable world with Earth-like gravity
    GasGiant,
    Planet,
    /// Artificial structure with artificial gravity
    /// Gas giant with strong gravity and large radius
    /// Exotic matter that oscillates between attractive and repulsive gravity
    LauncherMass,
    Debug,
}

impl LargeBodyType {
    /// Get default mass for this body type (in kg, scaled for gameplay)
    pub fn default_mass(self) -> f32 {
        match self {
            LargeBodyType::BlackHole => 10_000_000.0, // Extreme mass
            LargeBodyType::BlackHoleLarge => 100_000_000.0,
            LargeBodyType::WhiteHole => -9000_000.0, // Slightly less negative mass for stability
            LargeBodyType::NeutronStar => 5000_000.0, // Very high mass
            LargeBodyType::ExoticMatter => 5000_000.0, // High mass for strong oscillating effects
            LargeBodyType::Star => 3000_000.0,       // Very high mass for strong gravity
            LargeBodyType::GasGiant => 1500_000.0,   // Large mass
            LargeBodyType::Planet => 500_000.0,      // Medium mass
            LargeBodyType::LauncherMass => 49_000.0,
            LargeBodyType::Debug => 100.0, // Debug body with small but reasonable mass
        }
    }

    /// Get default radius for this body type (for rendering and collision)
    pub fn default_radius(self) -> f32 {
        match self {
            LargeBodyType::BlackHole => 1.0, // Small but visible
            LargeBodyType::BlackHoleLarge => 50.0,
            LargeBodyType::WhiteHole => 1.0, // Same size as black hole, but opposite effect
            LargeBodyType::NeutronStar => 3.0, // Very small but dense
            LargeBodyType::ExoticMatter => 15.0, // Large and visible for its effects
            LargeBodyType::Star => 100.0,    // Large and bright for visibility
            LargeBodyType::GasGiant => 20.0, // Very large
            LargeBodyType::Planet => 10.0,   // Medium size
            LargeBodyType::LauncherMass => 3.0,
            LargeBodyType::Debug => 1.0, // Debug body with small but reasonable radius
        }
    }

    /// Get the color for rendering this body type
    pub fn color(self) -> Color {
        match self {
            LargeBodyType::BlackHole => Color::MAGENTA,
            LargeBodyType::BlackHoleLarge => Color::MAGENTA,
            LargeBodyType::WhiteHole => Color::WHITE,
            LargeBodyType::NeutronStar => Color::GREEN,
            LargeBodyType::ExoticMatter => Color::PINK,
            LargeBodyType::Star => Color::RED,
            LargeBodyType::GasGiant => Color::YELLOW,
            LargeBodyType::Planet => Color::NAVY,
            LargeBodyType::LauncherMass => Color::WHITE,
            LargeBodyType::Debug => Color::random_color(),
        }
    }

    /// Get default collision radius ratio for this body type (multiplier of visual radius)
    pub fn default_collision_radius_ratio(self) -> f32 {
        match self {
            LargeBodyType::BlackHole => 1.0,
            LargeBodyType::BlackHoleLarge => 1.0,
            LargeBodyType::WhiteHole => 1.0,
            LargeBodyType::NeutronStar => 1.0,
            LargeBodyType::ExoticMatter => 1.0, // Large collision area for oscillating effects
            LargeBodyType::Star => 1.0,
            LargeBodyType::GasGiant => 1.0,
            LargeBodyType::Planet => 1.0,
            LargeBodyType::LauncherMass => 1.0,
            LargeBodyType::Debug => 1.0, // Standard collision radius ratio
        }
    }

    /// Get default angular velocity for this body type (radians per second)
    pub fn default_angular_velocity(self) -> f32 {
        match self {
            LargeBodyType::BlackHole => 10.0, // Fast spinning black hole for frame-dragging
            LargeBodyType::BlackHoleLarge => 1.0, // Spinning black hole for frame-dragging
            LargeBodyType::WhiteHole => -3.0, // Counter-rotating white hole
            LargeBodyType::NeutronStar => 12.0, // Extremely fast pulsar rotation
            LargeBodyType::ExoticMatter => 6.0, // Rapid oscillating rotation for visual effect
            LargeBodyType::Star => 0.5,       // Moderate stellar rotation
            LargeBodyType::GasGiant => 1.0,   // Fast rotation like Jupiter
            LargeBodyType::Planet => 0.3,     // Earth-like rotation (slower)
            LargeBodyType::LauncherMass => 3.0,
            LargeBodyType::Debug => 0.5, // Debug body with moderate rotation
        }
    }

    /// Get default ergosphere radius ratio (multiplied by visual radius)
    pub fn default_ergosphere_radius_ratio(self) -> f32 {
        match self {
            LargeBodyType::BlackHole => 30.0, // Much larger ergosphere for visible frame-dragging
            LargeBodyType::BlackHoleLarge => 2.0, // Reduced to match playable area
            LargeBodyType::NeutronStar => 20.0, // Large intense ergosphere
            LargeBodyType::WhiteHole => 20.0, // Significant ergosphere effect
            LargeBodyType::ExoticMatter => 20.0, //
            LargeBodyType::LauncherMass => 20.0,
            _ => 0.0,
        }
    }

    /// Get default frame-dragging strength (based on mass and angular velocity)
    pub fn default_frame_dragging_strength(self) -> f32 {
        let mass = self.default_mass();
        let angular_vel = self.default_angular_velocity().abs(); // Use absolute value
        let strength_factor = match self {
            LargeBodyType::BlackHole => 0.2,      // Strong frame-dragging
            LargeBodyType::BlackHoleLarge => 0.5, // Strong frame-dragging
            LargeBodyType::NeutronStar => 0.25,   // Very strong (dense + fast spinning)
            LargeBodyType::WhiteHole => 0.15,     // Moderate frame-dragging
            LargeBodyType::ExoticMatter => 0.8,   //
            LargeBodyType::LauncherMass => 5.0,
            _ => 0.0, // No frame-dragging for other types
        };
        mass * angular_vel * strength_factor
    }

    /// Get the primitive type for rendering
    pub fn primitive_type(self) -> PrimitiveType {
        match self {
            LargeBodyType::BlackHole => PrimitiveType::Sphere,
            LargeBodyType::BlackHoleLarge => PrimitiveType::Sphere,
            LargeBodyType::WhiteHole => PrimitiveType::Sphere,
            LargeBodyType::NeutronStar => PrimitiveType::Sphere,
            LargeBodyType::Star => PrimitiveType::Sphere,
            LargeBodyType::GasGiant => PrimitiveType::Sphere,
            LargeBodyType::Planet => PrimitiveType::Sphere,
            LargeBodyType::ExoticMatter => PrimitiveType::Sphere,
            LargeBodyType::LauncherMass => PrimitiveType::Icosahedron,
            LargeBodyType::Debug => PrimitiveType::Sphere,
        }
    }

    /// Get atmosphere radius multiplier (None = no atmosphere)
    pub fn atmosphere_radius_multiplier(self) -> Option<f32> {
        match self {
            LargeBodyType::BlackHole => Some(2.0), // Accretion disk glow
            LargeBodyType::BlackHoleLarge => Some(1.5), // Subtle glow
            LargeBodyType::Star => Some(3.5),      // Corona effect
            LargeBodyType::GasGiant => Some(2.0),  // Thick atmosphere
            LargeBodyType::Planet => Some(1.5),    // Thin atmosphere
            LargeBodyType::NeutronStar => Some(2.0), // Radiation glow
            LargeBodyType::ExoticMatter => Some(2.0), // Energy field
            _ => None, // WhiteHole, LauncherMass, Debug don't have atmospheres
        }
    }

    /// Get atmosphere color with transparency
    pub fn atmosphere_color(self) -> Color {
        match self {
            LargeBodyType::BlackHole => Color::PURPLE,
            LargeBodyType::BlackHoleLarge => Color::PURPLE,
            LargeBodyType::Star => Color::ORANGE,
            LargeBodyType::GasGiant => Color::YELLOW,
            LargeBodyType::Planet => Color::CYAN,
            LargeBodyType::NeutronStar => Color::GREEN,
            LargeBodyType::ExoticMatter => Color::PINK,
            _ => Color::new(1.0, 1.0, 1.0, 0.1), // Fallback
        }
    }
}
