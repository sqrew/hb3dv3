use crate::engine::entity::EntityId;
use crate::engine::math::Vec3;
use crate::graphics::color::Color;

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

    /// Collect events from all managers at end of frame
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
        let mut graphics_events_batch = Vec::new();
        let mut audio_events = Vec::new();
        let mut debug_events = Vec::new();

        // Group events by type
        for event in events {
            match event {
                EventType::Player(e) => player_events.push(e),
                EventType::Enemy(e) => enemy_events.push(e),
                EventType::Collision(e) => collision_events.push(e),
                EventType::Weapon(e) => weapon_events.push(e),
                EventType::Graphics(e) => graphics_events_batch.push(e),
                EventType::Audio(e) => audio_events.push(e),
                EventType::Debug(e) => debug_events.push(e),
            }
        }

        // Process batches - order matters for dependencies
        // 1. Process collision events first (they generate other events)
        Self::handle_collision_events_batch(collision_events, scheduler, graphics_events);

        // 2. Process player events (could affect weapon state)
        for event in player_events {
            scheduler.player_mut().handle_event(event);
        }

        // 3. Process enemy events
        for event in enemy_events {
            scheduler.enemies_mut().handle_event(event);
        }

        // 4. Process weapon events
        for event in weapon_events {
            Self::handle_weapon_event(event, scheduler);
        }

        // 5. Process graphics events (visual effects)
        graphics_events.extend(graphics_events_batch);

        // 6. Process audio events (sound effects)
        Self::handle_audio_events_batch(audio_events);

        // 7. Process debug events
        Self::handle_debug_events_batch(debug_events);
    }

    fn sort_events_by_priority(&mut self) {
        // Sort so that certain events process first (e.g., damage before death)
        self.pending_events.sort_by_key(|event| {
            match event {
                EventType::Collision(_) => 0, // Process collisions first
                EventType::Weapon(_) => 1,
                EventType::Enemy(_) => 2,
                EventType::Player(_) => 3,
                EventType::Graphics(_) => 4, // Graphics last
                EventType::Audio(_) => 5,
                EventType::Debug(_) => 6,
            }
        });
    }

    fn handle_collision_event(
        event: CollisionEvent,
        scheduler: &mut crate::engine::scheduler::Scheduler,
        graphics_events: &mut Vec<GraphicsEvent>,
    ) {
        // Handle cross-system collision effects
        match event {
            CollisionEvent::BulletHitEnemy {
                bullet_id,
                enemy_id,
                damage,
                impact_point,
            } => {
                // Use direct damage to avoid generating another damage event
                scheduler
                    .enemies_mut()
                    .damage_enemy_direct(enemy_id, damage, bullet_id);
                scheduler.bullets_mut().mark_bullet_for_removal(bullet_id);

                // Spawn hit particles
                // println!("Collision impact point: {:?}", impact_point);
                use crate::graphics::Color;
                graphics_events.push(GraphicsEvent::SpawnParticles {
                    position: impact_point,
                    velocity: Vec3::new(0.0, 1.0, 0.0), // Default upward
                    count: 15,
                    lifetime: 0.8,
                    color: Color::new(1.0, 0.6, 0.2, 1.0), // Orange sparks
                });
            }
            CollisionEvent::EnemyHitPlayer {
                enemy_id: _,
                player_id: _,
                damage,
            } => {
                scheduler.player_mut().player_mut().take_damage(damage);
                // Could queue screen shake event
            }
        }
    }

    fn handle_weapon_event(
        event: WeaponEvent,
        _scheduler: &mut crate::engine::scheduler::Scheduler,
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

    fn handle_collision_events_batch(
        events: Vec<CollisionEvent>,
        scheduler: &mut crate::engine::scheduler::Scheduler,
        graphics_events: &mut Vec<GraphicsEvent>,
    ) {
        if events.is_empty() {
            return;
        }

        // Group collision events by type for more efficient processing
        let mut bullet_hits: Vec<CollisionEvent> = Vec::new();
        let mut enemy_hits: Vec<CollisionEvent> = Vec::new();

        for event in events {
            match event {
                CollisionEvent::BulletHitEnemy { .. } => bullet_hits.push(event),
                CollisionEvent::EnemyHitPlayer { .. } => enemy_hits.push(event),
            }
        }

        // Batch process bullet hits - optimize bullet removal and particle spawning
        let mut bullets_to_remove = Vec::new();
        
        for event in bullet_hits {
            match event {
                CollisionEvent::BulletHitEnemy {
                    bullet_id,
                    enemy_id,
                    damage,
                    impact_point,
                } => {
                    // Apply damage directly
                    scheduler.enemies_mut().damage_enemy_direct(enemy_id, damage, bullet_id);
                    
                    // Collect bullet for batch removal
                    bullets_to_remove.push(bullet_id);
                    
                    // Queue particle effect
                    use crate::graphics::Color;
                    graphics_events.push(GraphicsEvent::SpawnParticles {
                        position: impact_point,
                        velocity: Vec3::new(0.0, 1.0, 0.0),
                        count: 15,
                        lifetime: 0.8,
                        color: Color::new(1.0, 0.6, 0.2, 1.0),
                    });
                }
                CollisionEvent::EnemyHitPlayer { .. } => {
                    // This should not happen in bullet_hits batch, but handle gracefully
                    unreachable!("EnemyHitPlayer event should not be in bullet_hits batch");
                }
            }
        }
        
        // Batch remove all bullets at once (more efficient than individual removals)
        for bullet_id in bullets_to_remove {
            scheduler.bullets_mut().mark_bullet_for_removal(bullet_id);
        }

        // Batch process enemy hits (could group damage, screen effects, etc.)
        for event in enemy_hits {
            Self::handle_collision_event(event, scheduler, graphics_events);
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
}

/// Main event type hierarchy
#[derive(Clone, Debug)]
pub enum EventType {
    Player(PlayerEvent),
    Enemy(EnemyEvent),
    Collision(CollisionEvent),
    Weapon(WeaponEvent),
    Graphics(GraphicsEvent),
    Audio(AudioEvent),
    Debug(DebugEvent),
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
