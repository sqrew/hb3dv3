use super::super::entity::Enemy;
use super::super::types::EnemyType;
use super::super::behaviors::cannibal;
use super::spawner;
use crate::engine::{Vec3, entity::EntityManager, dispatcher::{EventType, GraphicsEvent}};
use crate::graphics::Color;

/// Tick cooldowns and timers for enemies that have them
pub fn tick_cooldowns(enemies: &mut [Enemy], dt: f32) {
    for enemy in enemies.iter_mut() {
        if let EnemyType::Cannibal(data) = enemy.enemy_type_mut() {
            data.tick_cooldown(dt);
        }
    }
}

/// Handle snake growth - spawn new segments when timer expires
pub fn handle_snake_growth(
    enemies: &mut Vec<Enemy>,
    entity_manager: &mut EntityManager,
    dt: f32,
) {
    let mut growth_events = Vec::new(); // (head_id, head_pos, current_segment_count)

    // Check which snakes need to grow
    for enemy in enemies.iter_mut() {
        // Extract values before mutable borrow
        let head_id = enemy.entity_id();
        let head_pos = enemy.position();

        if let EnemyType::Snake(data) = enemy.enemy_type_mut() {
            if data.tick_growth(dt) {
                let segment_count = data.segment_count();
                growth_events.push((head_id, head_pos, segment_count));
            }
        }
    }

    // Spawn new segments for growing snakes
    for (head_id, head_pos, segment_count) in growth_events {
        spawner::spawn_snake_segment(enemies, head_id, head_pos, segment_count, entity_manager);
    }
}

/// Handle cannibal eating behavior
pub fn handle_predator_eating(
    enemies: &mut Vec<Enemy>,
    event_queue: &mut Vec<EventType>,
    _dt: f32,
) {
    use crate::engine::entity::EntityId;

    let mut eating_events = Vec::new(); // (cannibal_id, prey_id, prey_pos)

    // Find cannibals that can eat
    for i in 0..enemies.len() {
        if let EnemyType::Cannibal(data) = enemies[i].enemy_type() {
            if data.can_eat() {
                let cannibal_pos = enemies[i].position();
                let cannibal_id = enemies[i].entity_id();

                if let Some(prey_idx) = cannibal::find_prey_in_range(cannibal_pos, &enemies, cannibal_id) {
                    let prey_id = enemies[prey_idx].entity_id();
                    let prey_pos = enemies[prey_idx].position();
                    eating_events.push((cannibal_id, prey_id, prey_pos));
                }
            }
        }
    }

    // Process eating events: mark prey as dead and grow cannibals
    for (cannibal_id, prey_id, prey_pos) in eating_events {
        // Mark prey as dead (will be cleaned up by handle_deaths)
        for enemy in enemies.iter_mut() {
            if enemy.entity_id() == prey_id {
                enemy.take_damage(9999.0);
                break;
            }
        }

        // Grow cannibal
        if let Some(cannibal) = enemies.iter_mut().find(|e| e.entity_id() == cannibal_id) {
            cannibal.consume_enemy();

            event_queue.push(EventType::Graphics(GraphicsEvent::SpawnParticles {
                position: prey_pos,
                velocity: Vec3::new(0.0, 0.0, 0.0),
                count: 30,
                lifetime: 1.0,
                color: Color::new(0.8, 0.0, 0.0, 1.0),
            }));
        }
    }
}

/// Update vulnerability states for shield orb cores
pub fn update_vulnerability(enemies: &mut [Enemy]) {
    for i in 0..enemies.len() {
        if enemies[i].is_shield_orb_core() {
            let core_id = enemies[i].entity_id();

            let active_shield_count = enemies
                .iter()
                .filter(|e| {
                    if let Some(shield_core_id) = e.shield_core_id() {
                        shield_core_id == core_id && e.is_alive()
                    } else {
                        false
                    }
                })
                .count();

            enemies[i].update_vulnerability(active_shield_count);
        }
    }
}

/// Update all enemies based on their type
pub fn update_all(enemies: &mut [Enemy], dt: f32, player_pos: Vec3) {
    use super::super::behaviors::snake;

    // First pass: update snake heads and calculate segment positions
    let mut segment_updates = Vec::new(); // (segment_id, target_pos)

    for i in 0..enemies.len() {
        match enemies[i].enemy_type() {
            EnemyType::Heavy | EnemyType::Chaser | EnemyType::Drone => {
                enemies[i].update(dt, player_pos);
            }
            EnemyType::Cannibal(_) => {
                let cannibal_pos = enemies[i].position();
                let cannibal_id = enemies[i].entity_id();

                let target_pos = cannibal::find_prey_target(cannibal_pos, &enemies, cannibal_id)
                    .unwrap_or(player_pos);

                enemies[i].update_with_target(dt, target_pos);
            }
            EnemyType::Splitter(_) => {
                enemies[i].update(dt, player_pos);
            }
            EnemyType::Shield(_) => {
                if let Some(core_id) = enemies[i].shield_core_id() {
                    if let Some(core) = enemies.iter().find(|e| e.entity_id() == core_id) {
                        let core_pos = core.position();
                        enemies[i].update_shield_orbit(dt, core_pos);
                    } else {
                        enemies[i].take_damage(9999.0);
                    }
                }
            }
            EnemyType::ShieldOrbCore(_) => {
                enemies[i].update(dt, player_pos);
            }
            EnemyType::Snake(_) => {
                let head_pos = enemies[i].position();
                let head_id = enemies[i].entity_id();
                let head_vel = enemies[i].velocity();

                // Update head movement
                let target = snake::find_snake_target(head_pos, &enemies, head_id, player_pos);
                enemies[i].update_with_target(dt, target);

                // Calculate segment positions
                let updated_pos = enemies[i].position();
                if let EnemyType::Snake(data) = enemies[i].enemy_type() {
                    let positions = snake::calculate_segment_positions(
                        updated_pos,
                        head_vel,
                        &data.segment_ids,
                        &enemies,
                    );
                    segment_updates.extend(positions);
                }
            }
            EnemyType::SnakeSegment(_) => {
                // Segments don't update on their own - position set by snake head
            }
        }
    }

    // Second pass: update segment positions
    for (segment_id, target_pos) in segment_updates {
        if let Some(segment) = enemies.iter_mut().find(|e| e.entity_id() == segment_id) {
            // Smoothly move segment toward target position
            let current_pos = segment.position();
            let direction = target_pos - current_pos;
            let distance = direction.magnitude();

            if distance > 0.1 {
                let speed = 100.0; // Fast following speed
                let max_move = speed * dt;
                let move_amount = max_move.min(distance);
                let new_pos = current_pos + direction.normalize() * move_amount;
                segment.set_position(new_pos);
            }
        }
    }
}

/// Handle enemy deaths and generate death events
pub fn handle_deaths(
    enemies: &mut Vec<Enemy>,
    event_queue: &mut Vec<EventType>,
    entity_manager: &mut EntityManager,
) {
    let mut enemies_to_remove = Vec::new();
    let mut segment_deaths = Vec::new(); // (segment_id, head_id)
    let mut snake_head_deaths = Vec::new(); // (head_id, segment_ids)

    for enemy in &*enemies {
        if !enemy.is_alive() {
            // Track snake head deaths to kill all segments
            if let EnemyType::Snake(data) = enemy.enemy_type() {
                snake_head_deaths.push((enemy.entity_id(), data.segment_ids.clone()));
            }

            // Track snake segment deaths to update head
            if let EnemyType::SnakeSegment(data) = enemy.enemy_type() {
                segment_deaths.push((enemy.entity_id(), data.head_id));
            }

            enemies_to_remove.push((
                enemy.entity_id(),
                enemy.position(),
                enemy.enemy_type().clone(),
            ));
        }
    }

    // Kill all segments when snake head dies
    for (head_id, segment_ids) in snake_head_deaths {
        for segment_id in segment_ids {
            for enemy in enemies.iter_mut() {
                if enemy.entity_id() == segment_id {
                    enemy.take_damage(9999.0);
                    break;
                }
            }
        }
    }

    // Remove segments from their snake heads
    for (segment_id, head_id) in segment_deaths {
        for enemy in enemies.iter_mut() {
            if enemy.entity_id() == head_id {
                enemy.remove_segment(segment_id);
                break;
            }
        }
    }

    // Generate death events and handle splitting
    for (enemy_id, position, enemy_type) in enemies_to_remove {
        use crate::engine::dispatcher::EnemyEvent;

        event_queue.push(EventType::Enemy(EnemyEvent::Die { enemy_id }));

        // Death particles
        event_queue.push(EventType::Graphics(GraphicsEvent::SpawnParticles {
            position,
            velocity: Vec3::new(0.0, 0.0, 0.0),
            count: 150,
            lifetime: 2.0,
            color: Color::GREEN,
        }));

        // Handle splitting based on type
        match enemy_type {
            EnemyType::Splitter(data) => {
                if data.current_generation < data.max_generation {
                    spawner::spawn_splitter_children(
                        enemies,
                        position,
                        data.current_generation,
                        data.max_generation,
                        entity_manager,
                    );

                    event_queue.push(EventType::Graphics(GraphicsEvent::SpawnParticles {
                        position,
                        velocity: Vec3::new(0.0, 0.0, 0.0),
                        count: 50,
                        lifetime: 1.5,
                        color: Color::ORANGE,
                    }));
                }
            }
            EnemyType::Shield(data) => {
                if data.current_generation < data.max_generation {
                    spawner::spawn_shield_children(
                        enemies,
                        position,
                        data.current_generation,
                        data.max_generation,
                        data.core_id,
                        data.orbit_angle,
                        data.orbit_inclination,
                        data.orbit_radius,
                        entity_manager,
                    );

                    event_queue.push(EventType::Graphics(GraphicsEvent::SpawnParticles {
                        position,
                        velocity: Vec3::new(0.0, 0.0, 0.0),
                        count: 30,
                        lifetime: 1.0,
                        color: Color::new(0.3, 0.6, 1.0, 1.0),
                    }));
                }
            }
            _ => {}
        }
    }

    // Remove dead enemies
    enemies.retain(|e| e.is_alive());
}
