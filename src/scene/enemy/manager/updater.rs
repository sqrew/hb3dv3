use super::super::behaviors::cannibal;
use super::super::entity::Enemy;
use super::super::types::EnemyType;
use super::spawner;
use crate::engine::{
    Vec3,
    dispatcher::{EventType, GraphicsEvent},
    entity::EntityManager,
};
use crate::graphics::Color;
use crate::scene::enemy::behaviors::blob::{
    BLOB_MAX_FACTORY_COUNT, BLOB_WITHER_RATE_MAX, BLOB_WITHER_RATE_MIN,
};

/// Tick cooldowns and timers for enemies that have them
pub fn tick_cooldowns(enemies: &mut [Enemy], dt: f32) {
    for enemy in enemies.iter_mut() {
        if let EnemyType::Cannibal(data) = enemy.enemy_type_mut() {
            data.tick_cooldown(dt);
        }
    }
}

/// Handle snake growth - spawn new segments when timer expires
pub fn handle_snake_growth(enemies: &mut Vec<Enemy>, entity_manager: &mut EntityManager, dt: f32) {
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
    let mut eating_events = Vec::new(); // (cannibal_id, prey_id, prey_pos)

    // Find cannibals that can eat
    for i in 0..enemies.len() {
        if let EnemyType::Cannibal(data) = enemies[i].enemy_type() {
            if data.can_eat() {
                let cannibal_pos = enemies[i].position();
                let cannibal_id = enemies[i].entity_id();

                if let Some(prey_idx) =
                    cannibal::find_prey_in_range(cannibal_pos, &enemies, cannibal_id)
                {
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

/// Handle blob growth - spawn new nodes based on phase
pub fn handle_blob_growth(
    enemies: &mut Vec<Enemy>,
    entity_manager: &mut EntityManager,
    dt: f32,
    player_pos: Vec3,
) {
    use super::super::behaviors::blob::{self, BlobPhase};
    use std::collections::HashSet;
    use crate::engine::entity::EntityId;

    // (core_id, core_pos, occupied_positions)
    let mut growth_events = Vec::new();
    // (core_id, core_pos, occupied_positions)
    let mut factory_upgrade_events = Vec::new();

    // First pass: collect growth events
    for i in 0..enemies.len() {
        let core_id = enemies[i].entity_id();
        let core_pos = enemies[i].position();

        if let EnemyType::BlobCore(_) = enemies[i].enemy_type() {
            // Extract data before mutable operations
            let (should_grow, is_at_max) = {
                if let EnemyType::BlobCore(data) = enemies[i].enemy_type() {
                    (true, data.is_at_max_capacity())
                } else {
                    (false, false)
                }
            };

            if should_grow {
                // Now tick with mutable access
                let (ticked_growth, ticked_factory) = {
                    if let EnemyType::BlobCore(data) = enemies[i].enemy_type_mut() {
                        (
                            data.tick_growth(dt) && !is_at_max,
                            data.tick_factory_spawn(dt),
                        )
                    } else {
                        (false, false)
                    }
                };

                if ticked_growth {
                    // Collect occupied grid positions for this blob
                    let mut occupied_positions = HashSet::new();
                    occupied_positions.insert((0, 0, 0)); // Core at origin

                    for other_enemy in enemies.iter() {
                        if let EnemyType::BlobNode(node_data) = other_enemy.enemy_type() {
                            if node_data.core_id == core_id {
                                occupied_positions.insert(node_data.grid_position);
                            }
                        }
                    }

                    // Clone occupied_positions if we need it for both growth and factory
                    if ticked_factory {
                        factory_upgrade_events.push((core_id, core_pos, occupied_positions.clone()));
                    }
                    growth_events.push((core_id, core_pos, occupied_positions));
                } else if ticked_factory {
                    // Only factory tick, still need occupied_positions
                    let mut occupied_positions = HashSet::new();
                    occupied_positions.insert((0, 0, 0));
                    for other_enemy in enemies.iter() {
                        if let EnemyType::BlobNode(node_data) = other_enemy.enemy_type() {
                            if node_data.core_id == core_id {
                                occupied_positions.insert(node_data.grid_position);
                            }
                        }
                    }
                    factory_upgrade_events.push((core_id, core_pos, occupied_positions));
                }
            }
        }
    }

    // Spawn new nodes for growing blobs
    for (core_id, core_pos, occupied_positions) in growth_events {
        // Read current phase from core (phase may have changed after spawning previous nodes)
        let phase = enemies
            .iter()
            .find(|e| e.entity_id() == core_id)
            .and_then(|e| {
                if let EnemyType::BlobCore(data) = e.enemy_type() {
                    Some(data.phase)
                } else {
                    None
                }
            })
            .unwrap_or(BlobPhase::Bolstering);

        match phase {
            BlobPhase::Bolstering => {
                // Phase 1: Omnidirectional growth - grow in all empty adjacent positions
                let mut new_positions = Vec::new();

                for pos in &occupied_positions {
                    for adj in blob::get_adjacent_positions(*pos) {
                        if !occupied_positions.contains(&adj) {
                            new_positions.push(adj);
                        }
                    }
                }

                // Pick the position closest to core for layer-by-layer cubic growth
                if !new_positions.is_empty() {
                    // Sort by Manhattan distance from origin (core at 0,0,0)
                    new_positions.sort_by_key(|pos| pos.0.abs() + pos.1.abs() + pos.2.abs());

                    let new_pos = new_positions[0];
                    spawner::spawn_blob_node(
                        enemies,
                        core_id,
                        core_pos,
                        new_pos,
                        false, // Not a factory
                        entity_manager,
                    );
                }
            }
            BlobPhase::Reaching => {
                // Phase 2: Directional growth - prioritize edges toward player
                // Calculate direction from core to player in grid space (cheaper)
                let core_to_player_world = player_pos - core_pos;
                let player_grid_approx = (
                    (core_to_player_world.x / blob::BLOB_GRID_SPACING) as i32,
                    (core_to_player_world.y / blob::BLOB_GRID_SPACING) as i32,
                    (core_to_player_world.z / blob::BLOB_GRID_SPACING) as i32,
                );

                // Find all edge positions with score using grid-space math
                let mut edge_positions_with_score = Vec::new();
                for pos in &occupied_positions {
                    if blob::is_edge_position(*pos, &occupied_positions) {
                        // Grid-space dot product (all integer math, very fast)
                        let dot_product = pos.0 * player_grid_approx.0
                            + pos.1 * player_grid_approx.1
                            + pos.2 * player_grid_approx.2;

                        // Manhattan distance from core
                        let distance = pos.0.abs() + pos.1.abs() + pos.2.abs();

                        // Score: heavily weight alignment to player, bonus for distance
                        let score = if dot_product > 0 {
                            (dot_product * 10) + (distance / 2)
                        } else {
                            -1000 // Penalize nodes away from player
                        };

                        edge_positions_with_score.push((*pos, score));
                    }
                }

                // Sort by score (highest first) to prioritize edges toward player
                edge_positions_with_score.sort_by_key(|(_, score)| -score);

                // Pick from the top 10% highest scoring edges (or at least 1) for aggressive tendrils
                if !edge_positions_with_score.is_empty() {
                    use rand::Rng;
                    let top_candidates = (edge_positions_with_score.len() * 1 / 10).max(1);
                    let random_idx = rand::rng().random_range(0..top_candidates);
                    let (edge_pos, _) = edge_positions_with_score[random_idx];
                    let direction = blob::direction_toward_target(edge_pos, player_pos, core_pos);

                    let new_pos = (
                        edge_pos.0 + direction.0,
                        edge_pos.1 + direction.1,
                        edge_pos.2 + direction.2,
                    );

                    // Try preferred direction first, then fallback to any empty adjacent
                    if !occupied_positions.contains(&new_pos) {
                        spawner::spawn_blob_node(
                            enemies,
                            core_id,
                            core_pos,
                            new_pos,
                            false,
                            entity_manager,
                        );
                    } else {
                        // Preferred direction blocked - try any adjacent position
                        let adjacent = blob::get_adjacent_positions(edge_pos);
                        let available: Vec<_> = adjacent
                            .iter()
                            .filter(|pos| !occupied_positions.contains(pos))
                            .collect();

                        if !available.is_empty() {
                            let fallback_idx = rand::rng().random_range(0..available.len());
                            spawner::spawn_blob_node(
                                enemies,
                                core_id,
                                core_pos,
                                *available[fallback_idx],
                                false,
                                entity_manager,
                            );
                        }
                    }
                }
            }
        }
    }

    // Handle factory upgrades - upgrade base nodes on furthest edges toward player (creating focused tendrils)
    for (core_id, core_pos, occupied_positions) in factory_upgrade_events {
        // Calculate direction from core to player in grid space (cheaper than world space)
        let core_to_player_world = player_pos - core_pos;
        let player_grid_approx = (
            (core_to_player_world.x / blob::BLOB_GRID_SPACING) as i32,
            (core_to_player_world.y / blob::BLOB_GRID_SPACING) as i32,
            (core_to_player_world.z / blob::BLOB_GRID_SPACING) as i32,
        );

        // Find all base nodes on edges for this blob
        let mut edge_base_nodes = Vec::new(); // (entity_id, grid_position, score)

        // Find edge base nodes with score (distance + alignment to player) using grid-space math
        for enemy in enemies.iter() {
            if let EnemyType::BlobNode(node_data) = enemy.enemy_type() {
                if node_data.core_id == core_id
                    && node_data.is_base()
                    && blob::is_edge_position(node_data.grid_position, &occupied_positions)
                {
                    let grid_pos = node_data.grid_position;

                    // Grid-space dot product (cheaper - no sqrt, all integer math)
                    let dot_product = grid_pos.0 * player_grid_approx.0
                        + grid_pos.1 * player_grid_approx.1
                        + grid_pos.2 * player_grid_approx.2;

                    // Manhattan distance from core
                    let distance = grid_pos.0.abs() + grid_pos.1.abs() + grid_pos.2.abs();

                    // Score: heavily weight alignment to player (dot product), bonus for distance
                    let score = if dot_product > 0 {
                        (dot_product * 10) + (distance / 2)
                    } else {
                        -1000 // Penalize nodes away from player
                    };

                    edge_base_nodes.push((enemy.entity_id(), grid_pos, score));
                }
            }
        }

        // Sort by score (descending) - highest scoring nodes first
        edge_base_nodes.sort_by_key(|(_, _, score)| -score);

        // Count current factories
        let factory_count = enemies
            .iter()
            .filter(|e| {
                if let EnemyType::BlobNode(node_data) = e.enemy_type() {
                    node_data.core_id == core_id && node_data.is_factory()
                } else {
                    false
                }
            })
            .count();

        // Upgrade one of the highest scoring edge base nodes to factory (creates focused tendrils toward player)
        if factory_count < BLOB_MAX_FACTORY_COUNT as usize && !edge_base_nodes.is_empty() {
            // Pick from the top 10% highest scoring nodes (or at least 1) for aggressive player-directed growth
            use rand::Rng;
            let top_candidates = (edge_base_nodes.len() / 10).max(1);
            let random_idx = rand::rng().random_range(0..top_candidates);
            let node_id = edge_base_nodes[random_idx].0;

            for enemy in enemies.iter_mut() {
                if enemy.entity_id() == node_id {
                    if let EnemyType::BlobNode(node_data) = enemy.enemy_type_mut() {
                        node_data.upgrade_to_factory();
                        // Update config to match factory stats
                        let new_config = node_data.config();
                        enemy.update_config(new_config);
                    }
                    break;
                }
            }
        }
    }
}

/// Check connectivity of blob nodes and mark disconnected ones
pub fn check_blob_connectivity(enemies: &mut [Enemy]) {
    use std::collections::{HashMap, HashSet, VecDeque};
    use crate::engine::entity::EntityId;

    // Single pass: collect all blob cores and their nodes
    let mut blob_data: HashMap<EntityId, HashMap<(i32, i32, i32), usize>> = HashMap::new();
    let mut core_ids = Vec::new();

    for (idx, enemy) in enemies.iter().enumerate() {
        match enemy.enemy_type() {
            EnemyType::BlobCore(_) => {
                let core_id = enemy.entity_id();
                core_ids.push(core_id);
                blob_data.insert(core_id, HashMap::new());
            }
            EnemyType::BlobNode(node_data) => {
                if let Some(positions) = blob_data.get_mut(&node_data.core_id) {
                    positions.insert(node_data.grid_position, idx);
                }
            }
            _ => {}
        }
    }

    // For each blob, run flood-fill and update connectivity
    for core_id in core_ids {
        if let Some(node_positions) = blob_data.get(&core_id) {
            let core_grid_pos = (0, 0, 0);

            // Flood-fill from core to find all connected nodes
            let mut connected = HashSet::new();
            let mut queue = VecDeque::new();
            queue.push_back(core_grid_pos);
            connected.insert(core_grid_pos);

            while let Some(pos) = queue.pop_front() {
                for adj in super::super::behaviors::blob::get_adjacent_positions(pos) {
                    if node_positions.contains_key(&adj) && !connected.contains(&adj) {
                        connected.insert(adj);
                        queue.push_back(adj);
                    }
                }
            }

            // Update connectivity status using cached indices
            for (grid_pos, &enemy_idx) in node_positions.iter() {
                if let EnemyType::BlobNode(node_data) = enemies[enemy_idx].enemy_type_mut() {
                    node_data.connected_to_core = connected.contains(grid_pos);
                }
            }
        }
    }
}

/// Apply withering damage to disconnected blob nodes
pub fn apply_blob_withering(enemies: &mut [Enemy], dt: f32) {
    // Single pass - use cached wither rate instead of recalculating every frame
    for enemy in enemies.iter_mut() {
        if let EnemyType::BlobNode(node_data) = enemy.enemy_type() {
            if !node_data.connected_to_core {
                let max_hp = enemy.max_health();
                let wither_damage = max_hp * node_data.wither_rate * dt;
                enemy.take_damage(wither_damage);
            }
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
    use std::collections::HashSet;
    use crate::engine::entity::EntityId;

    // Build set of alive blob core IDs once (avoids O(n²) checks)
    let alive_blob_cores: HashSet<EntityId> = enemies
        .iter()
        .filter(|e| e.is_alive() && matches!(e.enemy_type(), EnemyType::BlobCore(_)))
        .map(|e| e.entity_id())
        .collect();

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
            EnemyType::BlobCore(_) => {
                // Blob cores are stationary
            }
            EnemyType::BlobNode(node_data) => {
                // Check if core is still alive - if not, mark as disconnected for withering
                let core_id = node_data.core_id;
                if !alive_blob_cores.contains(&core_id) {
                    // Core is dead - mark as disconnected so it withers
                    if let EnemyType::BlobNode(node_data) = enemies[i].enemy_type_mut() {
                        node_data.connected_to_core = false;
                    }
                }
                // Blob nodes are stationary
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
    for (_head_id, segment_ids) in snake_head_deaths {
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
