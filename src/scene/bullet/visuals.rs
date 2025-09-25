/// Visual style for rendering bullets
#[derive(Debug, Clone)]
pub struct BulletVisuals {
    pub primitive_type: crate::graphics::PrimitiveType,
    pub color: crate::graphics::Color,
    pub scale: f32,
}

impl BulletVisuals {
    pub fn basic_blaster() -> Self {
        Self {
            primitive_type: crate::graphics::PrimitiveType::Tetrahedron,
            color: crate::graphics::Color::YELLOW,
            scale: 1.0,
        }
    }

    pub fn chain_lightning() -> Self {
        Self {
            primitive_type: crate::graphics::PrimitiveType::Sphere,
            color: crate::graphics::Color::new(0.3, 0.8, 1.0, 1.0), // Electric blue
            scale: 0.8,
        }
    }

    pub fn seeking_explosive() -> Self {
        Self {
            primitive_type: crate::graphics::PrimitiveType::Octahedron,
            color: crate::graphics::Color::ORANGE,
            scale: 1.2,
        }
    }

    pub fn rapid_fire() -> Self {
        Self {
            primitive_type: crate::graphics::PrimitiveType::Tetrahedron,
            color: crate::graphics::Color::GREEN,
            scale: 0.6,
        }
    }

    pub fn shotgun() -> Self {
        Self {
            primitive_type: crate::graphics::PrimitiveType::Sphere,
            color: crate::graphics::Color::YELLOW,
            scale: 0.4,
        }
    }

    pub fn anti_gravity() -> Self {
        Self {
            primitive_type: crate::graphics::PrimitiveType::Octahedron,
            color: crate::graphics::Color::MAGENTA,
            scale: 1.1,
        }
    }

    pub fn fractal_cannon() -> Self {
        Self {
            primitive_type: crate::graphics::PrimitiveType::Tetrahedron,
            color: crate::graphics::Color::YELLOW,
            scale: 2.0, // start large, each successive generation becomes smaller so the huge swarm will end up composed of mostly regular sized bullets
        }
    }

    pub fn laser_cannon() -> Self {
        Self {
            primitive_type: crate::graphics::PrimitiveType::Tetrahedron,
            color: crate::graphics::Color::RED, // Bright red laser
            scale: 1.0,                         // Small projectile (trail is the main visual)
        }
    }
}
