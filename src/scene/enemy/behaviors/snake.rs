use super::super::types::EnemyConfig;
use crate::engine::{Vec3, entity::EntityId};
use crate::graphics::{Color, PrimitiveType};

/// Snake head data - the vulnerable main body
#[derive(Debug, Clone, PartialEq)]
pub struct SnakeData {
    pub segment_ids: Vec<EntityId>,
    pub growth_timer: f32,
    pub growth_interval: f32,
    pub max_segments: usize,
    pub slither_phase: f32,     // Time-based phase for sinusoidal motion
    pub slither_amplitude: f32, // How far side-to-side the snake weaves
    pub slither_frequency: f32, // How fast the oscillation cycles
}

impl SnakeData {
    pub fn new() -> Self {
        Self {
            segment_ids: Vec::new(),
            growth_timer: 0.0,
            growth_interval: 2.0, // Grow a new segment every 2 seconds
            max_segments: 30,
            slither_phase: 0.0,
            slither_amplitude: 50.0, // Side-to-side distance in units
            slither_frequency: 0.2,  // Oscillations per second
        }
    }

    pub fn tick_growth(&mut self, dt: f32) -> bool {
        if self.segment_ids.len() >= self.max_segments {
            return false;
        }

        self.growth_timer += dt;
        if self.growth_timer >= self.growth_interval {
            self.growth_timer = 0.0;
            true // Signal to spawn new segment
        } else {
            false
        }
    }

    pub fn add_segment(&mut self, segment_id: EntityId) {
        self.segment_ids.push(segment_id);
    }

    pub fn remove_last_segment(&mut self) -> Option<EntityId> {
        self.segment_ids.pop()
    }

    pub fn segment_count(&self) -> usize {
        self.segment_ids.len()
    }

    pub fn tick_slither(&mut self, dt: f32) {
        self.slither_phase += dt * self.slither_frequency * 2.0 * std::f32::consts::PI;
        // Keep phase in reasonable range to avoid float precision issues
        if self.slither_phase > 100.0 * std::f32::consts::PI {
            self.slither_phase -= 100.0 * std::f32::consts::PI;
        }
    }

    /// Calculate slither offset vector perpendicular to movement direction
    pub fn calculate_slither_offset(&self, movement_direction: Vec3) -> Vec3 {
        let dir_magnitude = movement_direction.magnitude();

        // No slither if not moving
        if dir_magnitude < 0.01 {
            return Vec3::zeros();
        }

        let normalized_dir = movement_direction / dir_magnitude;

        // Create two perpendicular vectors for 3D slithering
        // First perpendicular: cross with world up (or forward if aligned with up)
        let world_up = Vec3::new(0.0, 1.0, 0.0);
        let perp1 = if normalized_dir.y.abs() > 0.99 {
            // Nearly vertical - use world forward instead
            normalized_dir.cross(&Vec3::new(0.0, 0.0, 1.0))
        } else {
            normalized_dir.cross(&world_up)
        };

        let perp1_normalized = if perp1.magnitude() > 0.01 {
            perp1.normalize()
        } else {
            Vec3::new(1.0, 0.0, 0.0) // Fallback
        };

        // Second perpendicular: cross movement direction with first perpendicular
        let perp2_normalized = normalized_dir.cross(&perp1_normalized).normalize();

        // Create 3D circular slithering motion
        let phase_sin = self.slither_phase.sin();
        let phase_cos = self.slither_phase.cos();

        // Blend both perpendicular directions for true 3D serpentine motion
        let offset = perp1_normalized * (phase_sin * self.slither_amplitude)
            + perp2_normalized * (phase_cos * self.slither_amplitude * 0.6);

        offset
    }

    pub fn config(&self) -> EnemyConfig {
        EnemyConfig {
            health: 500.0,
            speed: 160.0, // Fast and aggressive
            mass: 100.0,
            collision_radius: 3.0,
            visual_scale: 3.0,
            primitive_type: PrimitiveType::Tetrahedron,
            color: Color::GREEN,
        }
    }
}

/// Snake segment data - invulnerable body parts that follow the head
#[derive(Debug, Clone, PartialEq)]
pub struct SnakeSegmentData {
    pub head_id: EntityId,
    pub position_in_chain: usize, // 0 = directly behind head, 1 = second segment, etc.
    pub follow_distance: f32,
}

impl SnakeSegmentData {
    pub fn new(head_id: EntityId, position_in_chain: usize) -> Self {
        Self {
            head_id,
            position_in_chain,
            follow_distance: 5.0, // Distance to maintain behind previous segment
        }
    }

    pub fn config(&self) -> EnemyConfig {
        EnemyConfig {
            health: 50.0, // Can take damage when routed from head
            speed: 0.0,   // Doesn't move on its own
            mass: 100.0,
            collision_radius: 2.0,
            visual_scale: 2.0,
            primitive_type: PrimitiveType::Dodecahedron,
            color: Color::FOREST,
        }
    }
}

pub fn config() -> EnemyConfig {
    SnakeData::new().config()
}

/// Find the target for the snake head to follow
/// Prioritizes player unless there are closer enemies to chase
pub fn find_snake_target(
    _head_pos: Vec3,
    _all_enemies: &[super::super::entity::Enemy],
    _head_id: EntityId,
    player_pos: Vec3,
) -> Vec3 {
    // For now, just chase the player
    // Future: could add predatory behavior to chase other enemies
    player_pos
}

/// Calculate slithering target position for snake head
/// Applies perpendicular oscillation to the seek direction for serpentine motion
pub fn calculate_slither_target(
    snake_data: &mut SnakeData,
    head_pos: Vec3,
    base_target: Vec3,
    dt: f32,
) -> Vec3 {
    // Update slither phase
    snake_data.tick_slither(dt);

    // Calculate direction to base target
    let to_target = base_target - head_pos;

    // Calculate slither offset perpendicular to movement direction
    let slither_offset = snake_data.calculate_slither_offset(to_target);

    // Apply slither offset to target position
    base_target + slither_offset
}

/// Calculate positions for all segments following the chain
pub fn calculate_segment_positions(
    head_pos: Vec3,
    _head_vel: Vec3,
    segment_ids: &[EntityId],
    all_enemies: &[super::super::entity::Enemy],
) -> Vec<(EntityId, Vec3)> {
    let mut positions = Vec::new();

    if segment_ids.is_empty() {
        return positions;
    }

    // First segment follows head
    let follow_distance = 5.0;
    let mut prev_pos = head_pos;

    for &segment_id in segment_ids {
        // Find current segment position
        if let Some(segment) = all_enemies.iter().find(|e| e.entity_id() == segment_id) {
            let current_pos = segment.position();

            // Calculate direction from current position to previous position
            let to_prev = prev_pos - current_pos;
            let distance = to_prev.magnitude();

            // If too far, move closer; if too close, move away
            let target_pos = if distance > 0.1 {
                let direction = to_prev.normalize();
                prev_pos - direction * follow_distance
            } else {
                current_pos
            };

            positions.push((segment_id, target_pos));
            prev_pos = target_pos;
        }
    }

    positions
}
