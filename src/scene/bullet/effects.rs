use crate::{
    engine::{EntityId, Vec3},
    graphics::Color,
};

/// Trait for effects that trigger when a bullet hits a target
pub trait OnHitEffect: std::fmt::Debug + std::any::Any {
    /// Called when the bullet hits a target
    /// Returns chain lightning events if this is a chain lightning bullet
    fn on_hit(&self, hit_position: Vec3, target_id: Option<EntityId>) -> Vec<ChainLightningEvent>;

    /// Enable downcasting for trait objects
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Trait for effects that trigger when a bullet expires
pub trait OnExpireEffect: std::fmt::Debug {
    /// Called when the bullet expires/times out
    /// Returns explosion events to be processed by the event system
    fn on_expire(&self, expire_position: Vec3) -> Vec<crate::engine::dispatcher::ExplosionEvent>;
}

/// Chain lightning effect that jumps between enemies
#[derive(Debug)]
pub struct ChainLightningEffect {
    pub base_damage: f32,
    pub max_jumps: usize,
    pub jump_range: f32,
    pub damage_falloff: f32, // Multiplier for damage reduction per jump (e.g., 0.75)
}

impl ChainLightningEffect {
    pub fn new(base_damage: f32, max_jumps: usize, jump_range: f32, damage_falloff: f32) -> Self {
        Self {
            base_damage,
            max_jumps,
            jump_range,
            damage_falloff,
        }
    }
}

impl OnHitEffect for ChainLightningEffect {
    fn on_hit(&self, hit_position: Vec3, target_id: Option<EntityId>) -> Vec<ChainLightningEvent> {
        // Generate a chain lightning event
        vec![ChainLightningEvent {
            start_position: hit_position,
            base_damage: self.base_damage,
            max_jumps: self.max_jumps,
            jump_range: self.jump_range,
            damage_falloff: self.damage_falloff,
            excluded_target: target_id,
        }]
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Explosion effect that triggers when bullet expires
#[derive(Debug, Clone)]
pub struct ExplosionEffect {
    pub max_radius: f32,
    pub force_strength: f32,
    pub duration: f32,
    pub falloff_type: crate::scene::explosion::FalloffType,
    pub damage: f32,
    pub damage_radius: f32,
    pub explosion_color: Color,
    pub particle_color: Color,
    pub particle_count: u32,
}

impl ExplosionEffect {
    pub fn new(
        max_radius: f32,
        force_strength: f32,
        duration: f32,
        falloff_type: crate::scene::explosion::FalloffType,
        damage: f32,
        damage_radius: f32,
        explosion_color: Color,
        particle_color: Color,
        particle_count: u32,
    ) -> Self {
        Self {
            max_radius,
            force_strength,
            duration,
            falloff_type,
            damage,
            damage_radius,
            explosion_color,
            particle_color,
            particle_count,
        }
    }
}

impl OnExpireEffect for ExplosionEffect {
    fn on_expire(&self, expire_position: Vec3) -> Vec<crate::engine::dispatcher::ExplosionEvent> {
        // Generate a custom explosion event
        vec![crate::engine::dispatcher::ExplosionEvent::Custom {
            position: expire_position,
            max_radius: self.max_radius,
            force_strength: self.force_strength,
            duration: self.duration,
            falloff_type: self.falloff_type,
            damage: self.damage,
            damage_radius: self.damage_radius,
            explosion_color: self.explosion_color,
            particle_color: self.particle_color,
            particle_count: self.particle_count,
        }]
    }
}

/// Implosion effect that triggers when bullet hits (for implosion launcher)
#[derive(Debug)]
pub struct ImplosionOnHitEffect {
    pub explosion_effect: ExplosionEffect,
}

impl OnHitEffect for ImplosionOnHitEffect {
    fn on_hit(&self, _hit_position: Vec3, _target_id: Option<EntityId>) -> Vec<ChainLightningEvent> {
        // We don't return chain lightning events
        // The explosion will be triggered via downcasting in the manager
        Vec::new()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ImplosionOnHitEffect {
    /// Get the explosion event for this implosion (called by manager when processing on_hit)
    pub fn get_explosion_event(&self, position: Vec3) -> crate::engine::dispatcher::ExplosionEvent {
        crate::engine::dispatcher::ExplosionEvent::Custom {
            position,
            max_radius: self.explosion_effect.max_radius,
            force_strength: self.explosion_effect.force_strength,
            duration: self.explosion_effect.duration,
            falloff_type: self.explosion_effect.falloff_type,
            damage: self.explosion_effect.damage,
            damage_radius: self.explosion_effect.damage_radius,
            explosion_color: self.explosion_effect.explosion_color,
            particle_color: self.explosion_effect.particle_color,
            particle_count: self.explosion_effect.particle_count,
        }
    }
}

/// Event fired when chain lightning should occur
#[derive(Debug, Clone)]
pub struct ChainLightningEvent {
    pub start_position: Vec3,
    pub base_damage: f32,
    pub max_jumps: usize,
    pub jump_range: f32,
    pub damage_falloff: f32,
    pub excluded_target: Option<EntityId>, // Don't chain back to the original target
}

/// Container for complex projectile effects
pub struct ProjectileEffects {
    pub on_hit: Option<Vec<Box<dyn OnHitEffect>>>,
    pub on_expire: Option<Vec<Box<dyn OnExpireEffect>>>,
}

impl std::fmt::Debug for ProjectileEffects {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProjectileEffects")
            .field(
                "on_hit",
                &self.on_hit.as_ref().map(|v| format!("{} effects", v.len())),
            )
            .field(
                "on_expire",
                &self
                    .on_expire
                    .as_ref()
                    .map(|v| format!("{} effects", v.len())),
            )
            .finish()
    }
}

impl Clone for ProjectileEffects {
    fn clone(&self) -> Self {
        // Note: We can't clone trait objects, so we just create empty effects
        // This is a limitation for now - custom effects can't be cloned
        Self {
            on_hit: None,
            on_expire: None,
        }
    }
}

impl Default for ProjectileEffects {
    fn default() -> Self {
        Self {
            on_hit: None,
            on_expire: None,
        }
    }
}
