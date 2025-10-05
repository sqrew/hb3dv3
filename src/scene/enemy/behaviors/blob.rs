use super::super::types::EnemyConfig;
use crate::engine::{Vec3, entity::EntityId};
use crate::graphics::{Color, PrimitiveType};

pub const BLOB_GRID_SPACING: f32 = 12.0;
pub const BLOB_MAX_SIZE: u32 = 5000;
pub const BLOB_MAX_FACTORY_COUNT: u32 = 100;
pub const BLOB_NODE_SPAWN_INTERVAL: f32 = 0.02;
pub const BLOB_FACTORY_SPAWN_INTERVAL: f32 = 1.0;
pub const BLOB_WITHER_RATE_MIN: f32 = 0.03; // 2% max HP per second
pub const BLOB_WITHER_RATE_MAX: f32 = 0.06; // 5% max HP per second

/// Blob node type classification
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlobNodeType {
    Core,
    Factory,
    Base,
}

/// Blob core data - central node that spawns initial blob
#[derive(Debug, Clone, PartialEq)]
pub struct BlobCoreData {
    pub node_ids: Vec<EntityId>,
    pub growth_timer: f32,
    pub growth_interval: f32,
    pub factory_spawn_timer: f32,
    pub factory_spawn_interval: f32,
    pub phase: BlobPhase,
    pub max_size: u32,
}

/// Blob growth phases
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlobPhase {
    Bolstering, // Phase 1: 0-273 nodes, omnidirectional
    Reaching,   // Phase 2: 273+ nodes, directional toward player
}

impl BlobCoreData {
    pub fn new() -> Self {
        Self {
            node_ids: Vec::new(),
            growth_timer: 0.0,
            growth_interval: BLOB_NODE_SPAWN_INTERVAL,
            factory_spawn_timer: 0.0,
            factory_spawn_interval: BLOB_FACTORY_SPAWN_INTERVAL,
            phase: BlobPhase::Bolstering,
            max_size: BLOB_MAX_SIZE,
        }
    }

    pub fn tick_growth(&mut self, dt: f32) -> bool {
        self.growth_timer += dt;
        if self.growth_timer >= self.growth_interval {
            self.growth_timer = 0.0;
            true
        } else {
            false
        }
    }

    pub fn tick_factory_spawn(&mut self, dt: f32) -> bool {
        self.factory_spawn_timer += dt;
        if self.factory_spawn_timer >= self.factory_spawn_interval {
            self.factory_spawn_timer = 0.0;
            true
        } else {
            false
        }
    }

    pub fn add_node(&mut self, node_id: EntityId) {
        self.node_ids.push(node_id);

        // Dynamic phase transitions based on node count
        self.update_phase();
    }

    pub fn remove_node(&mut self, node_id: EntityId) {
        self.node_ids.retain(|&id| id != node_id);

        // Re-evaluate phase after losing nodes
        self.update_phase();
    }

    /// Update phase based on current node count
    fn update_phase(&mut self) {
        let node_count = self.node_ids.len();

        if node_count >= 273 {
            self.phase = BlobPhase::Reaching;
        } else {
            self.phase = BlobPhase::Bolstering;
        }
    }

    pub fn node_count(&self) -> usize {
        self.node_ids.len()
    }

    pub fn is_at_max_capacity(&self) -> bool {
        self.node_ids.len() >= self.max_size as usize // 273 total including core
    }

    pub fn config(&self) -> EnemyConfig {
        EnemyConfig {
            health: 10000.0,
            speed: 0.0, // Stationary
            mass: 100000.0,
            collision_radius: 20.0,
            visual_scale: 20.0,
            primitive_type: PrimitiveType::Dodecahedron,
            color: Color::CYAN, // Bright cyan to stand out
        }
    }
}

/// Blob node data - individual blob segment
#[derive(Debug, Clone, PartialEq)]
pub struct BlobNodeData {
    pub core_id: EntityId,
    pub node_type: BlobNodeType,
    pub grid_position: (i32, i32, i32), // 3D grid position relative to core
    pub connected_to_core: bool,
    pub wither_rate: f32, // Cached random wither rate (2-5% per second)
}

impl BlobNodeData {
    pub fn new_base(core_id: EntityId, grid_position: (i32, i32, i32)) -> Self {
        use rand::Rng;
        Self {
            core_id,
            node_type: BlobNodeType::Base,
            grid_position,
            connected_to_core: true,
            wither_rate: rand::rng().random_range(BLOB_WITHER_RATE_MIN..BLOB_WITHER_RATE_MAX),
        }
    }

    pub fn new_factory(core_id: EntityId, grid_position: (i32, i32, i32)) -> Self {
        use rand::Rng;
        Self {
            core_id,
            node_type: BlobNodeType::Factory,
            grid_position,
            connected_to_core: true,
            wither_rate: rand::rng().random_range(BLOB_WITHER_RATE_MIN..BLOB_WITHER_RATE_MAX),
        }
    }

    pub fn upgrade_to_factory(&mut self) {
        self.node_type = BlobNodeType::Factory;
    }

    pub fn is_factory(&self) -> bool {
        self.node_type == BlobNodeType::Factory
    }

    pub fn is_base(&self) -> bool {
        self.node_type == BlobNodeType::Base
    }

    pub fn config(&self) -> EnemyConfig {
        match self.node_type {
            BlobNodeType::Core => unreachable!("Core uses BlobCoreData"),
            BlobNodeType::Factory => EnemyConfig {
                health: 1000.0,
                speed: 0.0,
                mass: 3000.0,
                collision_radius: 15.0,
                visual_scale: 15.0,
                primitive_type: PrimitiveType::Dodecahedron,
                color: Color::YELLOW, // Yellow for factories
            },
            BlobNodeType::Base => EnemyConfig {
                health: 100.0,
                speed: 0.0,
                mass: 2000.0,
                collision_radius: 5.0,
                visual_scale: 8.0,
                primitive_type: PrimitiveType::Cube,
                color: Color::FOREST, // Forest green for base
            },
        }
    }
}

pub fn config() -> EnemyConfig {
    BlobCoreData::new().config()
}

/// Calculate world position from grid position relative to core
pub fn grid_to_world_position(core_pos: Vec3, grid_pos: (i32, i32, i32)) -> Vec3 {
    Vec3::new(
        core_pos.x + grid_pos.0 as f32 * BLOB_GRID_SPACING,
        core_pos.y + grid_pos.1 as f32 * BLOB_GRID_SPACING,
        core_pos.z + grid_pos.2 as f32 * BLOB_GRID_SPACING,
    )
}

/// Get all 6 adjacent grid positions (±X, ±Y, ±Z)
pub fn get_adjacent_positions(pos: (i32, i32, i32)) -> [(i32, i32, i32); 6] {
    [
        (pos.0 + 1, pos.1, pos.2),
        (pos.0 - 1, pos.1, pos.2),
        (pos.0, pos.1 + 1, pos.2),
        (pos.0, pos.1 - 1, pos.2),
        (pos.0, pos.1, pos.2 + 1),
        (pos.0, pos.1, pos.2 - 1),
    ]
}

/// Find direction from current position toward target
pub fn direction_toward_target(
    from: (i32, i32, i32),
    to_world: Vec3,
    core_pos: Vec3,
) -> (i32, i32, i32) {
    let from_world = grid_to_world_position(core_pos, from);
    let direction = to_world - from_world;

    // Find the axis with largest magnitude
    let abs_x = direction.x.abs();
    let abs_y = direction.y.abs();
    let abs_z = direction.z.abs();

    if abs_x >= abs_y && abs_x >= abs_z {
        (direction.x.signum() as i32, 0, 0)
    } else if abs_y >= abs_z {
        (0, direction.y.signum() as i32, 0)
    } else {
        (0, 0, direction.z.signum() as i32)
    }
}

/// Find direction from current position toward core (origin)
pub fn direction_toward_core(from: (i32, i32, i32)) -> (i32, i32, i32) {
    // Find the axis with largest distance from origin
    let abs_x = from.0.abs();
    let abs_y = from.1.abs();
    let abs_z = from.2.abs();

    if abs_x >= abs_y && abs_x >= abs_z {
        (-from.0.signum(), 0, 0)
    } else if abs_y >= abs_z {
        (0, -from.1.signum(), 0)
    } else {
        (0, 0, -from.2.signum())
    }
}

/// Check if a grid position is on the edge of current blob structure
/// Edge means it has at least one empty adjacent position
pub fn is_edge_position(
    pos: (i32, i32, i32),
    occupied_positions: &std::collections::HashSet<(i32, i32, i32)>,
) -> bool {
    get_adjacent_positions(pos)
        .iter()
        .any(|adj| !occupied_positions.contains(adj))
}
