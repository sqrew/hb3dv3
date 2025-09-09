use crate::engine::{Vec3, CollisionMask};
use crate::engine::entity::{EntityId, EntityType};
use crate::input::{InputManager, Action};
use crate::graphics::{Primitive, PrimitiveType, Color};
use crate::scene::{WeaponManager, BulletSpawnRequest};

pub struct Player {
    entity_id: EntityId,
    pos: Vec3,
    vel: Vec3,
    health: f32,
    speed: f32,
    collision_radius: f32,
    collision_mask: CollisionMask,
    weapon_manager: WeaponManager,
}

impl Player {
    pub fn new(entity_id: EntityId) -> Self {
        Player {
            entity_id,
            pos: Vec3::new(0.0, 0.0, 0.0),
            vel: Vec3::zeros(),
            health: 100.0,
            speed: 10.0,
            collision_radius: 0.8,
            collision_mask: CollisionMask::from(EntityType::Player),
            weapon_manager: WeaponManager::new(),
        }
    }
    
    pub fn update(&mut self, delta_time: f32, input: &InputManager, camera_forward: Vec3, camera_right: Vec3, camera_up: Vec3) -> Option<Vec<BulletSpawnRequest>> {
        // Update weapon manager
        self.weapon_manager.update(delta_time, input);
        
        // Handle movement input with correct mapping
        let (input_x, input_z) = input.get_action_vector2(Action::MoveForward);
        let input_y = input.get_action_value(Action::MoveUp) - input.get_action_value(Action::MoveDown); // Up is positive Y
        
        // Calculate full 3D camera-relative movement
        let forward_movement = camera_forward * input_z; // Forward follows camera direction including pitch
        let right_movement = camera_right * input_x;     // Right strafe relative to camera
        let up_movement = camera_up * input_y;           // Up/down relative to camera up vector
        
        // Combine all movement vectors
        let movement_direction = forward_movement + right_movement + up_movement;
        
        // Update velocity based on camera-relative input
        self.vel = movement_direction * self.speed;
        
        // Update position
        self.pos += self.vel * delta_time;
        
        // Clamp to reasonable bounds (simple boundary)
        self.pos.x = self.pos.x.clamp(-50.0, 50.0);
        self.pos.y = self.pos.y.clamp(-30.0, 30.0);
        self.pos.z = self.pos.z.clamp(-50.0, 50.0);
        
        // Handle shooting input
        let fire_pressed = input.get_action_value(Action::Fire) > 0.0;
        
        if fire_pressed {
            // Try to fire weapon in camera forward direction
            self.weapon_manager.try_fire(self.pos, camera_forward)
        } else {
            None
        }
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
}

pub struct PlayerManager {
    player: Player,
}

impl PlayerManager {
    pub fn new(entity_id: EntityId) -> Self {
        Self {
            player: Player::new(entity_id),
        }
    }
    
    pub fn update(&mut self, delta_time: f32, input: &InputManager, camera_forward: Vec3, camera_right: Vec3, camera_up: Vec3) -> Option<Vec<BulletSpawnRequest>> {
        self.player.update(delta_time, input, camera_forward, camera_right, camera_up)
    }
    
    pub fn get_render_data(&self) -> Vec<Primitive> {
        vec![
            Primitive::new(
                PrimitiveType::Cube,
                self.player.pos,
                Color::new(0.0, 0.8, 0.2, 1.0) // Green player
            )
        ]
    }
    
    pub fn player(&self) -> &Player {
        &self.player
    }
}
