use super::super::entity::Enemy;
use super::super::types::EnemyType;
use super::super::behaviors::cannibal::CannibalData;
use super::super::behaviors::splitter::SplitterData;
use super::super::behaviors::shield::{ShieldData, ShieldOrbCoreData};
use super::super::behaviors::snake::{SnakeData, SnakeSegmentData};
use super::super::behaviors::blob::{BlobCoreData, BlobNodeData};
use crate::engine::{Vec3, entity::{EntityManager, EntityType, EntityId}};

/// Spawn initial enemy formations
pub fn create_formations(
    enemies: &mut Vec<Enemy>,
    entity_manager: &mut EntityManager,
) {
    use rand::Rng;

    // 1. Orbital Ring Formation (classic spiral around center)
    for i in 0..50 {
        let angle = (i as f32 / 50.0) * std::f32::consts::TAU;
        let radius = 8.0;
        let pos = Vec3::new(
            angle.cos() * radius,
            (i as f32) * 0.25 - 1.0,
            angle.sin() * radius,
        );
        let vel = Vec3::new(-angle.sin() * 2.0, 0.0, angle.cos() * 2.0);

        let enemy_entity = entity_manager.create_entity(EntityType::Enemy);
        enemies.push(Enemy::new(enemy_entity, pos, vel, EnemyType::random()));
    }

    // 2. Spherical Shell Formation (hollow sphere of enemies)
    for _i in 0..100 {
        let theta = rand::rng().random_range(0.0..std::f32::consts::TAU);
        let phi = rand::rng().random_range(0.0..std::f32::consts::PI);
        let radius = 25.0;

        let sin_phi = phi.sin();
        let pos = Vec3::new(
            sin_phi * theta.cos() * radius,
            phi.cos() * radius,
            sin_phi * theta.sin() * radius,
        );
        let vel = Vec3::new(0.0, 0.0, 0.0);

        let enemy_entity = entity_manager.create_entity(EntityType::Enemy);
        enemies.push(Enemy::new(enemy_entity, pos, vel, EnemyType::random()));
    }

    // 3. Linear Stream Formation (enemies in lines toward center)
    for direction in 0..8 {
        let base_angle = (direction as f32 / 8.0) * std::f32::consts::TAU;
        for distance in 1..20 {
            let radius = distance as f32 * 5.0;
            let pos = Vec3::new(
                base_angle.cos() * radius,
                rand::rng().random_range(-3.0..3.0),
                base_angle.sin() * radius,
            );
            let vel = Vec3::new(
                -base_angle.cos() * 1.5 + rand::rng().random_range(-0.5..0.5),
                rand::rng().random_range(-0.2..0.2),
                -base_angle.sin() * 1.5 + rand::rng().random_range(-0.5..0.5),
            );

            let enemy_entity = entity_manager.create_entity(EntityType::Enemy);
            enemies.push(Enemy::new(enemy_entity, pos, vel, EnemyType::random()));
        }
    }

    // 4. Vertical Tower Formation (enemies stacked vertically)
    for layer in 0..15 {
        let y = (layer as f32 - 7.0) * 4.0;
        let enemies_in_layer = 8 + (layer % 3) * 2;

        for i in 0..enemies_in_layer {
            let angle = (i as f32 / enemies_in_layer as f32) * std::f32::consts::TAU;
            let radius = 15.0 + (layer as f32 * 0.5);
            let pos = Vec3::new(angle.cos() * radius, y, angle.sin() * radius);
            let vel = Vec3::new(
                -angle.sin() * 0.5,
                rand::rng().random_range(-0.1..0.1),
                angle.cos() * 0.5,
            );

            let enemy_entity = entity_manager.create_entity(EntityType::Enemy);
            enemies.push(Enemy::new(enemy_entity, pos, vel, EnemyType::random()));
        }
    }

    // 5. Chaotic Cloud Formation (random cluster far away)
    let cloud_center = Vec3::new(50.0, 20.0, 50.0);
    for _i in 0..80 {
        let offset = Vec3::new(
            rand::rng().random_range(-10.0..10.0),
            rand::rng().random_range(-10.0..10.0),
            rand::rng().random_range(-10.0..10.0),
        );
        let pos = cloud_center + offset;
        let vel = Vec3::new(
            rand::rng().random_range(-1.0..1.0),
            rand::rng().random_range(-1.0..1.0),
            rand::rng().random_range(-1.0..1.0),
        );

        let enemy_entity = entity_manager.create_entity(EntityType::Enemy);
        enemies.push(Enemy::new(enemy_entity, pos, vel, EnemyType::random()));
    }
}

/// Spawn a new enemy near the player position (in a 3D sphere around player)
pub fn spawn_near_player(
    enemies: &mut Vec<Enemy>,
    player_pos: Vec3,
    entity_manager: &mut EntityManager,
) {
    use rand::Rng;

    // Random distance from player (5-100 units away)
    let distance = rand::rng().random_range(5.0..=100.0);

    // Generate random point on sphere using spherical coordinates
    let theta = rand::rng().random_range(0.0..std::f32::consts::TAU);
    let phi = rand::rng().random_range(0.0..std::f32::consts::PI);

    // Convert spherical to cartesian coordinates
    let sin_phi = phi.sin();
    let offset_x = sin_phi * theta.cos() * distance;
    let offset_y = phi.cos() * distance;
    let offset_z = sin_phi * theta.sin() * distance;

    let spawn_pos = Vec3::new(
        player_pos.x + offset_x,
        player_pos.y + offset_y,
        player_pos.z + offset_z,
    );

    // Initial velocity toward player with some randomness
    let to_player = player_pos - spawn_pos;
    let to_player_normalized = if to_player.magnitude() > 0.1 {
        to_player.normalize()
    } else {
        Vec3::new(1.0, 0.0, 0.0)
    };

    let base_speed = rand::rng().random_range(2.0..=5.0);
    let mut spawn_vel = to_player_normalized * base_speed;

    spawn_vel.x += rand::rng().random_range(-1.0..1.0);
    spawn_vel.y += rand::rng().random_range(-1.0..1.0);
    spawn_vel.z += rand::rng().random_range(-1.0..1.0);

    let enemy_entity = entity_manager.create_entity(EntityType::Enemy);
    enemies.push(Enemy::new(
        enemy_entity,
        spawn_pos,
        spawn_vel,
        EnemyType::random(),
    ));
}

/// Spawn a fractal splitter boss at a specific position
pub fn spawn_splitter(
    enemies: &mut Vec<Enemy>,
    position: Vec3,
    max_generation: u8,
    entity_manager: &mut EntityManager,
) {
    let enemy_entity = entity_manager.create_entity(EntityType::Enemy);
    enemies.push(Enemy::new(
        enemy_entity,
        position,
        Vec3::zeros(),
        EnemyType::Splitter(SplitterData::new(max_generation)),
    ));
}

/// Spawn a cannibal enemy at a specific position
pub fn spawn_cannibal(
    enemies: &mut Vec<Enemy>,
    position: Vec3,
    entity_manager: &mut EntityManager,
) {
    let enemy_entity = entity_manager.create_entity(EntityType::Enemy);
    enemies.push(Enemy::new(
        enemy_entity,
        position,
        Vec3::zeros(),
        EnemyType::Cannibal(CannibalData::new()),
    ));
}

/// Spawn a ShieldOrb boss (core with orbiting fractal shields)
pub fn spawn_shield_orb_boss(
    enemies: &mut Vec<Enemy>,
    position: Vec3,
    initial_shield_count: usize,
    max_shield_generation: u8,
    orbit_radius: f32,
    entity_manager: &mut EntityManager,
) {
    // Create the core
    let core_entity = entity_manager.create_entity(EntityType::Enemy);
    let mut shield_ids = Vec::new();

    // Distribute shields evenly on a sphere using Fibonacci sphere algorithm
    let golden_ratio = (1.0 + 5.0_f32.sqrt()) / 2.0;

    for i in 0..initial_shield_count {
        let idx = i as f32;
        let count = initial_shield_count as f32;

        // Azimuth angle (theta) - horizontal rotation
        let orbit_angle = std::f32::consts::TAU * idx / golden_ratio;

        // Polar angle (phi) - vertical angle from pole
        let z = 1.0 - (2.0 * (idx + 0.5)) / count;
        let orbit_inclination = z.acos();

        let shield_entity = entity_manager.create_entity(EntityType::Enemy);
        shield_ids.push(shield_entity);

        enemies.push(Enemy::new(
            shield_entity,
            position,
            Vec3::zeros(),
            EnemyType::Shield(ShieldData::new(
                core_entity,
                orbit_angle,
                orbit_inclination,
                orbit_radius,
                max_shield_generation,
            )),
        ));
    }

    // Now create the core with the shield IDs
    enemies.push(Enemy::new(
        core_entity,
        position,
        Vec3::zeros(),
        EnemyType::ShieldOrbCore(ShieldOrbCoreData::new(shield_ids)),
    ));
}

/// Spawn child splitters from a split event
pub fn spawn_splitter_children(
    enemies: &mut Vec<Enemy>,
    position: Vec3,
    current_generation: u8,
    max_generation: u8,
    entity_manager: &mut EntityManager,
) {
    use rand::Rng;

    let next_generation = current_generation + 1;

    for i in 0..2 {
        let angle = (i as f32 / 2.0) * std::f32::consts::TAU
            + rand::rng().random_range(0.0..0.5);
        let offset_distance = 2.0;

        let offset = Vec3::new(
            angle.cos() * offset_distance,
            rand::rng().random_range(-1.0..1.0),
            angle.sin() * offset_distance,
        );

        let outward_speed = 5.0;
        let vel = Vec3::new(
            angle.cos() * outward_speed,
            rand::rng().random_range(-2.0..2.0),
            angle.sin() * outward_speed,
        );

        let enemy_entity = entity_manager.create_entity(EntityType::Enemy);

        let mut data = SplitterData::new(max_generation);
        data.current_generation = next_generation;

        enemies.push(Enemy::new(
            enemy_entity,
            position + offset,
            vel,
            EnemyType::Splitter(data),
        ));
    }
}

/// Spawn child shields from a shield split event
pub fn spawn_shield_children(
    enemies: &mut Vec<Enemy>,
    position: Vec3,
    current_generation: u8,
    max_generation: u8,
    core_id: EntityId,
    orbit_angle: f32,
    orbit_inclination: f32,
    orbit_radius: f32,
    entity_manager: &mut EntityManager,
) {
    let next_generation = current_generation + 1;

    for i in 0..2 {
        let angle_offset = (i as f32 - 0.5) * 0.5;
        let new_orbit_angle = orbit_angle + angle_offset;

        let enemy_entity = entity_manager.create_entity(EntityType::Enemy);

        let mut data = ShieldData::new(
            core_id,
            new_orbit_angle,
            orbit_inclination,
            orbit_radius,
            max_generation,
        );
        data.current_generation = next_generation;

        enemies.push(Enemy::new(
            enemy_entity,
            position,
            Vec3::zeros(),
            EnemyType::Shield(data),
        ));
    }
}

/// Spawn a snake enemy
pub fn spawn_snake(
    enemies: &mut Vec<Enemy>,
    pos: Vec3,
    entity_manager: &mut EntityManager,
) {
    let enemy_entity = entity_manager.create_entity(EntityType::Enemy);
    let data = SnakeData::new();

    enemies.push(Enemy::new(
        enemy_entity,
        pos,
        Vec3::zeros(),
        EnemyType::Snake(data),
    ));
}

/// Spawn a snake segment for an existing snake
pub fn spawn_snake_segment(
    enemies: &mut Vec<Enemy>,
    head_id: EntityId,
    head_pos: Vec3,
    position_in_chain: usize,
    entity_manager: &mut EntityManager,
) {
    let segment_entity = entity_manager.create_entity(EntityType::Enemy);
    let data = SnakeSegmentData::new(head_id, position_in_chain);

    // Spawn segment slightly behind the head
    let spawn_offset = Vec3::new(0.0, 0.0, -5.0 * (position_in_chain as f32 + 1.0));
    let segment_pos = head_pos + spawn_offset;

    enemies.push(Enemy::new(
        segment_entity,
        segment_pos,
        Vec3::zeros(),
        EnemyType::SnakeSegment(data),
    ));

    // Register segment with snake head
    for enemy in enemies.iter_mut() {
        if enemy.entity_id() == head_id {
            if let EnemyType::Snake(snake_data) = enemy.enemy_type_mut() {
                snake_data.add_segment(segment_entity);
            }
            break;
        }
    }
}

/// Spawn a blob core enemy
pub fn spawn_blob(
    enemies: &mut Vec<Enemy>,
    pos: Vec3,
    entity_manager: &mut EntityManager,
) {
    let enemy_entity = entity_manager.create_entity(EntityType::Enemy);
    let data = BlobCoreData::new();

    enemies.push(Enemy::new(
        enemy_entity,
        pos,
        Vec3::zeros(),
        EnemyType::BlobCore(data),
    ));
}

/// Spawn a blob node for an existing blob core
pub fn spawn_blob_node(
    enemies: &mut Vec<Enemy>,
    core_id: EntityId,
    core_pos: Vec3,
    grid_position: (i32, i32, i32),
    is_factory: bool,
    entity_manager: &mut EntityManager,
) {
    use super::super::behaviors::blob;

    // DEBUG: Log every spawn
    let distance_from_core = grid_position.0.abs() + grid_position.1.abs() + grid_position.2.abs();
    if distance_from_core > 30 {
        println!(
            "🚨 SPAWNING {} at grid {:?}, distance {} from core!",
            if is_factory { "FACTORY" } else { "node" },
            grid_position,
            distance_from_core
        );
    }

    let node_entity = entity_manager.create_entity(EntityType::Enemy);
    let data = if is_factory {
        BlobNodeData::new_factory(core_id, grid_position)
    } else {
        BlobNodeData::new_base(core_id, grid_position)
    };

    // Calculate world position from grid position
    let world_pos = blob::grid_to_world_position(core_pos, grid_position);

    enemies.push(Enemy::new(
        node_entity,
        world_pos,
        Vec3::zeros(),
        EnemyType::BlobNode(data),
    ));

    // Register node with blob core
    for enemy in enemies.iter_mut() {
        if enemy.entity_id() == core_id {
            if let EnemyType::BlobCore(core_data) = enemy.enemy_type_mut() {
                core_data.add_node(node_entity);
            }
            break;
        }
    }
}
