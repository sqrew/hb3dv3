use crate::engine::Vec3;
use crate::engine::dispatcher::{EventType, WeaponEvent};
use crate::input::InputManager;

#[derive(Debug, Clone)]
pub enum WeaponType {
    BasicBlaster,
    RapidFire,
    Shotgun,
    AntiGravityCannon,
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
            bullet_mass: -5.0,      // Negative mass for anti-gravity effects!
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

        if self.stats.projectile_count == 1 {
            // Single projectile
            requests.push(BulletSpawnRequest {
                position: origin,
                direction: direction.normalize(),
                speed: self.stats.bullet_speed,
                lifetime: self.stats.bullet_lifetime,
                damage: self.stats.damage,
                mass: self.stats.bullet_mass,
            });
        } else {
            // Multiple projectiles (shotgun-style)
            let spread_rad = self.stats.spread_angle.to_radians();
            let half_spread = spread_rad / 2.0;

            for i in 0..self.stats.projectile_count {
                let t = if self.stats.projectile_count == 1 {
                    0.0
                } else {
                    (i as f32) / (self.stats.projectile_count - 1) as f32
                };

                let angle_offset = -half_spread + (t * spread_rad);

                // Rotate the direction vector around the up axis
                let cos_angle = angle_offset.cos();
                let sin_angle = angle_offset.sin();

                let spread_direction = Vec3::new(
                    direction.x * cos_angle - direction.z * sin_angle,
                    direction.y,
                    direction.x * sin_angle + direction.z * cos_angle,
                );

                requests.push(BulletSpawnRequest {
                    position: origin,
                    direction: spread_direction.normalize(),
                    speed: self.stats.bullet_speed,
                    lifetime: self.stats.bullet_lifetime,
                    damage: self.stats.damage,
                    mass: self.stats.bullet_mass,
                });
            }
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
    pub mass: f32, // Allow custom mass for exotic bullets
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
            WeaponType::RapidFire,
            WeaponType::Shotgun,
            WeaponType::AntiGravityCannon,
        ];

        Self {
            current_weapon: Weapon::new(WeaponType::AntiGravityCannon),
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
