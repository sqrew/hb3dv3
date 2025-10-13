// Large body module - manages gravitational celestial bodies

mod body;
mod body_type;
mod manager;
mod spawner;

// Re-export public types
pub use body::{BodySpawnRequest, LargeBody};
pub use body_type::{DeathState, LargeBodyType};
pub use manager::LargeBodyManager;
pub use spawner::{BodySpawner, SpawnRequest};
