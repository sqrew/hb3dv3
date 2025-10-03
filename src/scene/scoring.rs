use crate::engine::dispatcher::{EventType, GraphicsEvent};
use crate::engine::entity::{EntityId, EntityType};
use crate::engine::{CollisionMask, Vec3};
use crate::graphics::{Color, Primitive, PrimitiveType};
use crate::scene::GravityAffected;

const ATTRACTION_RANGE: f32 = 150.0; // range in units
const ATTRACTION_FORCE_MULTIPLIER: f32 = 100.0; // arbitrary number honestly
const COLLECTIBLE_AIR_DRAG: f32 = 0.995; // slight amount of drag for floatiness

#[derive(Debug, Clone, Copy)]
pub struct ScoreMultiplier {
    entity_id: EntityId,
    pos: Vec3,
    vel: Vec3,
    multiplier_value: f32,
    collision_radius: f32,
    collision_mask: CollisionMask,
    lifetime: f32,
    max_lifetime: f32,
    mass: f32,
    applied_force: Vec3,
}

impl ScoreMultiplier {
    pub fn new(entity_id: EntityId, pos: Vec3, vel: Vec3, multiplier_value: f32) -> Self {
        ScoreMultiplier {
            entity_id,
            pos,
            vel,
            multiplier_value,
            collision_radius: 3.0,
            collision_mask: CollisionMask::from(EntityType::Collectible),
            lifetime: 30.0, // 30 second lifetime
            max_lifetime: 30.0,
            mass: 1000.0, // Light mass for gentle floating
            applied_force: Vec3::zeros(),
        }
    }

    pub fn update(&mut self, dt: f32, player_pos: Vec3) {
        // Apply gravitational forces (F = ma, so a = F/m)
        let gravity_acceleration = self.applied_force / self.mass;

        // Calculate player attraction
        let to_player = player_pos - self.pos;
        let distance = to_player.magnitude();

        // Apply attraction when player is within attraction range (15 units)
        let attraction_range = 200.0;
        if distance > 0.1 && distance < attraction_range {
            // Stronger attraction when closer, using inverse square falloff
            let attraction_strength = (attraction_range - distance) / attraction_range;
            let attraction_force =
                to_player.normalize() * attraction_strength * ATTRACTION_FORCE_MULTIPLIER; // Base attraction force

            // Apply attraction as acceleration
            self.vel += attraction_force * dt;
        }

        // Update velocity with gravity
        self.vel += gravity_acceleration * dt;

        // Apply gentle drag for floating effect
        self.vel *= COLLECTIBLE_AIR_DRAG;

        // Add gentle floating upward tendency (reduced when under attraction)
        let float_strength = if distance < attraction_range {
            0.5
        } else {
            2.0
        };
        self.vel.y += float_strength * dt;

        // Update position
        self.pos += self.vel * dt;

        // Update lifetime
        self.lifetime -= dt;

        // Reset applied force for next frame
        self.applied_force = Vec3::zeros();
    }

    pub fn is_alive(&self) -> bool {
        self.lifetime > 0.0
    }

    pub fn position(&self) -> Vec3 {
        self.pos
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    pub fn collision_radius(&self) -> f32 {
        self.collision_radius
    }

    pub fn collision_mask(&self) -> CollisionMask {
        self.collision_mask
    }

    pub fn multiplier_value(&self) -> f32 {
        self.multiplier_value
    }

    pub fn lifetime_ratio(&self) -> f32 {
        self.lifetime / self.max_lifetime
    }
}

impl GravityAffected for ScoreMultiplier {
    fn position(&self) -> Vec3 {
        self.pos
    }

    fn mass(&self) -> f32 {
        self.mass
    }

    fn apply_force(&mut self, force: Vec3) {
        self.applied_force += force;
    }
}

pub struct ScoringSystem {
    score: u64,
    multiplier: f32,
    base_multiplier: f32,
    multiplier_decay_rate: f32,
    multiplier_decay_timer: f32,
    multiplier_decay_interval: f32,
    event_queue: Vec<EventType>,
    multipliers: Vec<ScoreMultiplier>,
}

impl ScoringSystem {
    pub fn new() -> Self {
        ScoringSystem {
            score: 0,
            multiplier: 1.0,
            base_multiplier: 1.0,
            multiplier_decay_rate: 0.1, // Multiplier decreases by 10% every interval
            multiplier_decay_timer: 0.0,
            multiplier_decay_interval: 5.0, // Decay every 5 seconds
            event_queue: Vec::new(),
            multipliers: Vec::new(),
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        player_pos: Vec3,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) {
        // Update multiplier decay
        self.multiplier_decay_timer += dt;
        if self.multiplier_decay_timer >= self.multiplier_decay_interval {
            self.multiplier_decay_timer = 0.0;

            // Decay multiplier towards base multiplier
            if self.multiplier > self.base_multiplier {
                let decay_amount =
                    (self.multiplier - self.base_multiplier) * self.multiplier_decay_rate;
                self.multiplier = (self.multiplier - decay_amount).max(self.base_multiplier);
            }
        }

        // Update all score multipliers with player position for attraction
        for multiplier in self.multipliers.iter_mut() {
            multiplier.update(dt, player_pos);
        }

        // Clean up expired multipliers
        let mut multipliers_to_remove = Vec::new();
        for (index, multiplier) in self.multipliers.iter().enumerate() {
            if !multiplier.is_alive() {
                multipliers_to_remove.push((index, multiplier.entity_id()));
            }
        }

        // Remove expired multipliers in reverse order to maintain indices
        for (index, entity_id) in multipliers_to_remove.into_iter().rev() {
            self.multipliers.remove(index);
            entity_manager.destroy_entity(entity_id);
        }
    }

    pub fn add_score(&mut self, base_points: u64) {
        let points_to_add = (base_points as f32 * self.multiplier) as u64;
        self.score += points_to_add;
    }

    pub fn collect_multiplier(&mut self, multiplier_value: f32, entity_id: EntityId) -> bool {
        // Find and remove the collected multiplier
        if let Some(index) = self
            .multipliers
            .iter()
            .position(|m| m.entity_id() == entity_id)
        {
            let collected = self.multipliers.remove(index);

            // Add to current multiplier
            self.multiplier += multiplier_value;

            // Generate collection particle effect
            self.event_queue
                .push(EventType::Graphics(GraphicsEvent::SpawnParticles {
                    position: collected.position(),
                    velocity: Vec3::new(0.0, 3.0, 0.0),
                    count: 20,
                    lifetime: 2.0,
                    color: Color::GREEN,
                }));
            true
        } else {
            false
        }
    }

    pub fn spawn_multiplier(
        &mut self,
        position: Vec3,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) {
        use rand::Rng;

        // Random spawn velocity with slight upward bias
        let spawn_vel = Vec3::new(
            rand::rng().random_range(-2.0..2.0),
            rand::rng().random_range(1.0..4.0), // Slight upward velocity
            rand::rng().random_range(-2.0..2.0),
        );

        // Create entity
        let multiplier_entity = entity_manager.create_entity(EntityType::Collectible);

        // Create multiplier (fixed value for now)
        let multiplier = ScoreMultiplier::new(multiplier_entity, position, spawn_vel, 0.1);
        self.multipliers.push(multiplier);

        // Spawn creation particle effect
        self.event_queue
            .push(EventType::Graphics(GraphicsEvent::SpawnParticles {
                position,
                velocity: Vec3::new(0.0, 2.0, 0.0),
                count: 10,
                lifetime: 1.0,
                color: Color::GREEN,
            }));
    }

    pub fn get_render_data(&self) -> Vec<Primitive> {
        self.multipliers
            .iter()
            .map(|multiplier| {
                // Fade alpha based on lifetime for visual feedback
                let alpha = (multiplier.lifetime_ratio() * 0.7 + 0.3).min(1.0);
                let color = Color::new(0.0, 0.8, 0.0, alpha); // Green with fading alpha

                Primitive::new(PrimitiveType::Tetrahedron, multiplier.pos, color)
            })
            .collect()
    }

    pub fn multipliers(&self) -> &[ScoreMultiplier] {
        &self.multipliers
    }

    pub fn score(&self) -> u64 {
        self.score
    }

    pub fn multiplier(&self) -> f32 {
        self.multiplier
    }

    pub fn reset_multiplier(&mut self) {
        self.multiplier = self.base_multiplier;
        self.multiplier_decay_timer = 0.0; // Reset decay timer
    }

    pub fn drain_events(&mut self) -> Vec<EventType> {
        self.event_queue.drain(..).collect()
    }

    // Enemy base points by type
    pub fn get_enemy_base_points(enemy_type: &crate::scene::enemy::EnemyType) -> u64 {
        match enemy_type {
            crate::scene::enemy::EnemyType::Drone => 100, // Pink cubes - basic enemies
            crate::scene::enemy::EnemyType::Chaser => 150, // Green octahedrons - more aggressive
            crate::scene::enemy::EnemyType::Heavy => 200, // Blue cylinders - tanky enemies
            crate::scene::enemy::EnemyType::Splitter(data) => {
                // Exponential scoring: higher generations (smaller enemies) worth less individually
                // But total HP pool across all splits increases, so total score potential increases
                let base = 500;
                let generation_multiplier = 0.7_f64.powi(data.current_generation as i32);
                (base as f64 * generation_multiplier) as u64
            }
            crate::scene::enemy::EnemyType::Cannibal(data) => {
                // Higher value based on how many enemies it consumed
                // Base 200 + 50 per meal (can be worth up to 700 if fully fed)
                200 + ((data.meals_consumed as u64) * 50)
            }
            crate::scene::enemy::EnemyType::Shield(data) => {
                // Shields are worth less as they split (similar to splitter)
                let base = 150;
                let generation_multiplier = 0.7_f64.powi(data.current_generation as i32);
                (base as f64 * generation_multiplier) as u64
            }
            crate::scene::enemy::EnemyType::ShieldOrbCore(_) => {
                // Boss enemy - high value
                1000
            }
            crate::scene::enemy::EnemyType::Snake(data) => {
                // Snake head value scales with number of segments
                300 + (data.segment_count() as u64 * 50)
            }
            crate::scene::enemy::EnemyType::SnakeSegment(_) => {
                // Segments are worth decent points when killed
                150
            }
            crate::scene::enemy::EnemyType::BlobCore(data) => {
                // Blob core value scales with number of nodes
                500 + (data.node_count() as u64 * 25)
            }
            crate::scene::enemy::EnemyType::BlobNode(data) => {
                // Factory nodes worth more than base nodes
                if data.is_factory() {
                    250 // Factories are critical targets
                } else {
                    100 // Base nodes
                }
            }
        }
    }
}

impl Default for ScoringSystem {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ScoreMultiplierManager {
    system: ScoringSystem,
}

impl ScoreMultiplierManager {
    pub fn new() -> Self {
        Self {
            system: ScoringSystem::new(),
        }
    }

    pub fn system(&self) -> &ScoringSystem {
        &self.system
    }

    pub fn system_mut(&mut self) -> &mut ScoringSystem {
        &mut self.system
    }

    pub fn update(
        &mut self,
        dt: f32,
        player_pos: Vec3,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) {
        self.system.update(dt, player_pos, entity_manager);
    }

    pub fn get_render_data(&self) -> Vec<Primitive> {
        self.system.get_render_data()
    }

    pub fn drain_events(&mut self) -> Vec<EventType> {
        self.system.drain_events()
    }
}

impl Default for ScoreMultiplierManager {
    fn default() -> Self {
        Self::new()
    }
}
