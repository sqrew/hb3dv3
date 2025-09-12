use crate::graphics::Vec3;
use wgpu::util::DeviceExt;

/// Maximum number of gravitational bodies in the system
const MAX_GRAVITATIONAL_BODIES: u32 = 32;

/// Maximum number of gravity-affected objects
const MAX_AFFECTED_OBJECTS: u32 = 65536;

/// Gravitational constant (scaled for game physics)
pub const GRAVITATIONAL_CONSTANT: f32 = 6.674e-1; // Much stronger than real physics for gameplay

/// Trait for objects that can be affected by gravitational forces
pub trait GravityAffected {
    fn position(&self) -> Vec3;
    fn mass(&self) -> f32;
    fn apply_force(&mut self, force: Vec3);
}

/// A large gravitational body (planet, asteroid, space station)
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GravitationalBody {
    pub position: [f32; 3],
    pub mass: f32,
    pub velocity: [f32; 3],
    pub radius: f32, // For collision detection
}

impl Default for GravitationalBody {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            mass: 0.0,
            velocity: [0.0; 3],
            radius: 0.0,
        }
    }
}

/// GPU data for gravity-affected objects
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuAffectedObject {
    position: [f32; 3],
    mass: f32,
    // Force output (written by compute shader)
    force: [f32; 3],
    _padding: f32,
}

impl Default for GpuAffectedObject {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            mass: 0.0,
            force: [0.0; 3],
            _padding: 0.0,
        }
    }
}

/// Physics system managing gravitational interactions
pub struct PhysicsManager {
    // Large gravitational bodies (planets, asteroids, etc.)
    gravitational_bodies: Vec<GravitationalBody>,

    // GPU resources (None until initialized)
    gpu_resources: Option<PhysicsGpuResources>,

    // CPU-side affected objects cache
    affected_objects_cache: Vec<GpuAffectedObject>,
}

struct PhysicsGpuResources {
    // Compute pipeline for force calculations
    gravity_compute_pipeline: wgpu::ComputePipeline,
    nbody_compute_pipeline: wgpu::ComputePipeline,

    // GPU buffers
    gravitational_bodies_buffer: wgpu::Buffer,
    affected_objects_buffer: wgpu::Buffer,
    body_count_buffer: wgpu::Buffer,
    affected_count_buffer: wgpu::Buffer,
    delta_time_buffer: wgpu::Buffer,

    // Staging buffer for GPU readback
    staging_buffer: wgpu::Buffer,

    // Bind groups
    gravity_compute_bind_group: wgpu::BindGroup,
    nbody_compute_bind_group: wgpu::BindGroup,
}

impl PhysicsManager {
    pub fn new() -> Self {
        Self {
            gravitational_bodies: Vec::new(),
            gpu_resources: None,
            affected_objects_cache: Vec::new(),
        }
    }

    pub fn initialize_gpu(&mut self, device: &wgpu::Device) {
        self.gpu_resources = Some(Self::create_gpu_resources(device));
    }

    fn create_gpu_resources(device: &wgpu::Device) -> PhysicsGpuResources {
        // Load compute shaders
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Physics Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../graphics/shaders/physics.wgsl").into(),
            ),
        });

        // Create buffers
        let gravitational_bodies_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Gravitational Bodies Buffer"),
            size: (MAX_GRAVITATIONAL_BODIES as usize * std::mem::size_of::<GravitationalBody>())
                as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let affected_objects_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Affected Objects Buffer"),
            size: (MAX_AFFECTED_OBJECTS as usize * std::mem::size_of::<GpuAffectedObject>()) as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let body_count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Body Count Buffer"),
            contents: bytemuck::cast_slice(&[0u32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let affected_count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Affected Count Buffer"),
            contents: bytemuck::cast_slice(&[0u32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let delta_time_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Physics Delta Time Buffer"),
            contents: bytemuck::cast_slice(&[0.016f32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create staging buffer for GPU-to-CPU readback
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Physics Staging Buffer"),
            size: (MAX_AFFECTED_OBJECTS as usize * std::mem::size_of::<GpuAffectedObject>()) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Create bind group layouts
        let gravity_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Gravity Compute Bind Group Layout"),
                entries: &[
                    // Gravitational bodies (read-only)
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
                    // Affected objects (read-write for force output)
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Body count
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Affected count
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let nbody_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("N-Body Compute Bind Group Layout"),
                entries: &[
                    // Gravitational bodies (read-write for position/velocity updates)
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Body count
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Delta time
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        // Create bind groups
        let gravity_compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Gravity Compute Bind Group"),
            layout: &gravity_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: gravitational_bodies_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: affected_objects_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: body_count_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: affected_count_buffer.as_entire_binding(),
                },
            ],
        });

        let nbody_compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("N-Body Compute Bind Group"),
            layout: &nbody_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: gravitational_bodies_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: body_count_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: delta_time_buffer.as_entire_binding(),
                },
            ],
        });

        // Create compute pipelines
        let gravity_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Gravity Compute Pipeline Layout"),
                bind_group_layouts: &[&gravity_bind_group_layout],
                push_constant_ranges: &[],
            });

        let nbody_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("N-Body Compute Pipeline Layout"),
                bind_group_layouts: &[&nbody_bind_group_layout],
                push_constant_ranges: &[],
            });

        let gravity_compute_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Gravity Force Compute Pipeline"),
                layout: Some(&gravity_pipeline_layout),
                module: &shader,
                entry_point: Some("compute_gravity_forces"),
                cache: None,
                compilation_options: Default::default(),
            });

        let nbody_compute_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("N-Body Simulation Pipeline"),
                layout: Some(&nbody_pipeline_layout),
                module: &shader,
                entry_point: Some("update_gravitational_bodies"),
                cache: None,
                compilation_options: Default::default(),
            });

        PhysicsGpuResources {
            gravity_compute_pipeline,
            nbody_compute_pipeline,
            gravitational_bodies_buffer,
            affected_objects_buffer,
            body_count_buffer,
            affected_count_buffer,
            delta_time_buffer,
            staging_buffer,
            gravity_compute_bind_group,
            nbody_compute_bind_group,
        }
    }

    /// Add a gravitational body to the system
    pub fn add_gravitational_body(
        &mut self,
        position: Vec3,
        mass: f32,
        velocity: Vec3,
        radius: f32,
    ) -> usize {
        if self.gravitational_bodies.len() < MAX_GRAVITATIONAL_BODIES as usize {
            let body = GravitationalBody {
                position: [position.x, position.y, position.z],
                mass,
                velocity: [velocity.x, velocity.y, velocity.z],
                radius,
            };
            self.gravitational_bodies.push(body);
            self.gravitational_bodies.len() - 1
        } else {
            panic!("Maximum gravitational bodies exceeded!");
        }
    }

    /// Update gravitational body position/velocity manually
    pub fn update_gravitational_body(&mut self, index: usize, position: Vec3, velocity: Vec3) {
        if let Some(body) = self.gravitational_bodies.get_mut(index) {
            body.position = [position.x, position.y, position.z];
            body.velocity = [velocity.x, velocity.y, velocity.z];
        }
    }

    /// Apply gravitational forces to all affected objects in one efficient GPU call
    pub fn update_gravity_batch(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        affected_objects: &mut [&mut dyn GravityAffected],
        delta_time: f32,
    ) {
        if self.gravitational_bodies.is_empty() || affected_objects.is_empty() {
            return;
        }

        let gpu = match &self.gpu_resources {
            Some(gpu) => gpu,
            None => {
                // Fallback to CPU physics if GPU not initialized
                self.update_gravity_cpu_batch(affected_objects, delta_time);
                return;
            }
        };

        // Prepare affected objects data for GPU
        self.affected_objects_cache.clear();
        self.affected_objects_cache
            .extend(affected_objects.iter().map(|obj| {
                let pos = obj.position();
                GpuAffectedObject {
                    position: [pos.x, pos.y, pos.z],
                    mass: obj.mass(),
                    force: [0.0; 3], // Will be computed by GPU
                    _padding: 0.0,
                }
            }));

        // Upload data to GPU
        queue.write_buffer(
            &gpu.gravitational_bodies_buffer,
            0,
            bytemuck::cast_slice(&self.gravitational_bodies),
        );

        queue.write_buffer(
            &gpu.affected_objects_buffer,
            0,
            bytemuck::cast_slice(&self.affected_objects_cache),
        );

        queue.write_buffer(
            &gpu.body_count_buffer,
            0,
            bytemuck::cast_slice(&[self.gravitational_bodies.len() as u32]),
        );

        queue.write_buffer(
            &gpu.affected_count_buffer,
            0,
            bytemuck::cast_slice(&[affected_objects.len() as u32]),
        );

        queue.write_buffer(
            &gpu.delta_time_buffer,
            0,
            bytemuck::cast_slice(&[delta_time]),
        );

        // Execute compute shader
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Physics Compute Encoder"),
        });

        // First update N-body interactions between large bodies
        if self.gravitational_bodies.len() > 1 {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("N-Body Physics Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&gpu.nbody_compute_pipeline);
            compute_pass.set_bind_group(0, &gpu.nbody_compute_bind_group, &[]);
            let workgroups = (self.gravitational_bodies.len() as u32 + 63) / 64;
            compute_pass.dispatch_workgroups(workgroups, 1, 1);
        }

        // Then compute forces on affected objects
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Gravity Force Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&gpu.gravity_compute_pipeline);
            compute_pass.set_bind_group(0, &gpu.gravity_compute_bind_group, &[]);
            let workgroups = (affected_objects.len() as u32 + 63) / 64;
            compute_pass.dispatch_workgroups(workgroups, 1, 1);
        }

        // Copy computed forces from GPU to staging buffer for readback
        encoder.copy_buffer_to_buffer(
            &gpu.affected_objects_buffer,
            0,
            &gpu.staging_buffer,
            0,
            (affected_objects.len() * std::mem::size_of::<GpuAffectedObject>()) as u64,
        );

        queue.submit(Some(encoder.finish()));

        // Map the staging buffer to read back the GPU computed forces
        let buffer_slice = gpu.staging_buffer.slice(..);

        // Request the buffer to be mapped for reading
        buffer_slice.map_async(wgpu::MapMode::Read, |result| {
            if result.is_err() {
                eprintln!("Failed to map physics staging buffer");
            }
        });

        // Wait for the mapping to complete (synchronous approach)
        device.poll(wgpu::PollType::Wait).unwrap();

        // Read the computed forces and apply them
        {
            let data = buffer_slice.get_mapped_range();
            let gpu_objects: &[GpuAffectedObject] = bytemuck::cast_slice(&data);

            // Apply the GPU-computed forces to our objects
            for (obj, gpu_obj) in affected_objects.iter_mut().zip(gpu_objects.iter()) {
                let force = Vec3::new(gpu_obj.force[0], gpu_obj.force[1], gpu_obj.force[2]);
                obj.apply_force(force);
            }
        }

        // Unmap the buffer
        gpu.staging_buffer.unmap();
    }

    /// Apply gravitational forces to all affected objects
    pub fn update_gravity<T: GravityAffected>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        affected_objects: &mut [T],
        delta_time: f32,
    ) {
        if self.gravitational_bodies.is_empty() || affected_objects.is_empty() {
            return;
        }

        let gpu = match &self.gpu_resources {
            Some(gpu) => gpu,
            None => {
                // Fallback to CPU physics if GPU not initialized
                self.update_gravity_cpu(affected_objects, delta_time);
                return;
            }
        };

        // Prepare affected objects data for GPU
        self.affected_objects_cache.clear();
        self.affected_objects_cache
            .extend(affected_objects.iter().map(|obj| {
                let pos = obj.position();
                GpuAffectedObject {
                    position: [pos.x, pos.y, pos.z],
                    mass: obj.mass(),
                    force: [0.0; 3], // Will be computed by GPU
                    _padding: 0.0,
                }
            }));

        // Upload data to GPU
        queue.write_buffer(
            &gpu.gravitational_bodies_buffer,
            0,
            bytemuck::cast_slice(&self.gravitational_bodies),
        );

        queue.write_buffer(
            &gpu.affected_objects_buffer,
            0,
            bytemuck::cast_slice(&self.affected_objects_cache),
        );

        queue.write_buffer(
            &gpu.body_count_buffer,
            0,
            bytemuck::cast_slice(&[self.gravitational_bodies.len() as u32]),
        );

        queue.write_buffer(
            &gpu.affected_count_buffer,
            0,
            bytemuck::cast_slice(&[affected_objects.len() as u32]),
        );

        queue.write_buffer(
            &gpu.delta_time_buffer,
            0,
            bytemuck::cast_slice(&[delta_time]),
        );

        // Execute compute shader
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Physics Compute Encoder"),
        });

        // First update N-body interactions between large bodies
        if self.gravitational_bodies.len() > 1 {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("N-Body Physics Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&gpu.nbody_compute_pipeline);
            compute_pass.set_bind_group(0, &gpu.nbody_compute_bind_group, &[]);
            let workgroups = (self.gravitational_bodies.len() as u32 + 63) / 64;
            compute_pass.dispatch_workgroups(workgroups, 1, 1);
        }

        // Then compute forces on affected objects
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Gravity Force Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&gpu.gravity_compute_pipeline);
            compute_pass.set_bind_group(0, &gpu.gravity_compute_bind_group, &[]);
            let workgroups = (affected_objects.len() as u32 + 63) / 64;
            compute_pass.dispatch_workgroups(workgroups, 1, 1);
        }

        // Copy computed forces from GPU to staging buffer for readback
        encoder.copy_buffer_to_buffer(
            &gpu.affected_objects_buffer,
            0,
            &gpu.staging_buffer,
            0,
            (affected_objects.len() * std::mem::size_of::<GpuAffectedObject>()) as u64,
        );

        queue.submit(Some(encoder.finish()));

        // Map the staging buffer to read back the GPU computed forces
        let buffer_slice = gpu.staging_buffer.slice(..);

        // Request the buffer to be mapped for reading
        buffer_slice.map_async(wgpu::MapMode::Read, |result| {
            if result.is_err() {
                eprintln!("Failed to map physics staging buffer");
            }
        });

        // Wait for the mapping to complete (synchronous approach)
        device.poll(wgpu::PollType::Wait).unwrap();

        // Read the computed forces and apply them
        {
            let data = buffer_slice.get_mapped_range();
            let gpu_objects: &[GpuAffectedObject] = bytemuck::cast_slice(&data);

            // Apply the GPU-computed forces to our objects
            for (obj, gpu_obj) in affected_objects.iter_mut().zip(gpu_objects.iter()) {
                let force = Vec3::new(gpu_obj.force[0], gpu_obj.force[1], gpu_obj.force[2]);
                obj.apply_force(force);
            }
        }

        // Unmap the buffer
        gpu.staging_buffer.unmap();
    }

    /// CPU batch fallback for gravity calculations
    fn update_gravity_cpu_batch(
        &self,
        affected_objects: &mut [&mut dyn GravityAffected],
        _delta_time: f32,
    ) {
        for obj in affected_objects.iter_mut() {
            let obj_pos = obj.position();
            let obj_mass = obj.mass();
            let mut total_force = Vec3::zeros();

            // Calculate force from each gravitational body
            for body in &self.gravitational_bodies {
                let body_pos = Vec3::new(body.position[0], body.position[1], body.position[2]);
                let displacement = body_pos - obj_pos;
                let distance_squared = displacement.magnitude_squared();

                if distance_squared < 0.01 {
                    continue;
                } // Avoid singularities

                let distance = distance_squared.sqrt();
                let force_magnitude =
                    (GRAVITATIONAL_CONSTANT * body.mass * obj_mass) / distance_squared;
                let force_direction = displacement / distance;

                total_force += force_direction * force_magnitude;
            }

            obj.apply_force(total_force);
        }
    }

    /// CPU fallback for gravity calculations
    fn update_gravity_cpu<T: GravityAffected>(&self, affected_objects: &mut [T], _delta_time: f32) {
        for obj in affected_objects.iter_mut() {
            let obj_pos = obj.position();
            let obj_mass = obj.mass();
            let mut total_force = Vec3::zeros();

            // Calculate force from each gravitational body
            for body in &self.gravitational_bodies {
                let body_pos = Vec3::new(body.position[0], body.position[1], body.position[2]);
                let displacement = body_pos - obj_pos;
                let distance_squared = displacement.magnitude_squared();

                if distance_squared < 0.01 {
                    continue;
                } // Avoid singularities

                let distance = distance_squared.sqrt();
                let force_magnitude =
                    (GRAVITATIONAL_CONSTANT * body.mass * obj_mass) / distance_squared;
                let force_direction = displacement / distance;

                total_force += force_direction * force_magnitude;
            }

            obj.apply_force(total_force);
        }
    }

    /// Get gravitational body data (for rendering or game logic)
    pub fn gravitational_bodies(&self) -> &[GravitationalBody] {
        &self.gravitational_bodies
    }

    /// Clear all gravitational bodies
    pub fn clear_gravitational_bodies(&mut self) {
        self.gravitational_bodies.clear();
    }

    /// Remove a gravitational body by index
    pub fn remove_gravitational_body(&mut self, index: usize) -> bool {
        if index < self.gravitational_bodies.len() {
            self.gravitational_bodies.remove(index);
            true
        } else {
            false
        }
    }
}
