use crate::engine::entity::EntityId;
use crate::engine::math::Vec3;
use crate::graphics::color::Color;
use crate::scene::bullet::ChainLightningEvent;

pub struct Dispatcher {
    /// Events to be processed at the start of next frame
    execution_queue: Vec<EventType>,
    /// Events collected during current frame
    pending_events: Vec<EventType>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self {
            execution_queue: Vec::new(),
            pending_events: Vec::new(),
        }
    }

    /// Collect events directly from all managers - cleaner architecture
    pub fn collect_from_managers(
        &mut self,
        scheduler: &mut crate::engine::scheduler::Scheduler,
        collision_manager: &mut crate::engine::collision::CollisionManager,
    ) {
        // Collect events from each manager directly
        let player_events = scheduler.player_mut().drain_events();
        let enemy_events = scheduler.enemies_mut().drain_events();
        let bullet_events = scheduler.bullets_mut().drain_events();
        let large_body_events = scheduler.large_bodies_mut().drain_events();
        let collision_events = collision_manager.drain_events();
        let scoring_events = scheduler.scoring_mut().drain_events();

        // Add all events to pending queue
        self.pending_events.extend(player_events);
        self.pending_events.extend(enemy_events);
        self.pending_events.extend(bullet_events);
        self.pending_events.extend(large_body_events);
        self.pending_events.extend(collision_events);
        self.pending_events.extend(scoring_events);
    }

    /// Legacy method - collect events from pre-drained vectors (for backward compatibility)
    pub fn collect_events(
        &mut self,
        player_events: Vec<EventType>,
        enemy_events: Vec<EventType>,
        bullet_events: Vec<EventType>,
        collision_events: Vec<EventType>,
    ) {
        self.pending_events.extend(player_events);
        self.pending_events.extend(enemy_events);
        self.pending_events.extend(bullet_events);
        self.pending_events.extend(collision_events);
    }

    /// Prepare events for next frame (sort, batch, prioritize)
    pub fn prepare_next_frame(&mut self) {
        // Sort by priority if needed
        self.sort_events_by_priority();

        // Move pending to execution queue
        std::mem::swap(&mut self.execution_queue, &mut self.pending_events);
        self.pending_events.clear();
    }

    /// Process all queued events at start of frame
    pub fn process_events(
        &mut self,
        scheduler: &mut crate::engine::scheduler::Scheduler,
        graphics_events: &mut Vec<GraphicsEvent>,
    ) {
        // Collect events first to avoid borrow conflicts
        let events: Vec<EventType> = self.execution_queue.drain(..).collect();

        // Optimization: Batch events by type for more efficient processing
        let mut player_events = Vec::new();
        let mut enemy_events = Vec::new();
        let mut collision_events = Vec::new();
        let mut weapon_events = Vec::new();
        let mut explosion_events = Vec::new();
        let mut graphics_events_batch = Vec::new();
        let mut audio_events = Vec::new();
        let mut debug_events = Vec::new();
        let mut chain_lightning_events = Vec::new();
        let mut score_events = Vec::new();

        // Group events by type
        for event in events {
            match event {
                EventType::Player(e) => player_events.push(e),
                EventType::Enemy(e) => enemy_events.push(e),
                EventType::Collision(e) => collision_events.push(e),
                EventType::Weapon(e) => weapon_events.push(e),
                EventType::Explosion(e) => explosion_events.push(e),
                EventType::Graphics(e) => graphics_events_batch.push(e),
                EventType::Audio(e) => audio_events.push(e),
                EventType::Debug(e) => debug_events.push(e),
                EventType::ChainLightning(e) => chain_lightning_events.push(e),
                EventType::Score(e) => score_events.push(e),
            }
        }

        // Process batches - order matters for dependencies
        // 1. Process collision events first (they generate other events)
        // Process collision events with batching for performance
        let mut collision_explosion_events = Vec::new();
        Self::handle_collision_events_batch(
            collision_events,
            scheduler,
            graphics_events,
            &mut collision_explosion_events,
        );

        // Add collision-generated explosion events to the main batch
        explosion_events.extend(collision_explosion_events);

        // 2. Process player events (could affect weapon state)
        for event in player_events {
            scheduler.player_mut().handle_event(event);
        }

        // 3. Process enemy events
        for event in enemy_events {
            Self::handle_enemy_event(event, scheduler, &mut graphics_events_batch);
        }

        // 4. Process score events (after enemy events to ensure proper enemy death handling)
        Self::handle_score_events_batch(score_events, scheduler, &mut graphics_events_batch);

        // 5. Process weapon events
        for event in weapon_events {
            Self::handle_weapon_event(event, scheduler, &mut graphics_events_batch);
        }

        // 6. Process explosion events
        for event in explosion_events {
            Self::handle_explosion_event(event, scheduler, &mut graphics_events_batch);
        }

        // 7. Process chain lightning events (after collisions and explosions)
        Self::handle_chain_lightning_events_batch(
            chain_lightning_events,
            scheduler,
            &mut graphics_events_batch,
        );

        // 8. Process graphics events (visual effects)
        graphics_events.extend(graphics_events_batch);

        // 9. Process audio events (sound effects)
        Self::handle_audio_events_batch(audio_events);

        // 10. Process debug events
        Self::handle_debug_events_batch(debug_events);
    }

    fn sort_events_by_priority(&mut self) {
        // Sort so that certain events process first (e.g., damage before death)
        self.pending_events.sort_by_key(|event| {
            match event {
                EventType::Collision(_) => 0, // Process collisions first
                EventType::Weapon(_) => 1,
                EventType::Explosion(_) => 2, // Process explosions after weapons
                EventType::ChainLightning(_) => 3, // Process chain lightning after explosions
                EventType::Enemy(_) => 4,
                EventType::Score(_) => 5, // Process scoring after enemy events
                EventType::Player(_) => 6,
                EventType::Graphics(_) => 7, // Graphics last
                EventType::Audio(_) => 8,
                EventType::Debug(_) => 9,
            }
        });
    }

    fn handle_collision_events_batch(
        events: Vec<CollisionEvent>,
        scheduler: &mut crate::engine::scheduler::Scheduler,
        graphics_events: &mut Vec<GraphicsEvent>,
        explosion_events: &mut Vec<ExplosionEvent>,
    ) {
        // Batch bullet-bullet collisions for performance optimization
        let mut bullet_bullet_impacts = Vec::new();
        const MAX_BULLET_BULLET_PARTICLES_PER_FRAME: usize = 50; // Throttle to prevent spam

        for event in events {
            match event {
                CollisionEvent::BulletHitBullet { impact_point, .. } => {
                    // Collect impact points for batching
                    if bullet_bullet_impacts.len() < MAX_BULLET_BULLET_PARTICLES_PER_FRAME {
                        bullet_bullet_impacts.push(impact_point);
                    }
                    // Skip individual processing for bullet-bullet - handle below
                    continue;
                }
                _ => {
                    // Process other collision types individually
                    Self::handle_collision_event(
                        event,
                        scheduler,
                        graphics_events,
                        explosion_events,
                    );
                }
            }
        }

        // Process batched bullet-bullet collisions with single spawn request
        if !bullet_bullet_impacts.is_empty() {
            // Use first impact point as representative position for batch spawn
            let batch_position = bullet_bullet_impacts[0];
            let batch_count = bullet_bullet_impacts
                .len()
                .min(MAX_BULLET_BULLET_PARTICLES_PER_FRAME);

            // Single optimized particle spawn for all bullet-bullet collisions
            graphics_events.push(GraphicsEvent::SpawnParticles {
                position: batch_position,
                velocity: Vec3::new(0.0, 0.0, 0.0), // Sparks spread in all directions
                count: batch_count as u32,          // One particle per collision, capped
                lifetime: 3.0,                      // Quick flash effect
                color: Color::WHITE,
            });
        }
    }

    fn handle_collision_event(
        event: CollisionEvent,
        scheduler: &mut crate::engine::scheduler::Scheduler,
        graphics_events: &mut Vec<GraphicsEvent>,
        explosion_events: &mut Vec<ExplosionEvent>,
    ) {
        match event {
            CollisionEvent::BulletHitEnemy {
                bullet_id,
                enemy_id,
                damage,
                impact_point,
            } => {
                // Trigger MetaBullet OnHitEffects (like chain lightning) before removing bullet
                scheduler
                    .bullets_mut()
                    .register_hit(bullet_id, enemy_id, impact_point);

                // Use direct damage to avoid generating another damage event
                scheduler
                    .enemies_mut()
                    .damage_enemy_direct(enemy_id, damage, bullet_id);
                scheduler.bullets_mut().mark_bullet_for_removal(bullet_id);
            }
            CollisionEvent::EnemyHitPlayer {
                enemy_id: _,
                player_id: _,
                damage,
            } => {
                scheduler.player_mut().player_mut().take_damage(damage);
                // Reset score multiplier when player gets hit
                scheduler.scoring_mut().system_mut().reset_multiplier();
                // Could queue screen shake event
                // or literally anything else for a player taking damage
            }
            CollisionEvent::EnemyHitLargeBody {
                enemy_id,
                large_body_id,
                impact_point,
            } => {
                // Try to kill the enemy (large body collisions are fatal)
                let enemy_found =
                    scheduler
                        .enemies_mut()
                        .damage_enemy_direct(enemy_id, 9999.0, large_body_id);

                if !enemy_found {
                    // Enemy already dead - but we should still reward the player for the collision!
                    // Spawn collectible directly since the normal enemy death logic already ran
                    let entity_manager_ptr = scheduler.entity_manager_mut() as *mut _;
                    unsafe {
                        scheduler
                            .scoring_mut()
                            .system_mut()
                            .spawn_multiplier(impact_point, &mut *entity_manager_ptr);
                    }
                }

                // Show particles regardless - the collision happened
                {
                    // Calculate tangential velocity for orbital motion around the large body
                    let tangential_velocity = if let Some(large_body) =
                        scheduler.large_bodies().get_body(large_body_id)
                    {
                        let displacement = large_body.position() - impact_point;
                        let distance = displacement.magnitude();

                        if distance > 0.1 {
                            // Create tangential direction (perpendicular to radial)
                            let radial_dir = displacement.normalize();
                            let up_vector = Vec3::new(0.0, 1.0, 0.0);
                            let mut tangent_dir = radial_dir.cross(&up_vector);

                            // Handle edge case where radial is parallel to up vector
                            if tangent_dir.magnitude() < 0.1 {
                                let alternate_ref = Vec3::new(1.0, 0.0, 0.0);
                                tangent_dir = radial_dir.cross(&alternate_ref);
                            }
                            let tangent_dir = tangent_dir.normalize();

                            // Scale tangential velocity based on large body's angular velocity and distance
                            let tangential_speed = large_body.angular_velocity() * distance * 0.1;
                            tangent_dir * tangential_speed * 0.1
                        } else {
                            Vec3::new(0.0, 0.0, 0.0)
                        }
                    } else {
                        Vec3::new(0.0, 0.0, 0.0)
                    };

                    // Spawn explosion particles with tangential velocity for orbital motion
                    graphics_events.push(GraphicsEvent::SpawnParticles {
                        position: impact_point,
                        velocity: tangential_velocity, // Particles inherit tangential motion
                        count: 100,                    // Increased from 100 for dramatic effect
                        lifetime: 60.0,                // Longer lifetime for visibility
                        color: Color::CYAN,
                    });
                }
            }
            CollisionEvent::BulletHitLargeBody {
                bullet_id: _,
                large_body_id: _,
                impact_point,
            } => {
                // Spawn smaller impact particles for bullet hits
                graphics_events.push(GraphicsEvent::SpawnParticles {
                    position: impact_point,
                    velocity: Vec3::new(0.0, 0.0, 0.0), // Impact sparks spread outward
                    count: 30,                          // Increased from 10 for better visibility
                    lifetime: 60.0,                     // Shorter lifetime for quick effect
                    color: Color::ORANGE,
                });
            }
            CollisionEvent::BulletHitBullet {
                bullet_a_id: _,
                bullet_b_id: _,
                impact_point,
            } => {
                // NOTE: Bullet-bullet collisions are now handled in batch in handle_collision_events_batch()
                // This individual handler should not be reached for bullet-bullet collisions
                // But keeping for safety in case batch handler is bypassed
                graphics_events.push(GraphicsEvent::SpawnParticles {
                    position: impact_point,
                    velocity: Vec3::new(0.0, 0.0, 0.0), // Sparks spread in all directions
                    count: 1,
                    lifetime: 3.0, // Quick flash effect
                    color: Color::WHITE,
                });
            }
            CollisionEvent::LargeBodyHitLargeBody {
                large_body_a_id: _,
                large_body_b_id: _,
                impact_point,
            } => {
                // PARTICLES CANNOT SPAWN HERE DUE TO FRAME SPIKING
                // SHUT UP LOL
                graphics_events.push(GraphicsEvent::SpawnParticles {
                    position: impact_point,
                    velocity: Vec3::new(0.0, 0.0, 0.0), // Sparks spread in all directions
                    count: 20,
                    lifetime: 3.0, // Quick flash effect
                    color: Color::WHITE,
                });

                // Queue a shockwave explosion event for physics effects
                explosion_events.push(ExplosionEvent::Shockwave {
                    position: impact_point,
                });
            }
            CollisionEvent::PlayerHitCollectible {
                player_id: _,
                collectible_id,
            } => {
                // Directly collect the multiplier through the scoring system
                // Find the multiplier value first
                let mut multiplier_value = 0.1; // Default value
                for multiplier in scheduler.scoring().system().multipliers() {
                    if multiplier.entity_id() == collectible_id {
                        multiplier_value = multiplier.multiplier_value();
                        break;
                    }
                }

                // Collect the multiplier (this removes it and updates the score multiplier)
                let collected = scheduler
                    .scoring_mut()
                    .system_mut()
                    .collect_multiplier(multiplier_value, collectible_id);

                if collected {
                    scheduler
                        .entity_manager_mut()
                        .destroy_entity(collectible_id);
                }
            }
        }
    }

    fn handle_weapon_event(
        event: WeaponEvent,
        _scheduler: &mut crate::engine::scheduler::Scheduler,
        _graphics_events: &mut Vec<GraphicsEvent>,
    ) {
        match event {
            WeaponEvent::Fired {
                weapon_type: _,
                position: _,
                direction: _,
                projectile_count: _,
            } => {
                // Could spawn muzzle flash, play sound, etc.
            }
        }
    }

    fn handle_explosion_event(
        event: ExplosionEvent,
        scheduler: &mut crate::engine::scheduler::Scheduler,
        graphics_events: &mut Vec<GraphicsEvent>,
    ) {
        match event {
            ExplosionEvent::SolarWind { position } => {
                scheduler.explosions_mut().spawn_solar_wind(position);
            }
            ExplosionEvent::AntiWind { position } => {
                scheduler.explosions_mut().spawn_anti_wind(position);
            }
            ExplosionEvent::Shockwave { position } => {
                scheduler.explosions_mut().spawn_simple_shockwave(position);
            }
            ExplosionEvent::Custom {
                position,
                max_radius,
                force_strength,
                duration,
                falloff_type,
                damage,
                damage_radius,
                explosion_color,
                particle_color,
                particle_count,
            } => {
                scheduler.explosions_mut().spawn_explosion(
                    position,
                    max_radius,
                    force_strength,
                    duration,
                    falloff_type,
                    explosion_color,
                    particle_color,
                    particle_count,
                );

                // Apply damage to enemies within damage radius
                if damage > 0.0 && damage_radius > 0.0 {
                    let damage_events = scheduler.explosions().calculate_explosion_damage(
                        position,
                        damage,
                        damage_radius,
                        scheduler.enemies().enemies(),
                    );

                    for (enemy_id, actual_damage) in damage_events {
                        scheduler
                            .enemies_mut()
                            .damage_enemy(enemy_id, actual_damage);
                    }
                }
                if particle_count > 0 {
                    graphics_events.push(GraphicsEvent::SpawnParticles {
                        position,
                        velocity: Vec3::new(0.0, 0.0, 0.0),
                        count: particle_count,
                        lifetime: 3.0,
                        color: particle_color,
                    });
                }
            }
        }
    }

    fn handle_enemy_event(
        event: EnemyEvent,
        scheduler: &mut crate::engine::scheduler::Scheduler,
        _graphics_events: &mut Vec<GraphicsEvent>,
    ) {
        // Delegate to the enemy manager's handle_event method which contains the score event generation logic
        scheduler.enemies_mut().handle_event(event);
    }

    fn handle_audio_event(event: AudioEvent) {
        match event {
            AudioEvent::PlaySound {
                sound_id,
                position,
                volume,
            } => {
                // Audio system not implemented yet, just log for now
                if let Some(pos) = position {
                    println!(
                        "Playing sound {} at {:?} with volume {}",
                        sound_id, pos, volume
                    );
                } else {
                    println!("Playing global sound {} with volume {}", sound_id, volume);
                }
            }
        }
    }

    fn handle_audio_events_batch(events: Vec<AudioEvent>) {
        if events.is_empty() {
            return;
        }

        // Batch process audio events - could optimize by grouping by sound type, position, etc.
        for event in events {
            Self::handle_audio_event(event);
        }
    }

    fn handle_debug_events_batch(events: Vec<DebugEvent>) {
        if events.is_empty() {
            return;
        }

        // Batch debug events - could group similar log messages, suppress duplicates, etc.
        for event in events {
            match event {
                DebugEvent::Log(message) => println!("[DEBUG] {}", message),
            }
        }
    }

    fn handle_debug_event(event: DebugEvent) {
        match event {
            DebugEvent::Log(message) => println!("[DEBUG] {}", message),
        }
    }

    fn handle_chain_lightning_events_batch(
        events: Vec<ChainLightningEvent>,
        scheduler: &mut crate::engine::scheduler::Scheduler,
        graphics_events: &mut Vec<GraphicsEvent>,
    ) {
        for event in events {
            Self::handle_chain_lightning_event(event, scheduler, graphics_events);
        }
    }

    fn handle_chain_lightning_event(
        event: ChainLightningEvent,
        scheduler: &mut crate::engine::scheduler::Scheduler,
        graphics_events: &mut Vec<GraphicsEvent>,
    ) {
        // Optimized chain lightning algorithm with early termination and reduced allocations
        let mut current_position = event.start_position;
        let mut current_damage = event.base_damage;
        let mut visited_targets = std::collections::HashSet::new(); // Use HashSet for O(1) lookups instead of Vec

        // Add the excluded target to visited list if specified
        if let Some(excluded) = event.excluded_target {
            visited_targets.insert(excluded);
        }

        let mut lightning_segments = Vec::with_capacity(event.max_jumps); // Pre-allocate capacity

        // Get enemies once and cache positions to avoid repeated method calls
        let enemies = scheduler.enemies().enemies();
        let enemy_data: Vec<_> = enemies
            .iter()
            .map(|enemy| (enemy.entity_id(), enemy.position()))
            .collect();

        for _jump in 0..event.max_jumps {
            let mut nearest_enemy = None;
            let mut nearest_distance_sq = event.jump_range * event.jump_range; // Use squared distance to avoid sqrt

            // Find nearest enemy using cached data
            for &(enemy_id, enemy_pos) in &enemy_data {
                // Skip if already visited (O(1) lookup with HashSet)
                if visited_targets.contains(&enemy_id) {
                    continue;
                }

                // Use squared distance to avoid expensive sqrt calls
                let diff = enemy_pos - current_position;
                let distance_sq = diff.x * diff.x + diff.y * diff.y + diff.z * diff.z;

                if distance_sq < nearest_distance_sq {
                    nearest_distance_sq = distance_sq;
                    nearest_enemy = Some((enemy_id, enemy_pos));
                }
            }

            // If no valid target found, stop chaining
            if let Some((target_id, target_pos)) = nearest_enemy {
                // Apply damage to the target
                scheduler
                    .enemies_mut()
                    .damage_enemy_direct(target_id, current_damage, target_id);

                // Add to visited set (O(1) insertion)
                visited_targets.insert(target_id);

                // Store lightning segment for visual effects
                lightning_segments.push((current_position, target_pos));

                // Update position and damage for next jump
                current_position = target_pos;
                current_damage *= event.damage_falloff;

                // Early termination if damage becomes negligible
                if current_damage < 1.0 {
                    break;
                }
            } else {
                // No more targets in range
                break;
            }
        }

        // Generate visual lightning effects for all segments using actual lightning bolts
        for (start, end) in lightning_segments {
            // Spawn actual lightning bolt between chain targets
            graphics_events.push(GraphicsEvent::SpawnLightning { start, end });

            // Add particle effects at both ends of each lightning segment for extra impact
            graphics_events.push(GraphicsEvent::SpawnParticles {
                position: start,
                velocity: (end - start).normalize() * 3.0,
                count: 30,
                lifetime: 6.0,
                color: Color::new(0.3, 0.8, 1.0, 1.0), // Electric blue-cyan
            });

            graphics_events.push(GraphicsEvent::SpawnParticles {
                position: end,
                velocity: Vec3::new(0.0, 2.0, 0.0) + (start - end).normalize() * 1.5,
                count: 20,
                lifetime: 6.0,
                color: Color::new(1.0, 1.0, 0.9, 1.0), // Bright electric white
            });

            // Add impact particles at the target
            graphics_events.push(GraphicsEvent::SpawnParticles {
                position: end,
                velocity: Vec3::zeros(),
                count: 15,
                lifetime: 6.0,
                color: Color::new(0.8, 0.9, 1.0, 1.0), // Electric glow
            });
        }
    }

    fn handle_score_events_batch(
        events: Vec<ScoreEvent>,
        scheduler: &mut crate::engine::scheduler::Scheduler,
        _graphics_events: &mut Vec<GraphicsEvent>,
    ) {
        for event in events {
            Self::handle_score_event(event, scheduler);
        }
    }

    fn handle_score_event(event: ScoreEvent, scheduler: &mut crate::engine::scheduler::Scheduler) {
        match event {
            ScoreEvent::EnemyKilled {
                enemy_id: _,
                enemy_type,
                position,
            } => {
                // Add points based on enemy type
                let base_points = crate::scene::ScoringSystem::get_enemy_base_points(&enemy_type);
                scheduler.scoring_mut().system_mut().add_score(base_points);

                // Spawn multiplier collectible at enemy death location - split borrow to avoid double mutable borrow
                let entity_manager_ptr = scheduler.entity_manager_mut() as *mut _;
                unsafe {
                    scheduler
                        .scoring_mut()
                        .system_mut()
                        .spawn_multiplier(position, &mut *entity_manager_ptr);
                }
            }
            ScoreEvent::MultiplierCollected {
                multiplier_id,
                multiplier_value,
            } => {
                // Collect the multiplier
                scheduler
                    .scoring_mut()
                    .system_mut()
                    .collect_multiplier(multiplier_value, multiplier_id);
            }
        }
    }
}

/// Main event type hierarchy
#[derive(Clone, Debug)]
pub enum EventType {
    Player(PlayerEvent),
    Enemy(EnemyEvent),
    Collision(CollisionEvent),
    Weapon(WeaponEvent),
    Explosion(ExplosionEvent),
    Graphics(GraphicsEvent),
    Audio(AudioEvent),
    Debug(DebugEvent),
    ChainLightning(ChainLightningEvent),
    Score(ScoreEvent),
}

/// Player-specific events
#[derive(Clone, Debug)]
pub enum PlayerEvent {
    TakeDamage { amount: f32, source: EntityId },
    Die,
    Heal { amount: f32 },
    WeaponSwitch { weapon_index: usize },
}

/// Enemy-specific events
#[derive(Clone, Debug)]
pub enum EnemyEvent {
    TakeDamage {
        enemy_id: EntityId,
        amount: f32,
        source: EntityId,
    },
    Die {
        enemy_id: EntityId,
    },
    Spawn {
        position: Vec3,
        enemy_type: u32,
    },
    Split {
        parent_id: EntityId,
        position: Vec3,
        current_generation: u8,
        max_generation: u8,
    },
}

/// Collision events that affect multiple systems
#[derive(Clone, Debug)]
pub enum CollisionEvent {
    BulletHitEnemy {
        bullet_id: EntityId,
        enemy_id: EntityId,
        damage: f32,
        impact_point: Vec3,
    },
    EnemyHitPlayer {
        enemy_id: EntityId,
        player_id: EntityId,
        damage: f32,
    },
    EnemyHitLargeBody {
        enemy_id: EntityId,
        large_body_id: EntityId,
        impact_point: Vec3,
    },
    BulletHitLargeBody {
        bullet_id: EntityId,
        large_body_id: EntityId,
        impact_point: Vec3,
    },
    BulletHitBullet {
        bullet_a_id: EntityId,
        bullet_b_id: EntityId,
        impact_point: Vec3,
    },
    LargeBodyHitLargeBody {
        large_body_a_id: EntityId,
        large_body_b_id: EntityId,
        impact_point: Vec3,
    },
    PlayerHitCollectible {
        player_id: EntityId,
        collectible_id: EntityId,
    },
}

/// Weapon/combat events
#[derive(Clone, Debug)]
pub enum WeaponEvent {
    Fired {
        weapon_type: crate::scene::weapon::WeaponType,
        position: Vec3,
        direction: Vec3,
        projectile_count: u32,
    },
}

/// Explosion events
#[derive(Clone, Debug)]
pub enum ExplosionEvent {
    SolarWind {
        position: Vec3,
    },
    AntiWind {
        position: Vec3,
    },
    Shockwave {
        position: Vec3,
    },
    Custom {
        position: Vec3,
        max_radius: f32,
        force_strength: f32,
        duration: f32,
        falloff_type: crate::scene::explosion::FalloffType,
        damage: f32,
        damage_radius: f32,
        explosion_color: Color,
        particle_color: Color,
        particle_count: u32,
    },
}

/// Graphics/rendering events
#[derive(Clone, Debug)]
pub enum GraphicsEvent {
    DrawLine {
        start: Vec3,
        end: Vec3,
        color: Color,
        duration: f32, // 0 = this frame only
    },
    SpawnParticles {
        position: Vec3,
        velocity: Vec3,
        count: u32,
        lifetime: f32,
        color: Color,
    },
    ScreenShake {
        intensity: f32,
        duration: f32,
    },
    SpawnLightning {
        start: Vec3,
        end: Vec3,
    },
}

/// Audio events
#[derive(Clone, Debug)]
pub enum AudioEvent {
    PlaySound {
        sound_id: u32,
        position: Option<Vec3>, // None = non-spatial
        volume: f32,
    },
}

/// Debug/development events
#[derive(Clone, Debug)]
pub enum DebugEvent {
    Log(String),
}

/// Scoring events
#[derive(Clone, Debug)]
pub enum ScoreEvent {
    EnemyKilled {
        enemy_id: EntityId,
        enemy_type: crate::scene::enemy::EnemyType,
        position: Vec3,
    },
    MultiplierCollected {
        multiplier_id: EntityId,
        multiplier_value: f32,
    },
}
