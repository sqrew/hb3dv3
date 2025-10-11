use super::super::behaviors::cannibal;
use super::super::entity::Enemy;
use super::super::types::EnemyType;
use super::spawner;
use crate::engine::{
    Vec3,
    dispatcher::{EventType, GraphicsEvent},
    entity::{EntityId, EntityManager},
};
use crate::graphics::Color;
use crate::scene::enemy::behaviors::blob::BLOB_MAX_FACTORY_COUNT;

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

    // (core_id, core_pos, occupied_positions, connected_positions)
    let mut growth_events = Vec::new();
    // (core_id, core_pos, occupied_positions)
    let mut factory_upgrade_events = Vec::new();

    // First pass: collect growth events
    for i in 0..enemies.len() {
        let core_id = enemies[i].entity_id();
        let core_pos = enemies[i].position();

        if let EnemyType::BlobCore(_) = enemies[i].enemy_type() {
            // Tick with mutable access and check capacity in real-time
            let (ticked_growth, ticked_factory) = {
                if let EnemyType::BlobCore(data) = enemies[i].enemy_type_mut() {
                    let is_at_max = data.is_at_max_capacity();
                    (
                        data.tick_growth(dt) && !is_at_max,
                        data.tick_factory_spawn(dt),
                    )
                } else {
                    (false, false)
                }
            };

            if ticked_growth || ticked_factory {
                if ticked_growth {
                    // Collect occupied grid positions, connected positions, and count factories
                    let mut occupied_positions = HashSet::new();
                    let mut connected_positions = HashSet::new();
                    occupied_positions.insert((0, 0, 0)); // Core at origin
                    connected_positions.insert((0, 0, 0)); // Core is always connected
                    let mut factory_count = 0;

                    let mut disconnected_nodes = Vec::new();
                    for other_enemy in enemies.iter() {
                        if let EnemyType::BlobNode(node_data) = other_enemy.enemy_type() {
                            if node_data.core_id == core_id {
                                occupied_positions.insert(node_data.grid_position);
                                // CRITICAL: Only use CONNECTED nodes for spawning
                                // Disconnected nodes should NEVER spawn new nodes
                                if node_data.connected_to_core {
                                    connected_positions.insert(node_data.grid_position);
                                    // Only count CONNECTED factories for growth rate
                                    if node_data.is_factory() {
                                        factory_count += 1;
                                    }
                                } else {
                                    // Track disconnected nodes with their distance
                                    let dist = node_data.grid_position.0.abs()
                                        + node_data.grid_position.1.abs()
                                        + node_data.grid_position.2.abs();
                                    disconnected_nodes.push((
                                        node_data.grid_position,
                                        dist,
                                        node_data.is_factory(),
                                    ));
                                }
                            }
                        }
                    }

                    // Debug: Log disconnected node statistics and verify no disconnected nodes in connected_positions
                    let total_disconnected = disconnected_nodes.len();
                    let disconnected_factories = disconnected_nodes
                        .iter()
                        .filter(|(_, _, is_fac)| *is_fac)
                        .count();

                    // CRITICAL DEBUG: Check if any disconnected positions leaked into connected_positions
                    let mut leaked_disconnected = 0;
                    for (disconnected_pos, _, _) in &disconnected_nodes {
                        if connected_positions.contains(disconnected_pos) {
                            leaked_disconnected += 1;
                            println!(
                                "❌ BUG: Disconnected node at {:?} is IN connected_positions!",
                                disconnected_pos
                            );
                        }
                    }

                    if total_disconnected > 0 {
                        println!(
                            "🔍 Blob stats: {} connected, {} disconnected ({} factories), {} total, {} leaked",
                            connected_positions.len(),
                            total_disconnected,
                            disconnected_factories,
                            occupied_positions.len(),
                            leaked_disconnected
                        );

                        // DEBUG: Show sample of what's in connected_positions
                        let furthest_connected = connected_positions
                            .iter()
                            .map(|pos| (pos, pos.0.abs() + pos.1.abs() + pos.2.abs()))
                            .max_by_key(|(_, dist)| *dist);
                        if let Some((pos, dist)) = furthest_connected {
                            println!(
                                "  → Furthest connected node: {:?} at distance {}",
                                pos, dist
                            );
                        }
                    }

                    if disconnected_factories > 0 {
                        println!(
                            "⚠️  BUG: {} disconnected factories detected!",
                            disconnected_factories
                        );
                        disconnected_nodes.sort_by_key(|(_, dist, _)| -dist);
                        for (pos, dist, _is_factory) in
                            disconnected_nodes.iter().filter(|(_, _, f)| *f).take(3)
                        {
                            println!("  - Factory at {:?}, distance {}", pos, dist);
                        }
                    }

                    // Clone occupied_positions if we need it for both growth and factory
                    if ticked_factory {
                        factory_upgrade_events.push((
                            core_id,
                            core_pos,
                            occupied_positions.clone(),
                            connected_positions.clone(),
                        ));
                    }

                    // CRITICAL FIX: Only ONE growth event per tick to prevent cascading disconnected spawns
                    // Previously: spawned (1 + factory_count) nodes per tick
                    // Problem: Later spawns saw earlier spawns as "connected" before connectivity check
                    // Solution: One spawn per tick, faster growth rate instead
                    //
                    // Growth is now: 1 node per 0.1s = 10 nodes/second (was 1-10+ nodes per 0.1s)
                    growth_events.push((
                        core_id,
                        core_pos,
                        occupied_positions.clone(),
                        connected_positions.clone(),
                    ));
                } else if ticked_factory {
                    // Only factory tick, still need occupied_positions and connected_positions
                    let mut occupied_positions = HashSet::new();
                    let mut connected_positions = HashSet::new();
                    occupied_positions.insert((0, 0, 0));
                    connected_positions.insert((0, 0, 0));
                    for other_enemy in enemies.iter() {
                        if let EnemyType::BlobNode(node_data) = other_enemy.enemy_type() {
                            if node_data.core_id == core_id {
                                occupied_positions.insert(node_data.grid_position);
                                if node_data.connected_to_core {
                                    connected_positions.insert(node_data.grid_position);
                                }
                            }
                        }
                    }
                    factory_upgrade_events.push((
                        core_id,
                        core_pos,
                        occupied_positions,
                        connected_positions,
                    ));
                }
            }
        }
    }

    // Handle factory upgrades FIRST (before spawning new nodes)
    // This ensures we only upgrade nodes that existed before this tick
    for (core_id, core_pos, _occupied_positions, connected_positions) in factory_upgrade_events {
        // Calculate direction from core to player in grid space (cheaper than world space)
        let core_to_player_world = player_pos - core_pos;
        let player_grid_approx = (
            (core_to_player_world.x / blob::BLOB_GRID_SPACING) as i32,
            (core_to_player_world.y / blob::BLOB_GRID_SPACING) as i32,
            (core_to_player_world.z / blob::BLOB_GRID_SPACING) as i32,
        );

        // OPTIMIZATION: Pre-compute edge positions (O(n) instead of O(n²))
        let mut edge_positions = HashSet::new();
        for pos in &connected_positions {
            // Check if this position has at least one empty adjacent
            if blob::get_adjacent_positions(*pos)
                .iter()
                .any(|adj| !connected_positions.contains(adj))
            {
                edge_positions.insert(*pos);
            }
        }

        // Find all connected base nodes on edges for this blob
        let mut edge_base_nodes = Vec::new(); // (entity_id, grid_position, score)

        // Find connected edge base nodes with score (only alignment to player)
        for enemy in enemies.iter() {
            if let EnemyType::BlobNode(node_data) = enemy.enemy_type() {
                if node_data.core_id == core_id
                    && node_data.is_base()
                    && node_data.connected_to_core
                    && edge_positions.contains(&node_data.grid_position)
                {
                    let grid_pos = node_data.grid_position;

                    // SAFETY CHECK: Only upgrade factories with 2+ connected neighbors
                    // This prevents creating vulnerable single-connection factories
                    let connected_neighbor_count = blob::get_adjacent_positions(grid_pos)
                        .iter()
                        .filter(|adj| connected_positions.contains(adj))
                        .count();

                    if connected_neighbor_count >= 2 {
                        // Grid-space dot product for alignment to player
                        let dot_product = grid_pos.0 * player_grid_approx.0
                            + grid_pos.1 * player_grid_approx.1
                            + grid_pos.2 * player_grid_approx.2;

                        // Score based purely on alignment to player direction
                        // No distance bonus - prevents prioritizing doomed disconnected arms
                        let score = if dot_product > 0 {
                            dot_product * 10
                        } else {
                            -1000 // Penalize nodes away from player
                        };

                        edge_base_nodes.push((enemy.entity_id(), grid_pos, score));
                    }
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

    // Spawn new nodes for growing blobs AFTER factory upgrades
    // CRITICAL: Process growth events ONE AT A TIME with connectivity checks between them
    for (core_id, core_pos, occupied_positions, connected_positions) in growth_events {
        // Read current phase and health from core
        let (phase, health_percent) = enemies
            .iter()
            .find(|e| e.entity_id() == core_id)
            .map(|e| {
                let health_pct = e.health() / e.max_health();
                if let EnemyType::BlobCore(data) = e.enemy_type() {
                    (data.phase, health_pct)
                } else {
                    (BlobPhase::Bolstering, 1.0)
                }
            })
            .unwrap_or((BlobPhase::Bolstering, 1.0));

        // If blob is damaged (below 70% health), force defensive backfilling
        let is_damaged = health_percent < 0.7;

        // OPTIMIZATION: Pre-compute edge positions ONCE for this growth event
        let mut edge_positions = HashSet::new();
        for pos in &connected_positions {
            if blob::get_adjacent_positions(*pos)
                .iter()
                .any(|adj| !connected_positions.contains(adj))
            {
                edge_positions.insert(*pos);
            }
        }

        // Find disconnected clusters worth saving (shared by both phases)
        // OPTIMIZATION: Only run cluster detection if we have disconnected nodes
        use std::collections::VecDeque;
        let disconnected_count = occupied_positions.len() - connected_positions.len();

        // AGGRESSIVE OPTIMIZATION: Skip cluster detection most of the time
        // Only check every ~5th growth tick (reduces frequency from 10Hz to 2Hz)
        use rand::Rng;
        let should_check_clusters = disconnected_count > 0
            && disconnected_count < 200  // Reduced from 500 - skip if too fragmented
            && rand::rng().random_range(0.0..1.0) < 0.2; // 20% chance = ~2Hz instead of 10Hz

        let valuable_clusters = if should_check_clusters {
            // Build a map of grid_position -> is_factory for fast lookups (single pass)
            // ONLY scan blob nodes for THIS core (filter early)
            let mut factory_positions = HashSet::new();
            for enemy in enemies.iter() {
                if let EnemyType::BlobNode(node_data) = enemy.enemy_type() {
                    if node_data.core_id == core_id && node_data.is_factory() {
                        factory_positions.insert(node_data.grid_position);
                    }
                }
            }

            let mut disconnected_positions: HashSet<(i32, i32, i32)> = occupied_positions
                .difference(&connected_positions)
                .copied()
                .collect();

            let mut clusters = Vec::new();

            // Limit cluster detection to prevent lag spikes
            let mut clusters_found = 0;
            const MAX_CLUSTERS_TO_CHECK: usize = 5; // Reduced from 10

            while !disconnected_positions.is_empty() && clusters_found < MAX_CLUSTERS_TO_CHECK {
                let start_pos = *disconnected_positions.iter().next().unwrap();
                let mut cluster = HashSet::new();
                let mut queue = VecDeque::new();
                let mut has_factory = false;

                queue.push_back(start_pos);
                cluster.insert(start_pos);
                disconnected_positions.remove(&start_pos);

                // Flood-fill to find entire disconnected cluster (with size limit)
                const MAX_CLUSTER_SIZE: usize = 50; // Reduced from 100
                while let Some(pos) = queue.pop_front() {
                    if cluster.len() >= MAX_CLUSTER_SIZE {
                        break; // Don't process giant disconnected blobs
                    }

                    // Check if this position is a factory (fast lookup)
                    if factory_positions.contains(&pos) {
                        has_factory = true;
                    }

                    for adj in blob::get_adjacent_positions(pos) {
                        if disconnected_positions.contains(&adj) {
                            disconnected_positions.remove(&adj);
                            cluster.insert(adj);
                            queue.push_back(adj);
                        }
                    }
                }

                let node_count = cluster.len();
                // Save cluster if it has 3+ nodes or any factory
                if node_count >= 3 || has_factory {
                    clusters.push((cluster, has_factory, node_count));
                }
                clusters_found += 1;
            }
            clusters
        } else {
            Vec::new() // Skip cluster detection if too many disconnected nodes
        };

        match phase {
            BlobPhase::Bolstering => {
                // Phase 1: Omnidirectional growth - prioritize backfilling gaps

                // Find all potential spawn positions with priority scores
                let mut spawn_candidates = Vec::new();

                // DEBUG: Verify we're only iterating connected positions
                let connected_count_before_spawn = connected_positions.len();
                println!(
                    "🎯 Bolstering: Checking {} connected positions for spawn opportunities",
                    connected_count_before_spawn
                );

                for pos in &connected_positions {
                    for adj in blob::get_adjacent_positions(*pos) {
                        if !occupied_positions.contains(&adj) {
                            // Count how many CONNECTED neighbors this position would have
                            let neighbor_count = blob::get_adjacent_positions(adj)
                                .iter()
                                .filter(|neighbor_pos| connected_positions.contains(neighbor_pos))
                                .count();

                            // DISABLED: Reconnection causes runaway disconnected growth
                            // Check if this position would reconnect a valuable cluster
                            let reconnection_bonus = 0;
                            // for (cluster, has_factory, node_count) in &valuable_clusters {
                            //     let touches_cluster = blob::get_adjacent_positions(adj)
                            //         .iter()
                            //         .any(|neighbor| cluster.contains(neighbor));
                            //
                            //     if touches_cluster {
                            //         // Massive bonus for reconnecting clusters
                            //         reconnection_bonus += if *has_factory {
                            //             5000 // Critical: save factories
                            //         } else {
                            //             1000 * (*node_count as i32) // Proportional to cluster size
                            //         };
                            //     }
                            // }

                            // Manhattan distance from core
                            let distance = adj.0.abs() + adj.1.abs() + adj.2.abs();

                            // If damaged, massively boost priority for ANY position with neighbors
                            // to create dense defensive structure
                            let base_priority = if is_damaged {
                                (neighbor_count as i32 * 1000) - distance
                            } else {
                                (neighbor_count as i32 * 100) - distance
                            };

                            let priority = base_priority + reconnection_bonus;
                            spawn_candidates.push((adj, priority));
                        }
                    }
                }

                // Remove duplicates by position
                spawn_candidates.sort_by_key(|(pos, _)| *pos);
                spawn_candidates.dedup_by_key(|(pos, _)| *pos);

                // Sort by priority (highest first)
                spawn_candidates.sort_by_key(|(_, priority)| -priority);

                if !spawn_candidates.is_empty() {
                    let (new_pos, priority) = spawn_candidates[0];

                    // Verify we're spawning from a connected position
                    let spawning_from_connected = blob::get_adjacent_positions(new_pos)
                        .iter()
                        .any(|adj| connected_positions.contains(adj));

                    if !spawning_from_connected {
                        println!(
                            "⚠️  BUG: Bolstering spawning at {:?} NOT adjacent to connected nodes!",
                            new_pos
                        );
                    }

                    // DEBUG: Log Bolstering spawns
                    let dist = new_pos.0.abs() + new_pos.1.abs() + new_pos.2.abs();
                    if dist > 30 {
                        println!(
                            "📍 BOLSTERING spawn at {:?} (dist={}, priority={}, connected_check={})",
                            new_pos, dist, priority, spawning_from_connected
                        );
                    }

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
                // Phase 2: Intelligent growth - balance backfilling with aggression
                // (valuable_clusters already computed above - shared between phases)

                // Collect ALL potential spawn positions with neighbor counts
                let mut all_spawn_candidates = Vec::new();

                for pos in &connected_positions {
                    for adj in blob::get_adjacent_positions(*pos) {
                        if !occupied_positions.contains(&adj) {
                            // Count how many CONNECTED neighbors this position would have
                            let neighbor_count = blob::get_adjacent_positions(adj)
                                .iter()
                                .filter(|neighbor_pos| connected_positions.contains(neighbor_pos))
                                .count();

                            // Check if this position would reconnect a valuable cluster
                            let mut is_reconnection = false;
                            for (cluster, _has_factory, _node_count) in &valuable_clusters {
                                let touches_cluster = blob::get_adjacent_positions(adj)
                                    .iter()
                                    .any(|neighbor| cluster.contains(neighbor));

                                if touches_cluster {
                                    is_reconnection = true;
                                    break;
                                }
                            }

                            all_spawn_candidates.push((adj, neighbor_count, is_reconnection));
                        }
                    }
                }

                // Remove duplicates
                all_spawn_candidates.sort_by_key(|(pos, _, _)| *pos);
                all_spawn_candidates.dedup_by_key(|(pos, _, _)| *pos);

                // DISABLED: Reconnection logic causes disconnected arms to keep growing
                // The issue: spawned reconnection nodes start with connected_to_core=true,
                // but might actually be disconnected, allowing them to spawn children before
                // connectivity check runs, creating runaway disconnected growth.
                //
                // TODO: Re-enable with proper connectivity validation
                let _reconnection_candidates: Vec<_> = all_spawn_candidates
                    .iter()
                    .filter(|(_, _, is_reconnection)| *is_reconnection)
                    .copied()
                    .collect();

                // Always use normal backfill/aggression logic (no reconnection)
                {
                    // No reconnections available, proceed with normal backfill/aggression logic

                    // Calculate player direction for scoring
                    let core_to_player_world = player_pos - core_pos;
                    let player_grid_approx = (
                        (core_to_player_world.x / blob::BLOB_GRID_SPACING) as i32,
                        (core_to_player_world.y / blob::BLOB_GRID_SPACING) as i32,
                        (core_to_player_world.z / blob::BLOB_GRID_SPACING) as i32,
                    );

                    // If damaged, prioritize ALL gaps (even 1-2 neighbors) for maximum defensive coverage
                    // Otherwise, only consider critical gaps (3+ neighbors)
                    let backfill_threshold = if is_damaged { 1 } else { 3 };

                    let backfill_candidates: Vec<_> = all_spawn_candidates
                        .iter()
                        .filter(|(_, count, _)| *count >= backfill_threshold)
                        .copied()
                        .collect();

                    // If damaged, ALWAYS backfill if possible (defensive mode)
                    // Otherwise, 30% chance to backfill (reduced from 50% - more aggressive)
                    use rand::Rng;
                    let should_backfill = if is_damaged {
                        !backfill_candidates.is_empty()
                    } else {
                        !backfill_candidates.is_empty() && rand::rng().random_range(0.0..1.0) < 0.3
                    };

                    if should_backfill {
                        // Score backfill positions
                        let mut scored_backfill: Vec<_> = backfill_candidates
                            .iter()
                            .map(|(pos, count, _)| {
                                let score = if is_damaged {
                                    // DEFENSIVE MODE: Prioritize HIGHEST neighbor count (densest gaps)
                                    // This fills internal voids near the core, NOT long edges
                                    // Distance penalty ensures we fill closest gaps first
                                    let distance_from_core =
                                        pos.0.abs() + pos.1.abs() + pos.2.abs();
                                    (*count as i32 * 1000) - distance_from_core
                                } else {
                                    // NORMAL MODE: Balance neighbor count with player direction
                                    let dot_product = pos.0 * player_grid_approx.0
                                        + pos.1 * player_grid_approx.1
                                        + pos.2 * player_grid_approx.2;

                                    let neighbor_score = *count as i32 * 100;
                                    let alignment_score = if dot_product > 0 {
                                        dot_product * 50 // Strong bonus for player-facing positions
                                    } else {
                                        dot_product * 10 // Mild penalty for positions away from player
                                    };

                                    neighbor_score + alignment_score
                                };

                                (*pos, score)
                            })
                            .collect();

                        // Sort by score
                        scored_backfill.sort_by_key(|(_, score)| -score);
                        let (new_pos, _) = scored_backfill[0];
                        spawner::spawn_blob_node(
                            enemies,
                            core_id,
                            core_pos,
                            new_pos,
                            false,
                            entity_manager,
                        );
                    } else if is_damaged {
                        // DAMAGED but no backfill candidates - pick random position close to core
                        // This ensures growth continues even when already dense
                        if !all_spawn_candidates.is_empty() {
                            // Score all candidates by proximity to core (defensive)
                            let mut defensive_candidates: Vec<_> = all_spawn_candidates
                                .iter()
                                .map(|(pos, _, _)| {
                                    let distance = pos.0.abs() + pos.1.abs() + pos.2.abs();
                                    (*pos, -distance) // Negative distance = closer is better
                                })
                                .collect();

                            defensive_candidates.sort_by_key(|(_, score)| -score);

                            // Pick from top 20% closest positions
                            let top_count = (defensive_candidates.len() / 5).max(1);
                            let random_idx = rand::rng().random_range(0..top_count);
                            let (new_pos, _) = defensive_candidates[random_idx];

                            spawner::spawn_blob_node(
                                enemies,
                                core_id,
                                core_pos,
                                new_pos,
                                false,
                                entity_manager,
                            );
                        }
                    } else {
                        // HEALTHY - Normal aggressive expansion toward player
                        // (player_grid_approx already calculated above)

                        // Find all connected edge positions with score (use pre-computed edges)
                        let mut edge_positions_with_score = Vec::new();
                        for pos in &edge_positions {
                            // Dot product with player direction (alignment score)
                            let dot_product = pos.0 * player_grid_approx.0
                                + pos.1 * player_grid_approx.1
                                + pos.2 * player_grid_approx.2;

                            // Score based purely on alignment to player direction
                            // No distance bonus - we just want to reach toward player
                            let score = if dot_product > 0 {
                                dot_product * 10
                            } else {
                                -1000
                            };

                            edge_positions_with_score.push((*pos, score));
                        }

                        edge_positions_with_score.sort_by_key(|(_, score)| -score);

                        if !edge_positions_with_score.is_empty() {
                            let top_candidates = (edge_positions_with_score.len() * 1 / 10).max(1);
                            let random_idx = rand::rng().random_range(0..top_candidates);
                            let (edge_pos, score) = edge_positions_with_score[random_idx];
                            let direction =
                                blob::direction_toward_target(edge_pos, player_pos, core_pos);

                            let new_pos = (
                                edge_pos.0 + direction.0,
                                edge_pos.1 + direction.1,
                                edge_pos.2 + direction.2,
                            );

                            // DEBUG: Log aggressive expansion spawns
                            let dist = new_pos.0.abs() + new_pos.1.abs() + new_pos.2.abs();
                            if dist > 30 {
                                println!(
                                    "📍 AGGRESSIVE spawn at {:?} from edge {:?} (dist={}, score={})",
                                    new_pos, edge_pos, dist, score
                                );
                            }

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
        }
    }
}

/// Check connectivity of blob nodes and mark disconnected ones
pub fn check_blob_connectivity(enemies: &mut [Enemy]) {
    use crate::engine::entity::EntityId;
    use std::collections::{HashMap, HashSet, VecDeque};

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
    use crate::engine::entity::EntityId;
    use std::collections::HashSet;

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
            EnemyType::BlobCore(_) => {
                // Blob cores regenerate 10% health per second
                let max_health = enemies[i].max_health();
                let regen_amount = max_health * 0.1 * dt;
                enemies[i].heal(regen_amount);
                // Blob cores are stationary, no update needed
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

                // Find base target (player or prey)
                let base_target = snake::find_snake_target(head_pos, &enemies, head_id, player_pos);

                // Calculate slithering target with perpendicular oscillation
                let slither_target = if let EnemyType::Snake(data) = enemies[i].enemy_type_mut() {
                    snake::calculate_slither_target(data, head_pos, base_target, dt)
                } else {
                    base_target
                };

                // Update head movement with slithering target
                enemies[i].update_with_target(dt, slither_target);

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

        let enemy_color = enemy_type.config().color;
        event_queue.push(EventType::Enemy(EnemyEvent::Die {
            enemy_id,
            color: enemy_color,
        }));

        // Handle logic based on type
        match enemy_type {
            EnemyType::Drone => {
                event_queue.push(EventType::Graphics(GraphicsEvent::SpawnParticles {
                    position,
                    velocity: Vec3::new(0.0, 0.0, 0.0),
                    count: 150,
                    lifetime: 2.0,
                    color: enemy_color,
                }));
                use crate::graphics::PrimitiveType;
                event_queue.push(EventType::Graphics(GraphicsEvent::SpawnShapeParticles {
                    position,
                    velocity: Vec3::new(0.0, 0.0, 0.0), // Slight upward velocity
                    count: 8,
                    lifetime: 2.0,
                    color: enemy_color,
                    primitive_type: PrimitiveType::Triangle2D,
                    angular_velocity: Vec3::new(3.0, 3.0, 3.0), // Spin in all directions
                    scale: 0.3,
                }));
            }
            EnemyType::Chaser => {
                event_queue.push(EventType::Graphics(GraphicsEvent::SpawnParticles {
                    position,
                    velocity: Vec3::new(0.0, 0.0, 0.0),
                    count: 150,
                    lifetime: 2.0,
                    color: enemy_color,
                }));
                use crate::graphics::PrimitiveType;
                event_queue.push(EventType::Graphics(GraphicsEvent::SpawnShapeParticles {
                    position,
                    velocity: Vec3::new(0.0, 0.0, 0.0), // Slight upward velocity
                    count: 8,
                    lifetime: 2.0,
                    color: enemy_color,
                    primitive_type: PrimitiveType::Diamond2D,
                    angular_velocity: Vec3::new(3.0, 3.0, 3.0), // Spin in all directions
                    scale: 0.3,
                }));
            }
            EnemyType::Heavy => {
                event_queue.push(EventType::Graphics(GraphicsEvent::SpawnParticles {
                    position,
                    velocity: Vec3::new(0.0, 0.0, 0.0),
                    count: 150,
                    lifetime: 2.0,
                    color: enemy_color,
                }));
                use crate::graphics::PrimitiveType;
                event_queue.push(EventType::Graphics(GraphicsEvent::SpawnShapeParticles {
                    position,
                    velocity: Vec3::new(0.0, 0.0, 0.0), // Slight upward velocity
                    count: 8,
                    lifetime: 2.0,
                    color: enemy_color,
                    primitive_type: PrimitiveType::Circle2D,
                    angular_velocity: Vec3::new(3.0, 3.0, 3.0), // Spin in all directions
                    scale: 0.3,
                }));
            }
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
                        color: enemy_color,
                    }));

                    // Shape particles for splitter split effect
                    use crate::graphics::PrimitiveType;
                    event_queue.push(EventType::Graphics(GraphicsEvent::SpawnShapeParticles {
                        position,
                        velocity: Vec3::new(0.0, 1.0, 0.0),
                        count: 6,
                        lifetime: 1.5,
                        color: enemy_color,
                        primitive_type: PrimitiveType::Diamond2D,
                        angular_velocity: Vec3::new(3.0, 3.0, 3.0),
                        scale: 0.25,
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
                        lifetime: 1.5,
                        color: enemy_color,
                    }));

                    // Shape particles for shield break effect
                    use crate::graphics::PrimitiveType;
                    event_queue.push(EventType::Graphics(GraphicsEvent::SpawnShapeParticles {
                        position,
                        velocity: Vec3::new(0.0, 0.5, 0.0),
                        count: 5,
                        lifetime: 1.5,
                        color: enemy_color,
                        primitive_type: PrimitiveType::Hexagon2D,
                        angular_velocity: Vec3::new(3.0, 3.0, 3.0),
                        scale: 0.2,
                    }));
                }
            }
            EnemyType::Cannibal(_) => {
                // Cannibal death - use star shapes for predatory enemy
                use crate::graphics::PrimitiveType;
                event_queue.push(EventType::Graphics(GraphicsEvent::SpawnParticles {
                    position,
                    velocity: Vec3::new(0.0, 0.0, 0.0),
                    count: 100,
                    lifetime: 1.5,
                    color: enemy_color,
                }));
                event_queue.push(EventType::Graphics(GraphicsEvent::SpawnShapeParticles {
                    position,
                    velocity: Vec3::new(0.0, 3.0, 0.0),
                    count: 10,
                    lifetime: 1.5,
                    color: enemy_color,
                    primitive_type: PrimitiveType::Star2D,
                    angular_velocity: Vec3::new(3.0, 3.0, 3.0),
                    scale: 0.35,
                }));
            }
            EnemyType::SnakeSegment(_) => {
                // Cannibal death - use star shapes for predatory enemy
                use crate::graphics::PrimitiveType;
                event_queue.push(EventType::Graphics(GraphicsEvent::SpawnParticles {
                    position,
                    velocity: Vec3::new(0.0, 0.0, 0.0),
                    count: 100,
                    lifetime: 1.5,
                    color: enemy_color,
                }));
                event_queue.push(EventType::Graphics(GraphicsEvent::SpawnShapeParticles {
                    position,
                    velocity: Vec3::new(0.0, 3.0, 0.0),
                    count: 10,
                    lifetime: 1.5,
                    color: enemy_color,
                    primitive_type: PrimitiveType::Cross2D,
                    angular_velocity: Vec3::new(3.0, 3.0, 3.0),
                    scale: 0.35,
                }));
            }
            _ => {}
        }
    }

    // Before removing dead enemies, notify blob cores about dead nodes
    let dead_blob_nodes: Vec<(EntityId, EntityId)> = enemies
        .iter()
        .filter(|e| !e.is_alive())
        .filter_map(|e| {
            if let EnemyType::BlobNode(node_data) = e.enemy_type() {
                Some((node_data.core_id, e.entity_id()))
            } else {
                None
            }
        })
        .collect();

    // Notify cores about their dead nodes
    for (core_id, node_id) in dead_blob_nodes {
        for enemy in enemies.iter_mut() {
            if enemy.entity_id() == core_id {
                if let EnemyType::BlobCore(core_data) = enemy.enemy_type_mut() {
                    core_data.remove_node(node_id);
                }
                break;
            }
        }
    }

    // Remove dead enemies
    enemies.retain(|e| e.is_alive());
}
