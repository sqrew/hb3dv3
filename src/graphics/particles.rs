use crate::graphics::Vec3;
use crate::scene::physics::GravitationalBody;
use wgpu::util::DeviceExt;

/// Maximum number of particles in the system
const MAX_PARTICLES: u32 = 8192;

/// Maximum spawn requests per frame
const MAX_SPAWN_REQUESTS: u32 = 32768;

/// Particles per collision effect
const PARTICLES_PER_COLLISION: u32 = 32;

/// GPU particle data structure
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuParticle {
    position: [f32; 3],
    _padding1: f32,
    velocity: [f32; 3],
    _padding2: f32,
    life: f32,
    max_life: f32,
    color: [f32; 4], // RGBA
}

impl Default for GpuParticle {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            _padding1: 0.0,
            velocity: [0.0; 3],
            _padding2: 0.0,
            life: 0.0,
            max_life: 0.0,
            color: [0.0; 4],
        }
    }
}

/// Spawn request for particles
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct SpawnRequest {
    position: [f32; 3],
    count: u32,
    velocity: [f32; 3],
    lifetime: f32,
    color: [f32; 4],    // RGBA
    _padding: [f32; 4], // Ensure 16-byte alignment
}

/// GPU particle system
pub struct ParticleSystem {
    // Compute pipelines
    update_pipeline: wgpu::ComputePipeline,
    spawn_pipeline: wgpu::ComputePipeline,

    // Render pipeline for points
    render_pipeline: wgpu::RenderPipeline,

    // GPU buffers
    particle_buffer: wgpu::Buffer,
    alive_count_buffer: wgpu::Buffer,
    spawn_queue_buffer: wgpu::Buffer,
    spawn_count_buffer: wgpu::Buffer,
    delta_time_buffer: wgpu::Buffer,

    // Bind groups
    compute_bind_group: wgpu::BindGroup,
    render_bind_group: wgpu::BindGroup,

    // CPU-side spawn queue
    spawn_queue: Vec<SpawnRequest>,
}

impl ParticleSystem {
    pub fn new(
        device: &wgpu::Device,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        gravitational_bodies_buffer: Option<&wgpu::Buffer>,
        body_count_buffer: Option<&wgpu::Buffer>,
    ) -> Self {
        // Load shaders
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Particle Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/particles.wgsl").into()),
        });

        // Create buffers
        let particle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Buffer"),
            size: (MAX_PARTICLES as usize * std::mem::size_of::<GpuParticle>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let alive_count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Alive Count Buffer"),
            contents: bytemuck::cast_slice(&[0u32]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let spawn_queue_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Spawn Queue Buffer"),
            size: (MAX_SPAWN_REQUESTS as usize * std::mem::size_of::<SpawnRequest>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let spawn_count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Spawn Count Buffer"),
            contents: bytemuck::cast_slice(&[0u32]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let delta_time_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Delta Time Buffer"),
            contents: bytemuck::cast_slice(&[0.016f32]), // Default to 60 FPS
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create compute bind group layout
        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Particle Compute Bind Group Layout"),
                entries: &[
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
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Gravitational bodies buffer (binding 5)
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Body count buffer (binding 6)
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        // Create dummy buffers if physics buffers not provided
        let dummy_gravitational_bodies_buffer;
        let dummy_body_count_buffer;

        let (grav_buffer, count_buffer) = if let (Some(grav_buf), Some(count_buf)) =
            (gravitational_bodies_buffer, body_count_buffer) {
            (grav_buf, count_buf)
        } else {
            // Create dummy buffers
            dummy_gravitational_bodies_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Dummy Gravitational Bodies Buffer"),
                size: std::mem::size_of::<GravitationalBody>() as u64,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            });
            dummy_body_count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Dummy Body Count Buffer"),
                contents: bytemuck::cast_slice(&[0u32]),
                usage: wgpu::BufferUsages::STORAGE,
            });
            (&dummy_gravitational_bodies_buffer, &dummy_body_count_buffer)
        };

        // Create compute bind group
        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Particle Compute Bind Group"),
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: particle_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: alive_count_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: spawn_queue_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: spawn_count_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: delta_time_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: grav_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: count_buffer.as_entire_binding(),
                },
            ],
        });

        // Create compute pipelines
        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Particle Compute Pipeline Layout"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        let update_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Particle Update Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &shader,
            entry_point: Some("update_particles"),
            cache: None,
            compilation_options: Default::default(),
        });

        let spawn_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Particle Spawn Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &shader,
            entry_point: Some("spawn_particles"),
            cache: None,
            compilation_options: Default::default(),
        });

        // Create render bind group layout (only needs particle buffer)
        let render_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Particle Render Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        // Create render bind group
        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Particle Render Bind Group"),
            layout: &render_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: particle_buffer.as_entire_binding(),
            }],
        });

        // Create render pipeline layout with proper ordering
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Particle Render Pipeline Layout"),
                bind_group_layouts: &[camera_bind_group_layout, &render_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Particle Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::PointList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            update_pipeline,
            spawn_pipeline,
            render_pipeline,
            particle_buffer,
            alive_count_buffer,
            spawn_queue_buffer,
            spawn_count_buffer,
            delta_time_buffer,
            compute_bind_group,
            render_bind_group,
            spawn_queue: Vec::new(),
        }
    }

    /// Add a particle spawn request with full collision event data
    pub fn spawn_particles(
        &mut self,
        position: Vec3,
        velocity: Vec3,
        count: u32,
        lifetime: f32,
        color: crate::graphics::Color,
    ) {
        if self.spawn_queue.len() < MAX_SPAWN_REQUESTS as usize {
            self.spawn_queue.push(SpawnRequest {
                position: [position.x, position.y, position.z],
                count,
                velocity: [velocity.x, velocity.y, velocity.z],
                lifetime,
                color: [color.r, color.g, color.b, color.a],
                _padding: [0.0; 4],
            });
        }
    }

    /// Add a particle spawn request at the given position with default values (backwards compatibility)
    pub fn spawn_particles_simple(&mut self, position: Vec3) {
        self.spawn_particles(
            position,
            Vec3::new(0.0, 1.0, 0.0), // Default upward velocity
            PARTICLES_PER_COLLISION,
            0.8,
            crate::graphics::Color::new(1.0, 0.6, 0.2, 1.0), // Orange sparks
        );
    }

    /// Set physics buffers for gravitational effects (recreates bind group)
    pub fn set_physics_buffers(
        &mut self,
        device: &wgpu::Device,
        gravitational_bodies_buffer: Option<&wgpu::Buffer>,
        body_count_buffer: Option<&wgpu::Buffer>,
    ) {
        // We need to recreate the compute bind group with the new physics buffers
        // For now, this is a simple solution that works but could be optimized

        // Create dummy buffers if physics buffers not provided
        let dummy_gravitational_bodies_buffer;
        let dummy_body_count_buffer;

        let (grav_buffer, count_buffer) = if let (Some(grav_buf), Some(count_buf)) =
            (gravitational_bodies_buffer, body_count_buffer) {
            (grav_buf, count_buf)
        } else {
            // Create dummy buffers
            dummy_gravitational_bodies_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Dummy Gravitational Bodies Buffer"),
                size: std::mem::size_of::<GravitationalBody>() as u64,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            });
            dummy_body_count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Dummy Body Count Buffer"),
                contents: bytemuck::cast_slice(&[0u32]),
                usage: wgpu::BufferUsages::STORAGE,
            });
            (&dummy_gravitational_bodies_buffer, &dummy_body_count_buffer)
        };

        // Create the bind group layout (we need this again)
        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Particle Compute Bind Group Layout"),
                entries: &[
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
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Gravitational bodies buffer (binding 5)
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Body count buffer (binding 6)
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        // Recreate the compute bind group with physics buffers
        self.compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Particle Compute Bind Group"),
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.particle_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.alive_count_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.spawn_queue_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.spawn_count_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: self.delta_time_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: grav_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: count_buffer.as_entire_binding(),
                },
            ],
        });
    }

    /// Update particle system on GPU
    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, delta_time: f32) {
        // Aggressively cap spawn queue at start of update
        if self.spawn_queue.len() > MAX_SPAWN_REQUESTS as usize {
            self.spawn_queue.truncate(MAX_SPAWN_REQUESTS as usize);
        }
        
        // Upload spawn requests if any
        if !self.spawn_queue.is_empty() {
            // Hard cap to prevent any overflow - be very conservative
            let max_safe_requests = 2097152 / std::mem::size_of::<SpawnRequest>();
            if self.spawn_queue.len() > max_safe_requests {
                self.spawn_queue.truncate(max_safe_requests);
            }
            
            let data_bytes = bytemuck::cast_slice::<SpawnRequest, u8>(&self.spawn_queue);
            if data_bytes.len() > 2097152 {
                // Emergency fallback - clear the queue entirely if still too big
                self.spawn_queue.clear();
            } else {
                queue.write_buffer(
                    &self.spawn_queue_buffer,
                    0,
                    data_bytes,
                );
            }
            queue.write_buffer(
                &self.spawn_count_buffer,
                0,
                bytemuck::cast_slice(&[self.spawn_queue.len() as u32]),
            );
        } else {
            queue.write_buffer(&self.spawn_count_buffer, 0, bytemuck::cast_slice(&[0u32]));
        }

        // Upload delta time
        queue.write_buffer(
            &self.delta_time_buffer,
            0,
            bytemuck::cast_slice(&[delta_time]),
        );

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Particle Update Command Encoder"),
        });

        // Spawn new particles
        if !self.spawn_queue.is_empty() {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Particle Spawn Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.spawn_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
            let workgroups = (self.spawn_queue.len() as u32 + 63) / 64;
            compute_pass.dispatch_workgroups(workgroups, 1, 1);
        }

        // Update existing particles
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Particle Update Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.update_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
            let workgroups = (MAX_PARTICLES + 63) / 64;
            compute_pass.dispatch_workgroups(workgroups, 1, 1);
        }

        queue.submit(Some(encoder.finish()));

        // Clear spawn queue after processing
        self.spawn_queue.clear();
    }

    /// Render particles
    pub fn render(&self, render_pass: &mut wgpu::RenderPass, camera_bind_group: &wgpu::BindGroup) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.set_bind_group(1, &self.render_bind_group, &[]);
        render_pass.draw(0..1, 0..MAX_PARTICLES);
    }
}
