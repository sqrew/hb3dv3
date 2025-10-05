// Bullet system modules
pub mod effects;
pub mod fractal;
pub mod manager;
pub mod pool;
pub mod types;
pub mod visuals;

// Re-export main types for convenience
pub use effects::{ChainLightningEffect, ChainLightningEvent, ProjectileEffects};
pub use fractal::FractalConfig;
pub use manager::BulletManager;
// pub use pool::{BulletPool, PoolStats}; // Unused - commented out
pub use types::{Bullet, MetaBullet, ProjectileType};
pub use visuals::BulletVisuals;
