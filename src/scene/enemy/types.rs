use super::behaviors::{blob, cannibal, chaser, drone, heavy, shield, snake, splitter};
use crate::graphics::{Color, PrimitiveType};

#[derive(Debug, Clone, PartialEq)]
pub enum EnemyType {
    Heavy,
    Chaser,
    Drone,
    Splitter(splitter::SplitterData),
    Cannibal(cannibal::CannibalData),
    Shield(shield::ShieldData),
    ShieldOrbCore(shield::ShieldOrbCoreData),
    Snake(snake::SnakeData),
    SnakeSegment(snake::SnakeSegmentData),
    BlobCore(blob::BlobCoreData),
    BlobNode(blob::BlobNodeData),
}

#[derive(Debug, Clone)]
pub struct EnemyConfig {
    pub health: f32,
    pub speed: f32,
    pub mass: f32,
    pub collision_radius: f32,
    pub visual_scale: f32,
    pub primitive_type: PrimitiveType,
    pub color: Color,
}

impl EnemyType {
    pub fn config(&self) -> EnemyConfig {
        match self {
            EnemyType::Heavy => heavy::config(),
            EnemyType::Chaser => chaser::config(),
            EnemyType::Drone => drone::config(),
            EnemyType::Splitter(data) => data.config(),
            EnemyType::Cannibal(data) => data.config(),
            EnemyType::Shield(data) => data.config(),
            EnemyType::ShieldOrbCore(data) => data.config(),
            EnemyType::Snake(data) => data.config(),
            EnemyType::SnakeSegment(data) => data.config(),
            EnemyType::BlobCore(data) => data.config(),
            EnemyType::BlobNode(data) => data.config(),
        }
    }

    pub fn random() -> Self {
        use rand::Rng;
        match rand::rng().random_range(0..3) {
            0 => EnemyType::Heavy,
            1 => EnemyType::Chaser,
            _ => EnemyType::Drone,
        }
    }
}
