use crate::engine::EntityType;
use wgpu::util::DeviceExt;

/// Maximum number of entities supported by the collision system
pub const MAX_ENTITIES: u32 = 65536;
/// Hash table size for spatial partitioning (must be power of 2)
pub const HASH_TABLE_SIZE: u32 = 16384;
/// Maximum collision pairs that can be detected per frame
pub const MAX_COLLISIONS: u32 = 16384;
/// Maximum entities per spatial hash cell
pub const MAX_ENTITIES_PER_CELL: u32 = 64;
/// Compute shader workgroup size
pub const WORKGROUP_SIZE: u32 = 256;

/// Entity data structure that matches the GPU shader
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuEntity {
    pub position: [f32; 3],
    pub radius: f32,
    pub entity_id: u32,
    pub entity_type: u32,
    pub collision_mask: u32,
    pub _padding: u32,
}

impl GpuEntity {
    pub fn from_player(player: &crate::scene::Player) -> Self {
        Self {
            position: [
                player.position().x,
                player.position().y,
                player.position().z,
            ],
            radius: player.collision_radius(),
            entity_id: player.entity_id().id(),
            entity_type: EntityType::Player as u32,
            collision_mask: player.collision_mask().0,
            _padding: 0,
        }
    }

    pub fn from_enemy(enemy: &crate::scene::Enemy) -> Self {
        Self {
            position: [enemy.position().x, enemy.position().y, enemy.position().z],
            radius: enemy.collision_radius(),
            entity_id: enemy.entity_id().id(),
            entity_type: EntityType::Enemy as u32,
            collision_mask: enemy.collision_mask().0,
            _padding: 0,
        }
    }

    pub fn from_bullet(bullet: &crate::scene::Bullet) -> Self {
        Self {
            position: [
                bullet.position().x,
                bullet.position().y,
                bullet.position().z,
            ],
            radius: bullet.collision_radius(),
            entity_id: bullet.entity_id().id(),
            entity_type: EntityType::PlayerBullet as u32,
            collision_mask: bullet.collision_mask().0,
            _padding: 0,
        }
    }

    pub fn from_metabullet(bullet: &crate::scene::MetaBullet) -> Self {
        Self {
            position: [
                bullet.position().x,
                bullet.position().y,
                bullet.position().z,
            ],
            radius: bullet.collision_radius(),
            entity_id: bullet.entity_id().id(),
            entity_type: EntityType::PlayerBullet as u32,
            collision_mask: bullet.collision_mask().0,
            _padding: 0,
        }
    }

    pub fn from_large_body(large_body: &crate::scene::LargeBody) -> Self {
        Self {
            position: [
                large_body.position().x,
                large_body.position().y,
                large_body.position().z,
            ],
            radius: large_body.collision_radius(),
            entity_id: large_body.entity_id().id(),
            entity_type: EntityType::LargeBody as u32,
            collision_mask: large_body.collision_mask().0,
            _padding: 0,
        }
    }

    pub fn from_collectible(collectible: &crate::scene::scoring::ScoreMultiplier) -> Self {
        Self {
            position: [
                collectible.position().x,
                collectible.position().y,
                collectible.position().z,
            ],
            radius: collectible.collision_radius(),
            entity_id: collectible.entity_id().id(),
            entity_type: EntityType::Collectible as u32,
            collision_mask: collectible.collision_mask().0,
            _padding: 0,
        }
    }
}

/// Collision pair output from GPU
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CollisionPair {
    pub entity_a: u32,
    pub entity_b: u32,
    pub distance_squared: f32,
    pub _padding: u32,
}

/// Hash cell structure for spatial partitioning
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct HashCell {
    pub entity_count: u32,
    pub _padding: [u32; 3],
    pub entities: [u32; MAX_ENTITIES_PER_CELL as usize],
}

impl Default for HashCell {
    fn default() -> Self {
        Self {
            entity_count: 0,
            _padding: [0; 3],
            entities: [0; MAX_ENTITIES_PER_CELL as usize],
        }
    }
}

/// GPU collision detection system
pub struct CollisionCompute {
    // Compute shader and pipeline
    compute_shader: wgpu::ShaderModule,
    clear_pipeline: wgpu::ComputePipeline,
    build_pipeline: wgpu::ComputePipeline,
    detect_pipeline: wgpu::ComputePipeline,

    // GPU buffers
    entity_buffer: wgpu::Buffer,
    entity_count_buffer: wgpu::Buffer,
    hash_table_buffer: wgpu::Buffer,
    collision_pairs_buffer: wgpu::Buffer,
    collision_count_buffer: wgpu::Buffer,

    // Staging buffers for readback
    collision_pairs_staging: wgpu::Buffer,
    collision_count_staging: wgpu::Buffer,

    // Bind groups
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,

    // CPU entity data for upload
    cpu_entities: Vec<GpuEntity>,
}

impl CollisionCompute {
    pub fn new(device: &wgpu::Device) -> Self {
        // Load and compile compute shader
        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Collision Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/collision_compute.wgsl").into()),
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Collision Compute Bind Group Layout"),
            entries: &[
                // Entities buffer (read-only)
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Entity count (read-only storage)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Hash table (read-write)
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Collision pairs output (write-only)
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Collision count (read-write atomic)
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Create compute pipelines
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Collision Compute Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let clear_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Clear Spatial Hash Pipeline"),
            layout: Some(&pipeline_layout),
            module: &compute_shader,
            entry_point: Some("clear_spatial_hash"),
            cache: None,
            compilation_options: Default::default(),
        });

        let build_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Build Spatial Hash Pipeline"),
            layout: Some(&pipeline_layout),
            module: &compute_shader,
            entry_point: Some("build_spatial_hash"),
            cache: None,
            compilation_options: Default::default(),
        });

        let detect_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Detect Collisions Pipeline"),
            layout: Some(&pipeline_layout),
            module: &compute_shader,
            entry_point: Some("detect_collisions"),
            cache: None,
            compilation_options: Default::default(),
        });

        // Create GPU buffers
        let entity_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Entity Buffer"),
            size: (MAX_ENTITIES as usize * std::mem::size_of::<GpuEntity>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let entity_count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Entity Count Buffer"),
            contents: bytemuck::cast_slice(&[0u32]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let hash_table_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Hash Table Buffer"),
            size: (HASH_TABLE_SIZE as usize * std::mem::size_of::<HashCell>()) as u64,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let collision_pairs_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Collision Pairs Buffer"),
            size: (MAX_COLLISIONS as usize * std::mem::size_of::<CollisionPair>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let collision_count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Collision Count Buffer"),
            contents: bytemuck::cast_slice(&[0u32]),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
        });

        // Create staging buffers for CPU readback
        let collision_pairs_staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Collision Pairs Staging Buffer"),
            size: (MAX_COLLISIONS as usize * std::mem::size_of::<CollisionPair>()) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let collision_count_staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Collision Count Staging Buffer"),
            size: std::mem::size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Collision Compute Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: entity_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: entity_count_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: hash_table_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: collision_pairs_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: collision_count_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            compute_shader,
            clear_pipeline,
            build_pipeline,
            detect_pipeline,
            entity_buffer,
            entity_count_buffer,
            hash_table_buffer,
            collision_pairs_buffer,
            collision_count_buffer,
            collision_pairs_staging,
            collision_count_staging,
            bind_group_layout,
            bind_group,
            cpu_entities: Vec::new(),
        }
    }

    /// Update entity data from the game world
    pub fn update_entities(
        &mut self,
        player: &crate::scene::Player,
        enemies: &[crate::scene::Enemy],
        bullets: &[crate::scene::Bullet],
        metabullets: &[crate::scene::MetaBullet],
        large_bodies: &[crate::scene::LargeBody],
        collectibles: &[crate::scene::scoring::ScoreMultiplier],
    ) {
        self.cpu_entities.clear();

        // Add player
        self.cpu_entities.push(GpuEntity::from_player(player));

        // Add enemies
        for enemy in enemies {
            self.cpu_entities.push(GpuEntity::from_enemy(enemy));
        }

        // Add bullets (only active ones)
        for bullet in bullets {
            if bullet.active() {
                self.cpu_entities.push(GpuEntity::from_bullet(bullet));
            }
        }

        // Add metabullets (only active ones)
        for metabullet in metabullets {
            if metabullet.active() {
                self.cpu_entities
                    .push(GpuEntity::from_metabullet(metabullet));
            }
        }

        // Add large bodies
        for large_body in large_bodies {
            self.cpu_entities
                .push(GpuEntity::from_large_body(large_body));
        }

        // Add collectibles
        for collectible in collectibles {
            self.cpu_entities
                .push(GpuEntity::from_collectible(collectible));
        }
    }

    /// Dispatch collision detection on GPU
    pub fn dispatch_collision_detection(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Upload entity data to GPU
        if !self.cpu_entities.is_empty() {
            // Ensure we don't exceed buffer capacity
            if self.cpu_entities.len() > MAX_ENTITIES as usize {
                self.cpu_entities.truncate(MAX_ENTITIES as usize);
            }
            
            queue.write_buffer(
                &self.entity_buffer,
                0,
                bytemuck::cast_slice(&self.cpu_entities),
            );

            queue.write_buffer(
                &self.entity_count_buffer,
                0,
                bytemuck::cast_slice(&[self.cpu_entities.len() as u32]),
            );

            // Reset collision count
            queue.write_buffer(
                &self.collision_count_buffer,
                0,
                bytemuck::cast_slice(&[0u32]),
            );
        }

        // Create command encoder
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Collision Detection Command Encoder"),
        });

        // Phase 1: Clear spatial hash table
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Clear Spatial Hash Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.clear_pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);

            let workgroups = (HASH_TABLE_SIZE + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE;
            compute_pass.dispatch_workgroups(workgroups, 1, 1);
        }

        // Phase 2: Build spatial hash table
        if !self.cpu_entities.is_empty() {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Build Spatial Hash Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.build_pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);

            let entity_count = self.cpu_entities.len() as u32;
            let workgroups = (entity_count + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE;
            compute_pass.dispatch_workgroups(workgroups, 1, 1);
        }

        // Phase 3: Detect collisions
        if !self.cpu_entities.is_empty() {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Detect Collisions Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.detect_pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);

            let entity_count = self.cpu_entities.len() as u32;
            let workgroups = (entity_count + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE;
            compute_pass.dispatch_workgroups(workgroups, 1, 1);
        }

        // Copy results to staging buffers
        encoder.copy_buffer_to_buffer(
            &self.collision_count_buffer,
            0,
            &self.collision_count_staging,
            0,
            std::mem::size_of::<u32>() as u64,
        );

        encoder.copy_buffer_to_buffer(
            &self.collision_pairs_buffer,
            0,
            &self.collision_pairs_staging,
            0,
            (MAX_COLLISIONS as usize * std::mem::size_of::<CollisionPair>()) as u64,
        );

        // Submit commands
        queue.submit(Some(encoder.finish()));

        Ok(())
    }

    /// Read collision results synchronously (blocking)
    /// This is simple but will block the CPU - good for prototyping
    pub fn read_collision_results_sync(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Vec<CollisionPair>, Box<dyn std::error::Error>> {
        // First copy the collision count
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Collision Results Copy"),
        });

        encoder.copy_buffer_to_buffer(
            &self.collision_count_buffer,
            0,
            &self.collision_count_staging,
            0,
            std::mem::size_of::<u32>() as u64,
        );

        queue.submit(Some(encoder.finish()));

        // Map and read collision count
        let count_slice = self.collision_count_staging.slice(..);
        count_slice.map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::Wait).unwrap();

        let count_view = count_slice.get_mapped_range();
        let collision_count = bytemuck::cast_slice::<u8, u32>(&count_view)[0];
        drop(count_view);
        self.collision_count_staging.unmap();

        // print number of collisions per frame
        // if collision_count > 0 {
        //     println!("GPU: Found {} collisions!", collision_count);
        // }

        if collision_count == 0 {
            return Ok(Vec::new());
        }

        // Now copy and read collision pairs
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Collision Pairs Copy"),
        });

        let pairs_size = (collision_count.min(MAX_COLLISIONS) as usize
            * std::mem::size_of::<CollisionPair>()) as u64;
        encoder.copy_buffer_to_buffer(
            &self.collision_pairs_buffer,
            0,
            &self.collision_pairs_staging,
            0,
            pairs_size,
        );

        queue.submit(Some(encoder.finish()));

        let pairs_slice = self.collision_pairs_staging.slice(..pairs_size);
        pairs_slice.map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::Wait).unwrap();

        let pairs_view = pairs_slice.get_mapped_range();
        let pairs_data = bytemuck::cast_slice::<u8, CollisionPair>(&pairs_view);
        let collision_pairs = pairs_data[..collision_count.min(MAX_COLLISIONS) as usize].to_vec();
        drop(pairs_view);
        self.collision_pairs_staging.unmap();

        Ok(collision_pairs)
    }
}
