use crate::engine::Vec3;
use crate::engine::dispatcher::{EventType, WeaponEvent};
use crate::input::InputManager;
use crate::scene::bullet::{
    BulletVisuals, ChainLightningEffect, FractalConfig, ProjectileEffects, ProjectileType,
};

// Chain Lightning Configuration Constants
const CHAIN_LIGHTNING_JUMP_RANGE: f32 = 256.0; // Maximum distance for chain lightning jumps
const CHAIN_LIGHTNING_MAX_JUMPS: usize = 32; // Maximum number of jumps
const CHAIN_LIGHTNING_DAMAGE_FALLOFF: f32 = 0.9; // Damage multiplier per jump (75% damage per jump)
const SEEKING_EXPLOSIVE_SEEK_FORCE: f32 = 5000.0;
const SEEKING_EXPLOSIVE_SEEK_RANGE: f32 = 1000.0;
const SEEKING_EXPLOSIVE_EXPLOSION_RADIUS: f32 = 30.0;
const SEEKING_EXPLOSIVE_EXPLOSION_FORCE: f32 = 5000.0;
const SEEKING_EXPLOSIVE_EXPLOSION_DURATION: f32 = 0.5;

#[derive(Debug, Clone)]
pub enum WeaponType {
    BasicBlaster,
    RapidFire,
    Shotgun,
    AntiGravityCannon,
    ChainLightning,
    SeekingExplosive,
    FractalCannon,
}

#[derive(Debug, Clone)]
pub struct WeaponStats {
    pub damage: f32,
    pub fire_rate: f32, // Shots per second
    pub bullet_speed: f32,
    pub bullet_lifetime: f32,
    pub projectile_count: u8, // For shotguns/spread weapons
    pub spread_angle: f32,    // Degrees of spread
    pub bullet_mass: f32,     // Mass of bullets fired by this weapon
}

impl WeaponStats {
    pub fn basic_blaster() -> Self {
        Self {
            damage: 25.0,
            fire_rate: 100.0,
            bullet_speed: 100.0,
            bullet_lifetime: 300.0,
            projectile_count: 1,
            spread_angle: 0.0,
            bullet_mass: 0.5, // Standard positive mass
        }
    }

    pub fn rapid_fire() -> Self {
        Self {
            damage: 15.0,
            fire_rate: 8.0,
            bullet_speed: 25.0,
            bullet_lifetime: 2.5,
            projectile_count: 1,
            spread_angle: 0.0,
            bullet_mass: 0.3, // Lighter bullets for rapid fire
        }
    }

    pub fn shotgun() -> Self {
        Self {
            damage: 12.0,
            fire_rate: 1.5,
            bullet_speed: 18.0,
            bullet_lifetime: 2.0,
            projectile_count: 5,
            spread_angle: 15.0,
            bullet_mass: 0.6, // Heavier pellets for shotgun
        }
    }

    pub fn anti_gravity_cannon() -> Self {
        Self {
            damage: 25.0,
            fire_rate: 100.0,       // Slower rate for powerful exotic rounds
            bullet_speed: 25.0,     // Starts slow but accelerates away from gravity
            bullet_lifetime: 300.0, // Lives longer to show crazy physics
            projectile_count: 1,
            spread_angle: 0.0,
            bullet_mass: -5.0, // Negative mass for anti-gravity effects!
        }
    }

    pub fn chain_lightning() -> Self {
        Self {
            damage: 50.0,         // Base damage for initial hit
            fire_rate: 1.0,       // Moderate firing rate
            bullet_speed: 200.0,  // Fast but trackable projectile
            bullet_lifetime: 5.0, // Decent range
            projectile_count: 1,  // Single lightning bolt
            spread_angle: 0.0,    // Precise targeting
            bullet_mass: 0.1,     // Very light - minimal gravity effect
        }
    }

    pub fn seeking_explosive() -> Self {
        Self {
            damage: 50.0,          // Base damage - explosion adds area effect
            fire_rate: 2.0,        // Slower rate for powerful explosive
            bullet_speed: 200.0,   // Starts slow to allow seeking time
            bullet_lifetime: 10.0, // Long lifetime for seeking behavior
            projectile_count: 1,   // Single seeking missile
            spread_angle: 0.0,     // Precise targeting
            bullet_mass: 1.0,      // Keep at 1.0 for seeking speed scaling
        }
    }

    pub fn fractal_cannon() -> Self {
        Self {
            damage: 30.0,          // Base damage - multiplies with fractal generations
            fire_rate: 2.0,        // Slower rate for spectacular fractal shots
            bullet_speed: 150.0,   // Moderate speed for good fractal development
            bullet_lifetime: 60.0, // Long enough for deep fractals to unfold
            projectile_count: 1,   // Single fractal parent
            spread_angle: 0.0,     // Precise initial shot
            bullet_mass: 0.8,      // Light for fractal dynamics
        }
    }
}

pub struct Weapon {
    weapon_type: WeaponType,
    stats: WeaponStats,
    last_fire_time: f32,
    total_time: f32,
}

impl Weapon {
    pub fn new(weapon_type: WeaponType) -> Self {
        let stats = match weapon_type {
            WeaponType::BasicBlaster => WeaponStats::basic_blaster(),
            WeaponType::RapidFire => WeaponStats::rapid_fire(),
            WeaponType::Shotgun => WeaponStats::shotgun(),
            WeaponType::AntiGravityCannon => WeaponStats::anti_gravity_cannon(),
            WeaponType::ChainLightning => WeaponStats::chain_lightning(),
            WeaponType::SeekingExplosive => WeaponStats::seeking_explosive(),
            WeaponType::FractalCannon => WeaponStats::fractal_cannon(),
        };

        Self {
            weapon_type,
            stats,
            last_fire_time: 0.0,
            total_time: 0.0,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.total_time += delta_time;
    }

    pub fn can_fire(&self) -> bool {
        let time_since_last_shot = self.total_time - self.last_fire_time;
        time_since_last_shot >= (1.0 / self.stats.fire_rate)
    }

    pub fn try_fire(&mut self, origin: Vec3, direction: Vec3) -> Option<Vec<BulletSpawnRequest>> {
        if !self.can_fire() {
            return None;
        }

        self.last_fire_time = self.total_time;

        let mut requests = Vec::new();

        // Calculate spread for multiple projectiles
        let spread_rad = self.stats.spread_angle.to_radians();
        let half_spread = spread_rad / 2.0;

        for i in 0..self.stats.projectile_count {
            // Calculate direction for this projectile
            let projectile_direction = if self.stats.projectile_count == 1 {
                // Single projectile - use original direction
                direction.normalize()
            } else {
                // Multiple projectiles - apply spread
                let t = (i as f32) / (self.stats.projectile_count - 1) as f32;
                let angle_offset = -half_spread + (t * spread_rad);

                // Rotate the direction vector around the up axis
                let cos_angle = angle_offset.cos();
                let sin_angle = angle_offset.sin();

                Vec3::new(
                    direction.x * cos_angle - direction.z * sin_angle,
                    direction.y,
                    direction.x * sin_angle + direction.z * cos_angle,
                )
                .normalize()
            };

            // Determine projectile type based on weapon
            let projectile_type = match self.weapon_type {
                WeaponType::ChainLightning => {
                    // Create chain lightning projectile with custom effects
                    let chain_effect = ChainLightningEffect::new(
                        self.stats.damage,              // Base damage for chain
                        CHAIN_LIGHTNING_MAX_JUMPS,      // Max jumps from constants
                        CHAIN_LIGHTNING_JUMP_RANGE,     // Jump range from constants
                        CHAIN_LIGHTNING_DAMAGE_FALLOFF, // Damage falloff from constants
                    );

                    let effects = ProjectileEffects {
                        on_hit: Some(vec![Box::new(chain_effect)]),
                        on_expire: None,
                    };

                    ProjectileType::Custom {
                        damage: self.stats.damage,
                        velocity: projectile_direction * self.stats.bullet_speed,
                        lifetime: self.stats.bullet_lifetime,
                        mass: self.stats.bullet_mass,
                        effects,
                        visuals: BulletVisuals::chain_lightning(),
                    }
                }
                WeaponType::SeekingExplosive => {
                    // Create seeking explosive projectile
                    ProjectileType::SeekingExplosive {
                        damage: self.stats.damage,
                        velocity: projectile_direction * self.stats.bullet_speed,
                        lifetime: self.stats.bullet_lifetime,
                        mass: self.stats.bullet_mass,
                        seeking_force: SEEKING_EXPLOSIVE_SEEK_FORCE,
                        seeking_range: SEEKING_EXPLOSIVE_SEEK_RANGE,
                        explosion_radius: SEEKING_EXPLOSIVE_EXPLOSION_RADIUS,
                        explosion_force: SEEKING_EXPLOSIVE_EXPLOSION_FORCE,
                        explosion_duration: SEEKING_EXPLOSIVE_EXPLOSION_DURATION,
                        visuals: BulletVisuals::seeking_explosive(),
                    }
                }
                WeaponType::FractalCannon => {
                    // Create fractal projectile with random pattern
                    ProjectileType::Fractal {
                        damage: self.stats.damage,
                        velocity: projectile_direction * self.stats.bullet_speed,
                        lifetime: self.stats.bullet_lifetime,
                        mass: self.stats.bullet_mass,
                        fractal_config: FractalConfig::random(), // Random fractal pattern each shot!
                        visuals: BulletVisuals::fractal_cannon(),
                    }
                }
                _ => {
                    // Standard projectile - map weapon type to visuals
                    let visuals = match self.weapon_type {
                        WeaponType::BasicBlaster => BulletVisuals::basic_blaster(),
                        WeaponType::RapidFire => BulletVisuals::rapid_fire(),
                        WeaponType::Shotgun => BulletVisuals::shotgun(),
                        WeaponType::AntiGravityCannon => BulletVisuals::anti_gravity(),
                        _ => BulletVisuals::basic_blaster(), // Fallback
                    };

                    ProjectileType::Basic {
                        damage: self.stats.damage,
                        velocity: projectile_direction * self.stats.bullet_speed,
                        lifetime: self.stats.bullet_lifetime,
                        mass: self.stats.bullet_mass,
                        visuals,
                    }
                }
            };

            requests.push(BulletSpawnRequest {
                position: origin,
                direction: projectile_direction,
                speed: self.stats.bullet_speed,
                lifetime: self.stats.bullet_lifetime,
                damage: self.stats.damage,
                mass: self.stats.bullet_mass,
                projectile_type,
            });
        }

        Some(requests)
    }

    pub fn weapon_type(&self) -> &WeaponType {
        &self.weapon_type
    }

    pub fn stats(&self) -> &WeaponStats {
        &self.stats
    }
}

#[derive(Debug)]
pub struct BulletSpawnRequest {
    pub position: Vec3,
    pub direction: Vec3,
    pub speed: f32,
    pub lifetime: f32,
    pub damage: f32,
    pub mass: f32,                       // Allow custom mass for exotic bullets
    pub projectile_type: ProjectileType, // What kind of projectile to spawn
}

pub struct WeaponManager {
    current_weapon: Weapon,
    available_weapons: Vec<WeaponType>,
    current_weapon_index: usize,
    event_queue: Vec<EventType>,
}

impl WeaponManager {
    pub fn new() -> Self {
        let available_weapons = vec![
            WeaponType::BasicBlaster,
            // WeaponType::RapidFire,
            // WeaponType::Shotgun,
            // WeaponType::AntiGravityCannon,
            WeaponType::ChainLightning,
            WeaponType::SeekingExplosive,
            WeaponType::FractalCannon,
        ];

        Self {
            current_weapon: Weapon::new(WeaponType::BasicBlaster),
            available_weapons,
            current_weapon_index: 0,
            event_queue: Vec::new(),
        }
    }

    pub fn update(&mut self, delta_time: f32, input: &InputManager) {
        self.current_weapon.update(delta_time);

        // Handle weapon switching
        if input.is_action_just_pressed(crate::input::Action::NextWeapon) {
            self.cycle_weapon();
        }
        if input.is_action_just_pressed(crate::input::Action::PrevWeapon) {
            self.cycle_weapon_backward();
        }
    }

    pub fn try_fire(&mut self, origin: Vec3, direction: Vec3) -> Option<Vec<BulletSpawnRequest>> {
        if let Some(bullet_requests) = self.current_weapon.try_fire(origin, direction) {
            // Generate weapon fired event
            self.event_queue.push(EventType::Weapon(WeaponEvent::Fired {
                weapon_type: self.current_weapon.weapon_type().clone(),
                position: origin,
                direction,
                projectile_count: bullet_requests.len() as u32,
            }));

            Some(bullet_requests)
        } else {
            None
        }
    }

    /// Get and clear weapon events
    pub fn drain_events(&mut self) -> Vec<EventType> {
        self.event_queue.drain(..).collect()
    }

    /// Check if there are pending events
    pub fn has_events(&self) -> bool {
        !self.event_queue.is_empty()
    }

    pub fn current_weapon(&self) -> &Weapon {
        &self.current_weapon
    }

    pub fn switch_weapon(&mut self, weapon_type: WeaponType) {
        if let Some(index) = self
            .available_weapons
            .iter()
            .position(|w| std::mem::discriminant(w) == std::mem::discriminant(&weapon_type))
        {
            self.current_weapon_index = index;
            self.current_weapon = Weapon::new(weapon_type);
        }
    }

    pub fn cycle_weapon(&mut self) {
        self.current_weapon_index = (self.current_weapon_index + 1) % self.available_weapons.len();
        let new_weapon_type = self.available_weapons[self.current_weapon_index].clone();
        self.current_weapon = Weapon::new(new_weapon_type);
        println!("🔫 Switched to {:?}", self.current_weapon.weapon_type());
    }

    pub fn cycle_weapon_backward(&mut self) {
        if self.current_weapon_index == 0 {
            self.current_weapon_index = self.available_weapons.len() - 1;
        } else {
            self.current_weapon_index -= 1;
        }
        let new_weapon_type = self.available_weapons[self.current_weapon_index].clone();
        self.current_weapon = Weapon::new(new_weapon_type);
        println!("🔫 Switched to {:?}", self.current_weapon.weapon_type());
    }
}
