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
}

impl SnakeData {
    pub fn new() -> Self {
        Self {
            segment_ids: Vec::new(),
            growth_timer: 0.0,
            growth_interval: 2.0, // Grow a new segment every 2 seconds
            max_segments: 30,
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

    pub fn config(&self) -> EnemyConfig {
        EnemyConfig {
            health: 500.0,
            speed: 80.0, // Fast and aggressive
            mass: 3000.0,
            collision_radius: 3.0,
            visual_scale: 3.0,
            primitive_type: PrimitiveType::Octahedron,
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
            health: 200.0, // Can take damage when routed from head
            speed: 0.0,    // Doesn't move on its own
            mass: 2000.0,
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
