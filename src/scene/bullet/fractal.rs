use crate::engine::{EntityId, Vec3};
use crate::scene::bullet::BulletVisuals;
use rand::Rng;

/// Different fractal splitting patterns
#[derive(Debug, Clone, Copy)]
pub enum FractalPattern {
    // 2D Patterns (XZ plane mostly)
    BinaryTree,         // 2-way splits at fixed angles
    SierpinskiTriangle, // 3-way splits forming triangles
    KochSnowflake,      // 4-way splits with geometric progression
    DragonCurve,        // Alternating L/R splits
    FibonacciSpiral,    // Golden ratio angle splits

    // True 3D Patterns (all three axes)
    Tetrahedron3D,     // 4-way tetrahedral splits in 3D space
    Octahedron3D,      // 6-way octahedral splits (up/down/4 cardinal directions)
    Cube3D,            // 8-way cubic splits to all cube corners
    Icosahedron3D,     // 12-way icosahedral splits (most complex!)
    HelixSpiral3D,     // Spiral around Y-axis with 3D branching
    SphereExplosion3D, // Radial explosion in all directions
}

impl FractalPattern {
    /// Randomly select a fractal pattern
    pub fn random() -> Self {
        let mut rng = rand::rng();
        match rng.random_range(0..11) {
            0 => FractalPattern::BinaryTree,
            1 => FractalPattern::SierpinskiTriangle,
            2 => FractalPattern::KochSnowflake,
            3 => FractalPattern::DragonCurve,
            4 => FractalPattern::FibonacciSpiral,
            5 => FractalPattern::Tetrahedron3D,
            6 => FractalPattern::Octahedron3D,
            7 => FractalPattern::Cube3D,
            8 => FractalPattern::Icosahedron3D,
            9 => FractalPattern::HelixSpiral3D,
            _ => FractalPattern::SphereExplosion3D,
        }
    }

    /// Get the number of children this pattern creates per split
    pub fn child_count(&self) -> usize {
        match self {
            // 2D Patterns
            FractalPattern::BinaryTree => 2,
            FractalPattern::SierpinskiTriangle => 3,
            FractalPattern::KochSnowflake => 4,
            FractalPattern::DragonCurve => 2,
            FractalPattern::FibonacciSpiral => 2,

            // 3D Patterns
            FractalPattern::Tetrahedron3D => 4,
            FractalPattern::Octahedron3D => 6,
            FractalPattern::Cube3D => 8,
            FractalPattern::Icosahedron3D => 12,
            FractalPattern::HelixSpiral3D => 3,
            FractalPattern::SphereExplosion3D => 10,
        }
    }

    /// Calculate split directions for children relative to parent direction
    /// Returns a vector of 3D direction offsets for true 3D fractal patterns
    pub fn get_split_directions(&self, parent_direction: Vec3, generation: usize) -> Vec<Vec3> {
        let mut rng = rand::rng();
        let randomness = 0.1; // 10% randomization for organic feel

        match self {
            // 2D Patterns (converted to work with 3D directions)
            FractalPattern::BinaryTree => {
                let base_angle = 30.0_f32.to_radians();
                let spread = base_angle * (1.0 + generation as f32 * 0.2);
                let left_angle = -spread + rng.random_range(-randomness..randomness) * spread;
                let right_angle = spread + rng.random_range(-randomness..randomness) * spread;

                // Rotate parent direction in XZ plane
                vec![
                    Self::rotate_vector_xz(parent_direction, left_angle),
                    Self::rotate_vector_xz(parent_direction, right_angle),
                ]
            }
            FractalPattern::SierpinskiTriangle => {
                let angles = vec![-120.0_f32.to_radians(), 0.0, 120.0_f32.to_radians()];
                angles
                    .into_iter()
                    .map(|angle| {
                        let rand_offset = rng.random_range(-randomness..randomness) * 0.3;
                        Self::rotate_vector_xz(parent_direction, angle + rand_offset)
                    })
                    .collect()
            }
            FractalPattern::KochSnowflake => {
                let base_spread = 45.0_f32.to_radians();
                vec![
                    -base_spread,
                    -base_spread / 2.0,
                    base_spread / 2.0,
                    base_spread,
                ]
                .into_iter()
                .map(|angle| {
                    let rand_offset = rng.random_range(-randomness..randomness) * base_spread;
                    Self::rotate_vector_xz(parent_direction, angle + rand_offset)
                })
                .collect()
            }
            FractalPattern::DragonCurve => {
                let angle = 90.0_f32.to_radians();
                let side = if generation % 2 == 0 { -1.0 } else { 1.0 };
                let rand_offset = rng.random_range(-randomness..randomness) * 0.5;
                vec![
                    Self::rotate_vector_xz(parent_direction, side * angle + rand_offset),
                    Self::rotate_vector_xz(parent_direction, -side * angle + rand_offset),
                ]
            }
            FractalPattern::FibonacciSpiral => {
                let golden_angle = 137.5_f32.to_radians();
                let rand_offset = rng.random_range(-randomness..randomness) * 0.3;
                vec![
                    Self::rotate_vector_xz(parent_direction, golden_angle + rand_offset),
                    Self::rotate_vector_xz(parent_direction, -golden_angle + rand_offset),
                ]
            }

            // TRUE 3D PATTERNS - THIS IS WHERE IT GETS AMAZING!
            FractalPattern::Tetrahedron3D => {
                // Tetrahedral geometry - 4 directions forming a perfect tetrahedron
                let _tet_angle = 109.47_f32.to_radians(); // Tetrahedral angle
                vec![
                    Vec3::new(1.0, 1.0, 1.0).normalize(),
                    Vec3::new(-1.0, -1.0, 1.0).normalize(),
                    Vec3::new(-1.0, 1.0, -1.0).normalize(),
                    Vec3::new(1.0, -1.0, -1.0).normalize(),
                ]
                .into_iter()
                .map(|dir| {
                    // Add some randomness and scale relative to parent direction
                    let rand_vec = Vec3::new(
                        rng.random_range(-randomness..randomness),
                        rng.random_range(-randomness..randomness),
                        rng.random_range(-randomness..randomness),
                    );
                    (dir + rand_vec).normalize()
                })
                .collect()
            }

            FractalPattern::Octahedron3D => {
                // 6 directions: up, down, and 4 cardinal directions
                vec![
                    Vec3::new(0.0, 1.0, 0.0),  // Up
                    Vec3::new(0.0, -1.0, 0.0), // Down
                    Vec3::new(1.0, 0.0, 0.0),  // Right
                    Vec3::new(-1.0, 0.0, 0.0), // Left
                    Vec3::new(0.0, 0.0, 1.0),  // Forward
                    Vec3::new(0.0, 0.0, -1.0), // Back
                ]
                .into_iter()
                .map(|dir| {
                    let rand_vec = Vec3::new(
                        rng.random_range(-randomness..randomness),
                        rng.random_range(-randomness..randomness),
                        rng.random_range(-randomness..randomness),
                    );
                    (dir + rand_vec).normalize()
                })
                .collect()
            }

            FractalPattern::Cube3D => {
                // 8 directions to cube corners - MAXIMUM 3D SPREAD!
                vec![
                    Vec3::new(1.0, 1.0, 1.0),    // +++
                    Vec3::new(-1.0, 1.0, 1.0),   // -++
                    Vec3::new(1.0, -1.0, 1.0),   // +-+
                    Vec3::new(1.0, 1.0, -1.0),   // ++-
                    Vec3::new(-1.0, -1.0, 1.0),  // --+
                    Vec3::new(-1.0, 1.0, -1.0),  // -+-
                    Vec3::new(1.0, -1.0, -1.0),  // +--
                    Vec3::new(-1.0, -1.0, -1.0), // ---
                ]
                .into_iter()
                .map(|dir| {
                    let rand_vec = Vec3::new(
                        rng.random_range(-randomness..randomness),
                        rng.random_range(-randomness..randomness),
                        rng.random_range(-randomness..randomness),
                    );
                    (dir.normalize() + rand_vec).normalize()
                })
                .collect()
            }

            FractalPattern::Icosahedron3D => {
                // 12 directions based on icosahedral geometry - THE ULTIMATE PATTERN!
                let phi = (1.0 + 5.0_f32.sqrt()) / 2.0; // Golden ratio
                vec![
                    Vec3::new(1.0, phi, 0.0),
                    Vec3::new(-1.0, phi, 0.0),
                    Vec3::new(1.0, -phi, 0.0),
                    Vec3::new(-1.0, -phi, 0.0),
                    Vec3::new(0.0, 1.0, phi),
                    Vec3::new(0.0, -1.0, phi),
                    Vec3::new(0.0, 1.0, -phi),
                    Vec3::new(0.0, -1.0, -phi),
                    Vec3::new(phi, 0.0, 1.0),
                    Vec3::new(-phi, 0.0, 1.0),
                    Vec3::new(phi, 0.0, -1.0),
                    Vec3::new(-phi, 0.0, -1.0),
                ]
                .into_iter()
                .map(|dir| {
                    let rand_vec = Vec3::new(
                        rng.random_range(-randomness..randomness),
                        rng.random_range(-randomness..randomness),
                        rng.random_range(-randomness..randomness),
                    );
                    (dir.normalize() + rand_vec).normalize()
                })
                .collect()
            }

            FractalPattern::HelixSpiral3D => {
                // 3 spiraling branches around the Y axis
                let y_component = 0.3; // Slight upward trend
                let radius_decay = 0.8_f32.powf(generation as f32); // Spiral tightens
                vec![0, 1, 2]
                    .into_iter()
                    .map(|i| {
                        let angle = (i as f32) * 120.0_f32.to_radians()
                            + generation as f32 * 30.0_f32.to_radians();
                        let rand_offset = rng.random_range(-randomness..randomness);
                        Vec3::new(
                            (angle + rand_offset).cos() * radius_decay,
                            y_component + rng.random_range(-randomness..randomness) * 0.2,
                            (angle + rand_offset).sin() * radius_decay,
                        )
                        .normalize()
                    })
                    .collect()
            }

            FractalPattern::SphereExplosion3D => {
                // Radial explosion in all directions - pure 3D chaos!
                (0..10)
                    .map(|_| {
                        Vec3::new(
                            rng.random_range(-1.0..1.0),
                            rng.random_range(-1.0..1.0),
                            rng.random_range(-1.0..1.0),
                        )
                        .normalize()
                    })
                    .collect()
            }
        }
    }

    /// Helper function to rotate a vector around the Y axis (XZ plane rotation)
    fn rotate_vector_xz(vector: Vec3, angle: f32) -> Vec3 {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        Vec3::new(
            vector.x * cos_a - vector.z * sin_a,
            vector.y, // Y stays the same for XZ rotation
            vector.x * sin_a + vector.z * cos_a,
        )
    }
}

/// Configuration for fractal behavior
#[derive(Debug, Clone)]
pub struct FractalConfig {
    pub pattern: FractalPattern,
    pub max_depth: usize,          // Maximum generations (2-4 recommended)
    pub split_delay: f32,          // Time before splitting (seconds)
    pub velocity_inheritance: f32, // How much speed children inherit (0.0-1.0)
    pub spread_factor: f32,        // How wide splits spread (multiplier)
    pub size_decay: f32,           // How much smaller each generation gets
}

impl FractalConfig {
    pub fn random() -> Self {
        let mut rng = rand::rng();
        let pattern = FractalPattern::random();

        // Performance-aware depth limits based on pattern complexity
        let max_depth = match pattern {
            // 2D patterns can go deeper (fewer splits per generation)
            FractalPattern::BinaryTree
            | FractalPattern::DragonCurve
            | FractalPattern::FibonacciSpiral => {
                rng.random_range(5..=8) // 2-way splits can handle deeper recursion
            }
            FractalPattern::SierpinskiTriangle | FractalPattern::HelixSpiral3D => {
                rng.random_range(5..=8) // 3-way splits need some limiting
            }
            FractalPattern::KochSnowflake | FractalPattern::Tetrahedron3D => {
                rng.random_range(4..=6) // 4-way splits - moderate depth
            }
            FractalPattern::Octahedron3D => {
                rng.random_range(3..=5) // 6-way splits - more conservative
            }
            FractalPattern::Cube3D | FractalPattern::SphereExplosion3D => {
                rng.random_range(3..=3) // 8+ way splits - very conservative for performance
            }
            FractalPattern::Icosahedron3D => {
                rng.random_range(3..=3) // 12-way splits - extremely conservative!
            }
        };

        FractalConfig {
            pattern,
            max_depth,
            split_delay: rng.random_range(0.2..0.6), // Faster splits for more action
            velocity_inheritance: rng.random_range(0.7..0.95), // Higher speed inheritance
            spread_factor: rng.random_range(0.8..1.5), // Spread variation
            size_decay: rng.random_range(0.75..0.9), // Less aggressive size decay for deep fractals
        }
    }

    pub fn binary_tree() -> Self {
        FractalConfig {
            pattern: FractalPattern::BinaryTree,
            max_depth: 8, // Deep binary trees look amazing
            split_delay: 0.3,
            velocity_inheritance: 0.85,
            spread_factor: 1.0,
            size_decay: 0.8,
        }
    }

    pub fn sierpinski() -> Self {
        FractalConfig {
            pattern: FractalPattern::SierpinskiTriangle,
            max_depth: 6, // Sierpinski gets dense quickly with 3-way splits
            split_delay: 0.25,
            velocity_inheritance: 0.8,
            spread_factor: 1.2,
            size_decay: 0.75,
        }
    }

    pub fn spectacular() -> Self {
        FractalConfig {
            pattern: FractalPattern::random(),
            max_depth: 10,     // Maximum spectacle!
            split_delay: 0.15, // Very fast splits
            velocity_inheritance: 0.9,
            spread_factor: 1.3,
            size_decay: 0.85, // Preserve size longer for deep fractals
        }
    }
}

/// Fractal bullet that can split into multiple children
#[derive(Debug, Clone)]
pub struct FractalBullet {
    pub entity_id: EntityId,
    pub position: Vec3,
    pub velocity: Vec3,
    pub lifetime: f32,
    pub damage: f32,
    pub mass: f32,
    pub visuals: BulletVisuals,

    // Fractal-specific properties
    pub config: FractalConfig,
    pub generation: usize,     // Current generation (0 = parent)
    pub time_until_split: f32, // Time remaining before splitting
    pub has_split: bool,       // Whether this bullet has already split
}

impl FractalBullet {
    pub fn new(
        entity_id: EntityId,
        position: Vec3,
        velocity: Vec3,
        lifetime: f32,
        damage: f32,
        mass: f32,
        config: FractalConfig,
        visuals: BulletVisuals,
    ) -> Self {
        FractalBullet {
            entity_id,
            position,
            velocity,
            lifetime,
            damage,
            mass,
            visuals,
            config: config.clone(),
            generation: 0,
            time_until_split: config.split_delay,
            has_split: false,
        }
    }

    /// Create a child bullet from this fractal bullet
    pub fn create_child(&self, entity_id: EntityId, split_direction: Vec3) -> FractalBullet {
        // Calculate new velocity using the 3D split direction
        let parent_speed = self.velocity.magnitude();
        let new_velocity =
            split_direction.normalize() * parent_speed * self.config.velocity_inheritance;

        // Child gets smaller visuals
        let mut child_visuals = self.visuals.clone();
        child_visuals.scale *= self.config.size_decay;

        FractalBullet {
            entity_id,
            position: self.position,
            velocity: new_velocity,
            lifetime: self.lifetime * 0.8, // Children live shorter
            damage: self.damage * 0.7,     // Reduced damage per generation
            mass: self.mass * 0.8,         // Lighter children
            visuals: child_visuals,
            config: self.config.clone(),
            generation: self.generation + 1,
            time_until_split: self.config.split_delay,
            has_split: false,
        }
    }

    /// Check if this bullet should split now
    pub fn should_split(&self, dt: f32) -> bool {
        if self.has_split || self.generation >= self.config.max_depth {
            return false;
        }

        self.time_until_split <= dt
    }

    /// Update the fractal bullet's split timer
    pub fn update_split_timer(&mut self, dt: f32) {
        if !self.has_split && self.generation < self.config.max_depth {
            self.time_until_split -= dt;
        }
    }

    /// Mark this bullet as having split
    pub fn mark_split(&mut self) {
        self.has_split = true;
    }

    /// Get the split directions for creating children
    pub fn get_split_directions(&self) -> Vec<Vec3> {
        self.config
            .pattern
            .get_split_directions(self.velocity.normalize(), self.generation)
    }
}

/// Event that represents a fractal split occurring
#[derive(Debug, Clone)]
pub struct FractalSplitEvent {
    pub parent_entity_id: EntityId,
    pub parent_position: Vec3,
    pub parent_velocity: Vec3,
    pub config: FractalConfig,
    pub generation: usize,
    pub visuals: BulletVisuals,
    pub damage: f32,
    pub mass: f32,
    pub lifetime: f32,
}
