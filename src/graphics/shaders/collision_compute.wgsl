// GPU Spatial Hashing Compute Shader
// Implements parallel spatial partitioning and collision detection on GPU

// Constants
const GRID_SIZE: u32 = 128u;  // 128x128x128 spatial grid
const CELL_SIZE: f32 = 10.0;  // Each cell is 10x10x10 units
const MAX_ENTITIES_PER_CELL: u32 = 64u;
const MAX_ENTITIES: u32 = 65536u;  // Support up to 64k entities
const WORKGROUP_SIZE: u32 = 256u;

// Hash table size (must be power of 2 for fast modulo)
const HASH_TABLE_SIZE: u32 = 16384u;  // 16k buckets
const MAX_COLLISIONS: u32 = 16384u;   // Maximum number of collision pairs

// Entity data structure
struct Entity {
    position: vec3<f32>,
    radius: f32,
    entity_id: u32,
    entity_type: u32,  // 0=player, 1=enemy, 2=playerProj, 3=enemyProj, 4=collectible
    collision_mask: u32,
    _padding: u32,
}

// Spatial hash cell
struct HashCell {
    entity_count: atomic<u32>,
    entities: array<u32, MAX_ENTITIES_PER_CELL>,
}

// Collision pair output
struct CollisionPair {
    entity_a: u32,
    entity_b: u32,
    distance_squared: f32,
    _padding: u32,
}

// Buffers
@group(0) @binding(0) var<storage, read> entities: array<Entity, MAX_ENTITIES>;
@group(0) @binding(1) var<storage, read> entity_count_buffer: array<u32>;
@group(0) @binding(2) var<storage, read_write> hash_table: array<HashCell, HASH_TABLE_SIZE>;
@group(0) @binding(3) var<storage, read_write> collision_pairs: array<CollisionPair, MAX_COLLISIONS>;
@group(0) @binding(4) var<storage, read_write> collision_count: atomic<u32>;

// Morton encoding for spatial hashing (Z-order curve)
fn expand_bits(v: u32) -> u32 {
    var x = v & 0x3FFu;  // 10 bits
    x = (x | (x << 16u)) & 0x30000FFu;
    x = (x | (x << 8u)) & 0x300F00Fu;
    x = (x | (x << 4u)) & 0x30C30C3u;
    x = (x | (x << 2u)) & 0x9249249u;
    return x;
}

fn morton_3d(x: u32, y: u32, z: u32) -> u32 {
    return expand_bits(x) | (expand_bits(y) << 1u) | (expand_bits(z) << 2u);
}

// Hash function for spatial grid - handles negative coordinates properly
fn spatial_hash(position: vec3<f32>) -> u32 {
    // Offset by large value to handle negative coordinates
    let offset = 32768.0; // Large enough to handle reasonable world bounds
    let grid_pos = vec3<u32>(
        u32((position.x + offset) / CELL_SIZE),
        u32((position.y + offset) / CELL_SIZE),
        u32((position.z + offset) / CELL_SIZE)
    );
    
    // Use Morton code for better spatial locality
    let morton = morton_3d(
        grid_pos.x & 0x3FFu,
        grid_pos.y & 0x3FFu,
        grid_pos.z & 0x3FFu
    );
    
    // Hash to table size
    return morton % HASH_TABLE_SIZE;
}

// Get all 27 neighboring cells (3x3x3 cube)
fn get_neighbor_hashes(position: vec3<f32>) -> array<u32, 27> {
    var neighbors: array<u32, 27>;
    var index = 0u;
    
    for (var dx = -1; dx <= 1; dx = dx + 1) {
        for (var dy = -1; dy <= 1; dy = dy + 1) {
            for (var dz = -1; dz <= 1; dz = dz + 1) {
                let cell_offset = vec3<f32>(
                    f32(dx) * CELL_SIZE,
                    f32(dy) * CELL_SIZE,
                    f32(dz) * CELL_SIZE
                );
                neighbors[index] = spatial_hash(position + cell_offset);
                index = index + 1u;
            }
        }
    }
    
    return neighbors;
}

// Clear hash table
@compute @workgroup_size(WORKGROUP_SIZE)
fn clear_spatial_hash(
    @builtin(global_invocation_id) global_id: vec3<u32>
) {
    let idx = global_id.x;
    if (idx >= HASH_TABLE_SIZE) {
        return;
    }
    
    atomicStore(&hash_table[idx].entity_count, 0u);
}

// Build spatial hash table
@compute @workgroup_size(WORKGROUP_SIZE)
fn build_spatial_hash(
    @builtin(global_invocation_id) global_id: vec3<u32>
) {
    let entity_idx = global_id.x;
    if (entity_idx >= entity_count_buffer[0]) {
        return;
    }
    
    let entity = entities[entity_idx];
    let hash = spatial_hash(entity.position);
    
    // Atomically add entity to hash cell
    let cell_idx = atomicAdd(&hash_table[hash].entity_count, 1u);
    if (cell_idx < MAX_ENTITIES_PER_CELL) {
        hash_table[hash].entities[cell_idx] = entity_idx;
    }
}

// Detect collisions using spatial hash
@compute @workgroup_size(WORKGROUP_SIZE)
fn detect_collisions(
    @builtin(global_invocation_id) global_id: vec3<u32>
) {
    let entity_idx = global_id.x;
    if (entity_idx >= entity_count_buffer[0]) {
        return;
    }
    
    let entity_a = entities[entity_idx];
    let neighbors = get_neighbor_hashes(entity_a.position);
    
    // Check all 27 neighboring cells (including current cell)
    for (var cell_idx = 0u; cell_idx < 27u; cell_idx = cell_idx + 1u) {
        let hash = neighbors[cell_idx];
        let cell_count = atomicLoad(&hash_table[hash].entity_count);
        
        // Check all entities in this cell
        for (var i = 0u; i < min(cell_count, MAX_ENTITIES_PER_CELL); i = i + 1u) {
            let entity_b_idx = hash_table[hash].entities[i];
            
            // Avoid duplicate pairs by only checking entities with higher indices
            if (entity_b_idx <= entity_idx) {
                continue;
            }
            
            let entity_b = entities[entity_b_idx];
            
            // Check collision masks (bidirectional)
            // Skip if neither entity is interested in colliding with the other's type
            if ((entity_a.collision_mask & (1u << entity_b.entity_type)) == 0u && 
                (entity_b.collision_mask & (1u << entity_a.entity_type)) == 0u) {
                continue;
            }
            
            // Distance check
            let diff = entity_a.position - entity_b.position;
            let dist_sq = dot(diff, diff);
            let radius_sum = entity_a.radius + entity_b.radius;
            
            if (dist_sq <= radius_sum * radius_sum) {
                // Collision detected! Add to output
                let pair_idx = atomicAdd(&collision_count, 1u);
                if (pair_idx < MAX_COLLISIONS) {
                    collision_pairs[pair_idx] = CollisionPair(
                        entity_a.entity_id,
                        entity_b.entity_id,
                        dist_sq,
                        0u
                    );
                }
            }
        }
    }
}

// Alternative: Broad phase only - return potential collision pairs
@compute @workgroup_size(WORKGROUP_SIZE)
fn broad_phase_pairs(
    @builtin(global_invocation_id) global_id: vec3<u32>
) {
    let entity_idx = global_id.x;
    if (entity_idx >= entity_count_buffer[0]) {
        return;
    }
    
    let entity_a = entities[entity_idx];
    let hash = spatial_hash(entity_a.position);
    let cell_count = atomicLoad(&hash_table[hash].entity_count);
    
    // Only check entities in same cell for broad phase
    for (var i = 0u; i < min(cell_count, MAX_ENTITIES_PER_CELL); i = i + 1u) {
        let entity_b_idx = hash_table[hash].entities[i];
        
        if (entity_b_idx <= entity_idx) {
            continue;
        }
        
        // Add as potential pair without distance check
        let pair_idx = atomicAdd(&collision_count, 1u);
        if (pair_idx < MAX_COLLISIONS) {
            collision_pairs[pair_idx] = CollisionPair(
                entity_idx,
                entity_b_idx,
                0.0,
                0u
            );
        }
    }
}