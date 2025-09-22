// Bullet system modules
pub mod visuals;
pub mod effects;
pub mod types;
pub mod manager;
pub mod fractal;

// Re-export main types for convenience
pub use visuals::BulletVisuals;
pub use effects::{OnHitEffect, OnExpireEffect, ChainLightningEffect, ExplosionEffect, ProjectileEffects, ChainLightningEvent};
pub use types::{ProjectileType, Bullet, MetaBullet};
pub use manager::BulletManager;
pub use fractal::{FractalPattern, FractalConfig, FractalBullet, FractalSplitEvent};