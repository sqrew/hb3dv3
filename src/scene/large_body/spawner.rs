use super::body_type::LargeBodyType;
use crate::engine::Vec3;

/// Type of spawn request from the spawner
#[derive(Debug, Clone)]
pub enum SpawnRequest {
    /// Spawn a single large body
    SingleBody {
        body_type: LargeBodyType,
        position: Vec3,
        lifetime: f32,
    },
    /// Spawn an asteroid shower event
    AsteroidShower {
        direction: Vec3,         // Direction asteroids come from
        count: usize,            // Number of asteroids
        speed_range: (f32, f32), // (min, max) speed
        spread_angle: f32,       // Cone spread angle
        lifetime: f32,           // Lifetime for asteroids
    },
    /// Spawn a binary pair of orbiting bodies
    BinaryPair {
        body_type1: LargeBodyType,
        body_type2: LargeBodyType,
        center_position: Vec3,
        separation_distance: f32,
        lifetime: f32,
    },
    /// Spawn a rogue body moving at high velocity
    RogueBody {
        body_type: LargeBodyType,
        position: Vec3,
        velocity: Vec3,
        lifetime: f32,
    },
}

/// Automatic spawner for maintaining a population of large bodies
#[derive(Debug, Clone)]
pub struct BodySpawner {
    // Pool management
    target_body_count: usize, // Maintain approximately this many bodies
    min_body_count: usize,    // Spawn more aggressively if below this
    max_body_count: usize,    // Stop spawning if above this

    // Spawn timing
    spawn_interval: f32, // Minimum seconds between spawns
    spawn_timer: f32,    // Current countdown timer

    // Lifetime configuration
    lifetime_min: f32, // Minimum lifetime for spawned bodies (seconds)
    lifetime_max: f32, // Maximum lifetime for spawned bodies (seconds)

    // Spawn location strategy
    spawn_radius_min: f32,    // Minimum distance from origin to spawn
    spawn_radius_max: f32,    // Maximum distance from origin to spawn
    avoid_player_radius: f32, // Don't spawn within this distance of player

    // Body type selection (simple approach - equal probability for now)
    enabled_body_types: Vec<LargeBodyType>, // Types that can be spawned

    // Asteroid shower event configuration
    asteroid_shower_chance: f32, // Probability of asteroid shower (0.0-1.0)
    asteroid_count_range: (usize, usize), // (min, max) asteroids in shower
    asteroid_speed_range: (f32, f32), // (min, max) speed in units/sec
    asteroid_spread_angle: f32,  // Cone spread angle in radians
    asteroid_lifetime: f32,      // Lifetime for shower asteroids

    // Binary pair event configuration
    binary_pair_chance: f32, // Probability of binary pair (0.0-1.0)
    binary_separation_range: (f32, f32), // (min, max) separation distance

    // Rogue body event configuration
    rogue_body_chance: f32,        // Probability of rogue body (0.0-1.0)
    rogue_speed_range: (f32, f32), // (min, max) speed in units/sec
}

impl BodySpawner {
    /// Create a new body spawner with default configuration
    pub fn new() -> Self {
        Self {
            target_body_count: 6,
            min_body_count: 3,
            max_body_count: 10,
            spawn_interval: 4.0,
            spawn_timer: 4.0, // Start with first spawn ready
            lifetime_min: 20.0,
            lifetime_max: 60.0,
            spawn_radius_min: 100.0,
            spawn_radius_max: 1000.0,
            avoid_player_radius: 100.0,
            // THIS IS THE LIST OF BODY TYPES ALLOWED TO BE SPAWNED BY THE SPAWNER
            // LauncherMass removed - only appears in asteroid showers
            // BlackHoleLarge NOT INCLUDED; only spawns from BlackHole on_death_sequence_complete
            //
            enabled_body_types: vec![
                LargeBodyType::Planet,
                LargeBodyType::Star,
                LargeBodyType::GasGiant,
                LargeBodyType::NeutronStar,
                LargeBodyType::BlackHole,
                LargeBodyType::WhiteHole,
                LargeBodyType::ExoticMatter,
            ],
            // Asteroid shower configuration (5% chance for dramatic effect)
            asteroid_shower_chance: 0.05,
            asteroid_count_range: (20, 60),
            asteroid_speed_range: (50.0, 250.0),
            asteroid_spread_angle: 0.5, // ~28 degrees cone
            asteroid_lifetime: 30.0,

            // Binary pair configuration (10% chance for interesting dynamics)
            binary_pair_chance: 0.10,
            binary_separation_range: (100.0, 1000.0),

            // Rogue body configuration (10% chance for chaotic element)
            rogue_body_chance: 0.10,
            rogue_speed_range: (100.0, 500.0),
        }
    }

    /// Create a custom spawner with specific configuration
    #[allow(clippy::too_many_arguments)]
    pub fn new_custom(
        target_body_count: usize,
        min_body_count: usize,
        max_body_count: usize,
        spawn_interval: f32,
        lifetime_min: f32,
        lifetime_max: f32,
        spawn_radius_min: f32,
        spawn_radius_max: f32,
        avoid_player_radius: f32,
        enabled_body_types: Vec<LargeBodyType>,
    ) -> Self {
        Self {
            target_body_count,
            min_body_count,
            max_body_count,
            spawn_interval,
            spawn_timer: spawn_interval,
            lifetime_min,
            lifetime_max,
            spawn_radius_min,
            spawn_radius_max,
            avoid_player_radius,
            enabled_body_types,
            // Use default asteroid shower config
            asteroid_shower_chance: 0.08,
            asteroid_count_range: (20, 40),
            asteroid_speed_range: (80.0, 150.0),
            asteroid_spread_angle: 0.5,
            asteroid_lifetime: 30.0,
            // Use default binary pair config
            binary_pair_chance: 0.10,
            binary_separation_range: (80.0, 200.0),
            // Use default rogue body config
            rogue_body_chance: 0.08,
            rogue_speed_range: (100.0, 300.0),
        }
    }

    /// Update spawner and potentially return a spawn request
    /// Returns Some(SpawnRequest) if something should be spawned
    pub fn update(
        &mut self,
        delta_time: f32,
        current_body_count: usize,
        player_position: Vec3,
    ) -> Option<SpawnRequest> {
        // Update spawn timer
        self.spawn_timer -= delta_time;

        // Check if we should spawn
        let should_spawn = self.spawn_timer <= 0.0
            && current_body_count < self.max_body_count
            && (current_body_count < self.target_body_count
                || current_body_count < self.min_body_count);

        if !should_spawn {
            return None;
        }

        // Reset timer
        self.spawn_timer = self.spawn_interval;

        // Roll for special events (asteroid shower, binary pair, rogue body)
        let event_roll = rand::random::<f32>();

        // Roll for asteroid shower event!
        if event_roll < self.asteroid_shower_chance {
            // ASTEROID SHOWER EVENT!
            // Generate random direction (spherical coordinates)
            let theta = rand::random::<f32>() * std::f32::consts::TAU; // 0 to 2π
            let phi = rand::random::<f32>() * std::f32::consts::PI; // 0 to π

            let direction = Vec3::new(phi.sin() * theta.cos(), phi.sin() * theta.sin(), phi.cos());

            // Random count within range
            let count = self.asteroid_count_range.0
                + (rand::random::<f32>()
                    * (self.asteroid_count_range.1 - self.asteroid_count_range.0) as f32)
                    as usize;

            return Some(SpawnRequest::AsteroidShower {
                direction,
                count,
                speed_range: self.asteroid_speed_range,
                spread_angle: self.asteroid_spread_angle,
                lifetime: self.asteroid_lifetime,
            });
        }

        // Roll for binary pair event!
        if event_roll < self.asteroid_shower_chance + self.binary_pair_chance {
            // BINARY PAIR EVENT!
            // Select two random body types (they can be the same or different)
            if self.enabled_body_types.is_empty() {
                return None;
            }

            let body_type1_index =
                (rand::random::<f32>() * self.enabled_body_types.len() as f32).floor() as usize;
            let body_type1 =
                self.enabled_body_types[body_type1_index % self.enabled_body_types.len()];

            let body_type2_index =
                (rand::random::<f32>() * self.enabled_body_types.len() as f32).floor() as usize;
            let body_type2 =
                self.enabled_body_types[body_type2_index % self.enabled_body_types.len()];

            // Generate random spawn position
            let center_position = self.generate_spawn_position(player_position);

            // Random separation distance
            let separation = self.binary_separation_range.0
                + rand::random::<f32>()
                    * (self.binary_separation_range.1 - self.binary_separation_range.0);

            // Generate random lifetime
            let lifetime =
                self.lifetime_min + rand::random::<f32>() * (self.lifetime_max - self.lifetime_min);

            return Some(SpawnRequest::BinaryPair {
                body_type1,
                body_type2,
                center_position,
                separation_distance: separation,
                lifetime,
            });
        }

        // Roll for rogue body event!
        if event_roll
            < self.asteroid_shower_chance + self.binary_pair_chance + self.rogue_body_chance
        {
            // ROGUE BODY EVENT!
            // Select random body type
            if self.enabled_body_types.is_empty() {
                return None;
            }

            let body_type_index =
                (rand::random::<f32>() * self.enabled_body_types.len() as f32).floor() as usize;
            let body_type =
                self.enabled_body_types[body_type_index % self.enabled_body_types.len()];

            // Generate random spawn position (far from center)
            let spawn_position = self.generate_spawn_position(player_position);

            // Generate random velocity direction (toward center with some randomness)
            let to_center = -spawn_position.normalize();
            let random_offset = Vec3::new(
                (rand::random::<f32>() - 0.5) * 0.3,
                (rand::random::<f32>() - 0.5) * 0.3,
                (rand::random::<f32>() - 0.5) * 0.3,
            );
            let velocity_direction = (to_center + random_offset).normalize();

            // Random speed within range
            let speed = self.rogue_speed_range.0
                + rand::random::<f32>() * (self.rogue_speed_range.1 - self.rogue_speed_range.0);

            let velocity = velocity_direction * speed;

            // Generate random lifetime
            let lifetime =
                self.lifetime_min + rand::random::<f32>() * (self.lifetime_max - self.lifetime_min);

            return Some(SpawnRequest::RogueBody {
                body_type,
                position: spawn_position,
                velocity,
                lifetime,
            });
        }

        // Normal single body spawn (fallback)
        // Select random body type
        if self.enabled_body_types.is_empty() {
            return None;
        }

        let body_type_index =
            (rand::random::<f32>() * self.enabled_body_types.len() as f32).floor() as usize;
        let body_type = self.enabled_body_types[body_type_index % self.enabled_body_types.len()];

        // Generate random spawn position
        let spawn_position = self.generate_spawn_position(player_position);

        // Generate random lifetime
        let lifetime =
            self.lifetime_min + rand::random::<f32>() * (self.lifetime_max - self.lifetime_min);

        Some(SpawnRequest::SingleBody {
            body_type,
            position: spawn_position,
            lifetime,
        })
    }

    /// Generate a random spawn position based on configuration
    fn generate_spawn_position(&self, player_position: Vec3) -> Vec3 {
        let max_attempts = 20;
        for _ in 0..max_attempts {
            // Generate random angle
            let theta = rand::random::<f32>() * std::f32::consts::TAU; // 0 to 2π
            let phi = rand::random::<f32>() * std::f32::consts::PI; // 0 to π

            // Generate random radius within range
            let radius = self.spawn_radius_min
                + rand::random::<f32>() * (self.spawn_radius_max - self.spawn_radius_min);

            // Convert spherical to Cartesian coordinates
            let x = radius * phi.sin() * theta.cos();
            let y = radius * phi.sin() * theta.sin();
            let z = radius * phi.cos();

            let position = Vec3::new(x, y, z);

            // Check distance from player
            let distance_to_player = (position - player_position).magnitude();
            if distance_to_player >= self.avoid_player_radius {
                return position;
            }
        }

        // Fallback: spawn at max radius on X-axis
        Vec3::new(self.spawn_radius_max, 0.0, 0.0)
    }

    // Getters and setters for runtime configuration
    pub fn set_target_body_count(&mut self, count: usize) {
        self.target_body_count = count;
    }

    pub fn set_spawn_interval(&mut self, interval: f32) {
        self.spawn_interval = interval;
    }

    pub fn set_lifetime_range(&mut self, min: f32, max: f32) {
        self.lifetime_min = min;
        self.lifetime_max = max;
    }

    pub fn add_body_type(&mut self, body_type: LargeBodyType) {
        if !self.enabled_body_types.contains(&body_type) {
            self.enabled_body_types.push(body_type);
        }
    }

    pub fn remove_body_type(&mut self, body_type: LargeBodyType) {
        self.enabled_body_types.retain(|t| *t != body_type);
    }
}
