pub mod spawner;
pub mod updater;

use super::entity::Enemy;
use super::types::EnemyType;
use crate::engine::Vec3;
use crate::engine::dispatcher::{EnemyEvent, EventType};
use crate::graphics::{Color, Primitive};

pub struct EnemyManager {
    enemies: Vec<Enemy>,
    event_queue: Vec<EventType>,
    spawn_timer: f32,
    spawn_interval: f32,
}

impl EnemyManager {
    pub fn new() -> Self {
        Self {
            enemies: Vec::new(),
            event_queue: Vec::new(),
            spawn_timer: 0.0,
            spawn_interval: 0.5,
        }
    }

    pub fn spawn_initial_enemies(
        &mut self,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) {
        spawner::create_formations(&mut self.enemies, entity_manager);
    }

    pub fn update(
        &mut self,
        dt: f32,
        player_pos: Vec3,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) {
        // Tick cooldowns
        updater::tick_cooldowns(&mut self.enemies, dt);

        // Handle snake growth
        updater::handle_snake_growth(&mut self.enemies, entity_manager, dt);

        // Check blob connectivity BEFORE growth (so growth can use current connectivity data)
        updater::check_blob_connectivity(&mut self.enemies);

        // Handle blob growth (uses connectivity data from above)
        updater::handle_blob_growth(&mut self.enemies, entity_manager, dt, player_pos);

        // CRITICAL: Check connectivity IMMEDIATELY after growth
        // This prevents newly-spawned-but-actually-disconnected nodes from spawning children
        updater::check_blob_connectivity(&mut self.enemies);

        // Handle predator eating behavior
        updater::handle_predator_eating(&mut self.enemies, &mut self.event_queue, dt);

        // Apply withering to disconnected blob nodes
        updater::apply_blob_withering(&mut self.enemies, dt);

        // Update core vulnerability states
        updater::update_vulnerability(&mut self.enemies);

        // Update all enemies
        updater::update_all(&mut self.enemies, dt, player_pos);

        // Handle deaths (including segment cleanup)
        updater::handle_deaths(&mut self.enemies, &mut self.event_queue, entity_manager);

        // Spawn new enemies periodically
        self.spawn_timer += dt;
        if self.spawn_timer >= self.spawn_interval {
            self.spawn_timer = 0.0;
            spawner::spawn_near_player(&mut self.enemies, player_pos, entity_manager);
        }
    }

    pub fn get_render_data(&self) -> Vec<Primitive> {
        self.enemies
            .iter()
            .map(|enemy| {
                let config = enemy.config();
                let health_percent = enemy.health_percentage();

                // Apply health-based color tinting
                let color = match enemy.enemy_type() {
                    EnemyType::BlobCore(_) => {
                        // Core turns red below 70% health
                        if health_percent < 0.7 {
                            Color::YELLOW
                        } else if health_percent < 0.3 {
                            Color::RED
                        } else {
                            config.color
                        }
                    }
                    EnemyType::BlobNode(_) => {
                        // Nodes turn red below 30% health
                        if health_percent < 0.4 {
                            Color::YELLOW
                        } else if health_percent < 0.2 {
                            Color::RED
                        } else {
                            config.color
                        }
                    }
                    _ => config.color, // Other enemies use normal color
                };

                Primitive::new(config.primitive_type, enemy.position(), color)
                    .with_uniform_scale(config.visual_scale)
            })
            .collect()
    }

    pub fn enemies(&self) -> &[Enemy] {
        &self.enemies
    }

    pub fn enemies_mut(&mut self) -> &mut [Enemy] {
        &mut self.enemies
    }

    pub fn remove_enemy(&mut self, index: usize) -> bool {
        if index < self.enemies.len() {
            self.enemies.remove(index);
            true
        } else {
            false
        }
    }

    pub fn enemy_count(&self) -> usize {
        self.enemies.len()
    }

    pub fn cleanup_dead_enemies(&mut self) -> Vec<crate::engine::entity::EntityId> {
        let mut destroyed_entities = Vec::new();

        let mut i = 0;
        while i < self.enemies.len() {
            if !self.enemies[i].is_alive() {
                destroyed_entities.push(self.enemies[i].entity_id());
                self.enemies.remove(i);
            } else {
                i += 1;
            }
        }

        destroyed_entities
    }

    pub fn damage_enemy(
        &mut self,
        entity_id: crate::engine::entity::EntityId,
        damage: f32,
    ) -> bool {
        self.damage_enemy_direct(entity_id, damage, entity_id)
    }

    pub fn damage_enemy_direct(
        &mut self,
        entity_id: crate::engine::entity::EntityId,
        damage: f32,
        _source: crate::engine::entity::EntityId,
    ) -> bool {
        // Check if this is a snake head hit - route damage to last segment
        let mut segment_to_damage = None;
        for enemy in &mut self.enemies {
            if enemy.entity_id() == entity_id {
                if let Some(segment_id) = enemy.route_snake_damage(damage) {
                    segment_to_damage = Some(segment_id);
                    break;
                }
            }
        }

        // If damage was routed to a segment, damage that instead
        if let Some(segment_id) = segment_to_damage {
            // Routed damage to segment - bypass invulnerability
            for enemy in &mut self.enemies {
                if enemy.entity_id() == segment_id {
                    let old_health = enemy.health();
                    enemy.take_damage_routed(damage);

                    if enemy.health() <= 0.0 && old_health > 0.0 {
                        let enemy_type = enemy.enemy_type().clone();
                        let enemy_pos = enemy.position();
                        let enemy_color = enemy.config().color;

                        self.event_queue.push(EventType::Score(
                            crate::engine::dispatcher::ScoreEvent::EnemyKilled {
                                enemy_id: segment_id,
                                enemy_type,
                                position: enemy_pos,
                            },
                        ));

                        self.event_queue.push(EventType::Enemy(EnemyEvent::Die {
                            enemy_id: segment_id,
                            color: enemy_color,
                        }));
                    }

                    return true;
                }
            }
        } else {
            // Normal damage (head or other enemy with no routing)
            for enemy in &mut self.enemies {
                if enemy.entity_id() == entity_id {
                    let old_health = enemy.health();
                    enemy.take_damage(damage);

                    if enemy.health() <= 0.0 && old_health > 0.0 {
                        let enemy_type = enemy.enemy_type().clone();
                        let enemy_pos = enemy.position();
                        let enemy_color = enemy.config().color;

                        self.event_queue.push(EventType::Score(
                            crate::engine::dispatcher::ScoreEvent::EnemyKilled {
                                enemy_id: entity_id,
                                enemy_type,
                                position: enemy_pos,
                            },
                        ));

                        self.event_queue.push(EventType::Enemy(EnemyEvent::Die {
                            enemy_id: entity_id,
                            color: enemy_color,
                        }));
                    }

                    return true;
                }
            }
        }
        false
    }

    pub fn damage_enemy_with_event(
        &mut self,
        entity_id: crate::engine::entity::EntityId,
        damage: f32,
        source: crate::engine::entity::EntityId,
    ) -> bool {
        // Check if this is a snake head hit - route damage to last segment
        let mut segment_to_damage = None;
        for enemy in &mut self.enemies {
            if enemy.entity_id() == entity_id {
                if let Some(segment_id) = enemy.route_snake_damage(damage) {
                    segment_to_damage = Some(segment_id);
                    break;
                }
            }
        }

        // If damage was routed to a segment, damage that instead
        if let Some(segment_id) = segment_to_damage {
            // Routed damage to segment - bypass invulnerability
            for enemy in &mut self.enemies {
                if enemy.entity_id() == segment_id {
                    let old_health = enemy.health();
                    enemy.take_damage_routed(damage);

                    self.event_queue
                        .push(EventType::Enemy(EnemyEvent::TakeDamage {
                            enemy_id: segment_id,
                            amount: damage,
                            source,
                        }));

                    if enemy.health() <= 0.0 && old_health > 0.0 {
                        let enemy_color = enemy.config().color;
                        self.event_queue.push(EventType::Enemy(EnemyEvent::Die {
                            enemy_id: segment_id,
                            color: enemy_color,
                        }));
                    }

                    return true;
                }
            }
        } else {
            // Normal damage (head or other enemy with no routing)
            for enemy in &mut self.enemies {
                if enemy.entity_id() == entity_id {
                    let old_health = enemy.health();
                    enemy.take_damage(damage);

                    self.event_queue
                        .push(EventType::Enemy(EnemyEvent::TakeDamage {
                            enemy_id: entity_id,
                            amount: damage,
                            source,
                        }));

                    if enemy.health() <= 0.0 && old_health > 0.0 {
                        let enemy_color = enemy.config().color;
                        self.event_queue.push(EventType::Enemy(EnemyEvent::Die {
                            enemy_id: entity_id,
                            color: enemy_color,
                        }));
                    }

                    return true;
                }
            }
        }
        false
    }

    pub fn mark_enemy_for_removal(&mut self, entity_id: crate::engine::entity::EntityId) -> bool {
        for enemy in &mut self.enemies {
            if enemy.entity_id() == entity_id {
                enemy.take_damage(9999.0);
                return true;
            }
        }
        false
    }

    pub fn get_enemy_position(&self, entity_id: crate::engine::entity::EntityId) -> Option<Vec3> {
        for enemy in &self.enemies {
            if enemy.entity_id() == entity_id {
                return Some(enemy.position());
            }
        }
        None
    }

    pub fn get_enemy_color(&self, entity_id: crate::engine::entity::EntityId) -> Option<crate::graphics::color::Color> {
        for enemy in &self.enemies {
            if enemy.entity_id() == entity_id {
                return Some(enemy.config().color);
            }
        }
        None
    }

    pub fn get_all_enemy_positions(&self) -> Vec<Vec3> {
        self.enemies.iter().map(|enemy| enemy.position()).collect()
    }

    pub fn spawn_enemy_near_player(
        &mut self,
        player_pos: Vec3,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) {
        spawner::spawn_near_player(&mut self.enemies, player_pos, entity_manager);
    }

    pub fn drain_events(&mut self) -> Vec<EventType> {
        self.event_queue.drain(..).collect()
    }

    pub fn spawn_splitter(
        &mut self,
        position: Vec3,
        max_generation: u8,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) {
        spawner::spawn_splitter(&mut self.enemies, position, max_generation, entity_manager);
    }

    pub fn spawn_cannibal(
        &mut self,
        position: Vec3,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) {
        spawner::spawn_cannibal(&mut self.enemies, position, entity_manager);
    }

    pub fn spawn_snake(
        &mut self,
        position: Vec3,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) {
        spawner::spawn_snake(&mut self.enemies, position, entity_manager);
    }

    pub fn spawn_blob(
        &mut self,
        position: Vec3,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) {
        spawner::spawn_blob(&mut self.enemies, position, entity_manager);
    }

    pub fn spawn_shield_orb_boss(
        &mut self,
        position: Vec3,
        initial_shield_count: usize,
        max_shield_generation: u8,
        orbit_radius: f32,
        entity_manager: &mut crate::engine::entity::EntityManager,
    ) {
        spawner::spawn_shield_orb_boss(
            &mut self.enemies,
            position,
            initial_shield_count,
            max_shield_generation,
            orbit_radius,
            entity_manager,
        );
    }

    pub fn handle_event(&mut self, event: crate::engine::dispatcher::EnemyEvent) {
        use crate::engine::dispatcher::{EventType, GraphicsEvent};

        match event {
            EnemyEvent::TakeDamage {
                enemy_id,
                amount,
                source,
            } => {
                self.damage_enemy_with_event(enemy_id, amount, source);
            }
            EnemyEvent::Die { enemy_id, color } => {
                for enemy in &mut self.enemies {
                    if enemy.entity_id() == enemy_id {
                        let enemy_pos = enemy.position();

                        self.event_queue
                            .push(EventType::Graphics(GraphicsEvent::SpawnParticles {
                                position: enemy_pos,
                                velocity: Vec3::new(0.0, 0.0, 0.0),
                                count: 100,
                                lifetime: 2.0,
                                color,
                            }));

                        enemy.take_damage(9999.0);
                        break;
                    }
                }
            }
            EnemyEvent::Spawn {
                position,
                enemy_type: _,
            } => {
                let temp_id = (self.enemies.len() as u32).wrapping_add(50000);
                let enemy_entity = crate::engine::entity::EntityId(temp_id);
                self.enemies.push(Enemy::new(
                    enemy_entity,
                    position,
                    Vec3::zeros(),
                    EnemyType::Drone,
                ));
            }
            EnemyEvent::Split {
                parent_id: _,
                position: _,
                current_generation: _,
                max_generation: _,
            } => {
                // Split handling is now done automatically in updater::handle_deaths
            }
        }
    }
}
