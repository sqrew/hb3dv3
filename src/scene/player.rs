use crate::engine::dispatcher::EventType;
use crate::engine::entity::{EntityId, EntityType};
use crate::engine::{CollisionMask, Vec3};
use crate::graphics::{Color, Primitive, PrimitiveType};
use crate::input::{Action, InputManager};
use crate::scene::{GravityAffected, WeaponManager, WeaponSpawnRequest};

/// Request to spawn a panic explosion
pub struct PanicExplosionRequest {
    pub position: Vec3,
}

// Dash constants
const DASH_DURATION: f32 = 0.15; // 150ms dash duration
const DASH_COOLDOWN: f32 = 1.0; // 1 second cooldown
const DASH_SPEED_MULTIPLIER: f32 = 6.0; // 4x normal speed during dash
const IFRAMES_DURATION: f32 = 0.5;
const IFRAMES_COOLDOWN: f32 = 3.0;
const DAMAGE_FLASH_DURATION: f32 = 0.5; // How long to show red damage flash

// Panic explosion constants
const PANIC_EXPLOSION_COOLDOWN: f32 = 5.0; // 5 second cooldown

pub struct Player {
    entity_id: EntityId,
    pos: Vec3,
    vel: Vec3,
    health: f32,
    speed: f32,
    collision_radius: f32,
    collision_mask: CollisionMask,
    weapon_manager: WeaponManager,
    mass: f32,
    applied_force: Vec3,

    // Dash system
    is_dashing: bool,
    dash_timer: f32,
    dash_cooldown_timer: f32,

    // I-frame system
    iframe_timer: f32,
    iframe_cooldown_timer: f32,

    // Damage flash system
    damage_flash_timer: f32,

    // Panic explosion system
    panic_explosion_cooldown_timer: f32,
}

impl Player {
    pub fn new(entity_id: EntityId) -> Self {
        Player {
            entity_id,
            pos: Vec3::new(0.0, 0.0, 0.0),
            vel: Vec3::zeros(),
            health: 100.0,
            speed: 50.0,
            collision_radius: 0.8,
            collision_mask: CollisionMask::from(EntityType::Player),
            weapon_manager: WeaponManager::new(),
            mass: 75.0, // Player mass in kg
            applied_force: Vec3::zeros(),

            // Dash system initialization
            is_dashing: false,
            dash_timer: 0.0,
            dash_cooldown_timer: 0.0,

            // I-frame system initialization
            iframe_timer: 0.0,
            iframe_cooldown_timer: 0.0,

            // Damage flash system initialization
            damage_flash_timer: 0.0,

            // Panic explosion system initialization
            panic_explosion_cooldown_timer: 0.0,
        }
    }

    pub fn update(
        &mut self,
        delta_time: f32,
        input: &InputManager,
        camera_forward: Vec3,
        camera_right: Vec3,
        camera_up: Vec3,
    ) -> (Option<WeaponSpawnRequest>, Option<PanicExplosionRequest>) {
        // Update weapon manager
        self.weapon_manager.update(delta_time, input);

        // Update dash cooldown timer
        if self.dash_cooldown_timer > 0.0 {
            self.dash_cooldown_timer -= delta_time;
        }

        // Update i-frame timers
        if self.iframe_timer > 0.0 {
            self.iframe_timer -= delta_time;
        }
        if self.iframe_cooldown_timer > 0.0 {
            self.iframe_cooldown_timer -= delta_time;
        }

        // Update damage flash timer
        if self.damage_flash_timer > 0.0 {
            self.damage_flash_timer -= delta_time;
        }

        // Update panic explosion cooldown timer
        if self.panic_explosion_cooldown_timer > 0.0 {
            self.panic_explosion_cooldown_timer -= delta_time;
        }

        // Handle panic explosion input
        let mut panic_explosion_request = None;
        let panic_pressed = input.is_action_just_pressed(Action::PanicExplosion);
        if panic_pressed && self.panic_explosion_cooldown_timer <= 0.0 {
            panic_explosion_request = Some(PanicExplosionRequest {
                position: self.pos,
            });
            self.panic_explosion_cooldown_timer = PANIC_EXPLOSION_COOLDOWN;
            println!("💥 Panic explosion triggered!");
        }

        // Handle movement input with correct mapping
        let (input_x, input_z) = input.get_action_vector2(Action::MoveForward);
        let input_y =
            input.get_action_value(Action::MoveUp) - input.get_action_value(Action::MoveDown); // Up is positive Y

        // Calculate full 3D camera-relative movement
        let forward_movement = camera_forward * input_z; // Forward follows camera direction including pitch
        let right_movement = camera_right * input_x; // Right strafe relative to camera
        let up_movement = camera_up * input_y; // Up/down relative to camera up vector

        // Combine all movement vectors
        let movement_direction = forward_movement + right_movement + up_movement;

        // Handle dash input
        let dash_pressed = input.is_action_just_pressed(Action::Dash);
        if dash_pressed && !self.is_dashing && self.dash_cooldown_timer <= 0.0 {
            self.start_dash();
        }

        // Update dash timer and handle dash movement
        if self.is_dashing {
            self.dash_timer += delta_time;

            if self.dash_timer >= DASH_DURATION {
                // End dash
                self.is_dashing = false;
                self.dash_timer = 0.0;
                self.dash_cooldown_timer = DASH_COOLDOWN;
            }
        }

        // Calculate velocity
        let final_velocity = if self.is_dashing {
            // During dash: use current input direction at high speed, ignore gravity
            movement_direction * self.speed * DASH_SPEED_MULTIPLIER
        } else {
            // Normal movement: combine input with gravity
            let input_velocity = movement_direction * self.speed;
            let gravity_acceleration = self.applied_force / self.mass;
            input_velocity + gravity_acceleration * delta_time
        };

        self.vel = final_velocity;

        // Update position
        self.pos += self.vel * delta_time;

        // Reset applied force for next frame
        self.applied_force = Vec3::zeros();

        // Handle shooting input
        let fire_pressed = input.get_action_value(Action::Fire) > 0.0;

        let weapon_request = if fire_pressed {
            // Try to fire weapon in camera forward direction
            self.weapon_manager.try_fire(self.pos, camera_forward)
        } else {
            None
        };

        (weapon_request, panic_explosion_request)
    }

    fn start_dash(&mut self) {
        // Activate dash state and trigger i-frames
        self.is_dashing = true;
        self.dash_timer = 0.0;
        self.trigger_iframes();
    }

    pub fn position(&self) -> Vec3 {
        self.pos
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    /// Returns true if player has invincibility frames
    pub fn is_invincible(&self) -> bool {
        self.iframe_timer > 0.0
    }

    /// Returns true if player is currently dashing
    pub fn is_dashing(&self) -> bool {
        self.is_dashing
    }

    /// Returns dash cooldown progress (0.0 = ready, 1.0 = just used)
    pub fn dash_cooldown_progress(&self) -> f32 {
        if self.dash_cooldown_timer <= 0.0 {
            0.0
        } else {
            self.dash_cooldown_timer / DASH_COOLDOWN
        }
    }

    /// Returns i-frame progress (0.0 = no i-frames, 1.0 = full i-frames)
    pub fn iframe_progress(&self) -> f32 {
        if self.iframe_timer <= 0.0 {
            0.0
        } else {
            self.iframe_timer / IFRAMES_DURATION
        }
    }

    /// Returns i-frame cooldown progress (0.0 = ready, 1.0 = just used)
    pub fn iframe_cooldown_progress(&self) -> f32 {
        if self.iframe_cooldown_timer <= 0.0 {
            0.0
        } else {
            self.iframe_cooldown_timer / IFRAMES_COOLDOWN
        }
    }

    /// Returns panic explosion cooldown progress (0.0 = ready, 1.0 = just used)
    pub fn panic_explosion_cooldown_progress(&self) -> f32 {
        if self.panic_explosion_cooldown_timer <= 0.0 {
            0.0
        } else {
            self.panic_explosion_cooldown_timer / PANIC_EXPLOSION_COOLDOWN
        }
    }

    pub fn collision_radius(&self) -> f32 {
        self.collision_radius
    }

    pub fn collision_mask(&self) -> CollisionMask {
        self.collision_mask
    }

    pub fn take_damage(&mut self, damage: f32) {
        self.health -= damage;
        // Start damage flash timer
        self.damage_flash_timer = DAMAGE_FLASH_DURATION;
    }

    pub fn heal(&mut self, amount: f32) {
        self.health += amount;
        self.health = self.health.min(100.0); // Cap at max health
    }

    /// Trigger invincibility frames
    fn trigger_iframes(&mut self) {
        if self.iframe_cooldown_timer <= 0.0 {
            self.iframe_timer = IFRAMES_DURATION;
            self.iframe_cooldown_timer = IFRAMES_COOLDOWN;
        }
    }
}

impl GravityAffected for Player {
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

pub struct PlayerManager {
    player: Player,
    event_queue: Vec<EventType>,
}

impl PlayerManager {
    pub fn new(entity_id: EntityId) -> Self {
        Self {
            player: Player::new(entity_id),
            event_queue: Vec::new(),
        }
    }

    pub fn update(
        &mut self,
        delta_time: f32,
        input: &InputManager,
        camera_forward: Vec3,
        camera_right: Vec3,
        camera_up: Vec3,
    ) -> (Option<WeaponSpawnRequest>, Option<PanicExplosionRequest>) {
        let (weapon_request, panic_request) =
            self.player
                .update(delta_time, input, camera_forward, camera_right, camera_up);

        // Collect weapon events from player
        let weapon_events = self.player.weapon_manager.drain_events();
        self.event_queue.extend(weapon_events);

        (weapon_request, panic_request)
    }

    /// Get and clear player events (including weapon events)
    pub fn drain_events(&mut self) -> Vec<EventType> {
        self.event_queue.drain(..).collect()
    }

    pub fn get_render_data(&self) -> Vec<Primitive> {
        let color = if self.player.damage_flash_timer > 0.0 {
            Color::RED
        } else if self.player.is_invincible() {
            Color::CYAN
        } else {
            Color::WHITE
        };

        vec![Primitive::new(
            PrimitiveType::Cube,
            self.player.pos,
            color,
        )]
    }

    pub fn player(&self) -> &Player {
        &self.player
    }

    pub fn player_mut(&mut self) -> &mut Player {
        &mut self.player
    }

    /// Handle player events from dispatcher
    pub fn handle_event(&mut self, event: crate::engine::dispatcher::PlayerEvent) {
        use crate::engine::dispatcher::PlayerEvent;
        match event {
            PlayerEvent::TakeDamage { amount, source: _ } => {
                // Check for i-frames before applying damage
                if !self.player.is_invincible() {
                    // Apply damage (event already generated by collision system)
                    self.player.take_damage(amount);

                    // Check if player died and generate death event
                    if self.player.health <= 0.0 {
                        self.event_queue.push(EventType::Player(PlayerEvent::Die));
                    }
                } else {
                    // Player has i-frames, ignore damage
                    println!("🛡️ Damage blocked by i-frames!");
                }
            }
            PlayerEvent::Die => {
                // Handle player death
                println!("💀 Player died!");
            }
            PlayerEvent::Heal { amount } => {
                // Apply healing directly (don't re-generate heal event)
                self.player.heal(amount);
            }
            PlayerEvent::WeaponSwitch { weapon_index } => {
                // Weapon switching is handled by WeaponManager directly via input
                // This event is just for notification/logging purposes
                println!(
                    "🔄 Weapon switch event processed for index {}",
                    weapon_index
                );
            }
        }
    }

    /// Returns true if player has invincibility frames
    pub fn is_invincible(&self) -> bool {
        self.player.is_invincible()
    }

    /// Returns true if player is currently dashing
    pub fn is_dashing(&self) -> bool {
        self.player.is_dashing()
    }

    /// Returns dash cooldown progress (0.0 = ready, 1.0 = just used)
    pub fn dash_cooldown_progress(&self) -> f32 {
        self.player.dash_cooldown_progress()
    }

    /// Returns i-frame progress (0.0 = no i-frames, 1.0 = full i-frames)
    pub fn iframe_progress(&self) -> f32 {
        self.player.iframe_progress()
    }

    /// Returns i-frame cooldown progress (0.0 = ready, 1.0 = just used)
    pub fn iframe_cooldown_progress(&self) -> f32 {
        self.player.iframe_cooldown_progress()
    }

    /// Returns panic explosion cooldown progress (0.0 = ready, 1.0 = just used)
    pub fn panic_explosion_cooldown_progress(&self) -> f32 {
        self.player.panic_explosion_cooldown_progress()
    }
}
