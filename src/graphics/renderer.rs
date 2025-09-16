use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window::Window};

use crate::graphics::{
    BloomRenderer, CameraUniform, CollisionCompute, Frustum, InstancedLineRenderer, ParticleSystem,
    Primitive, PrimitiveType, Projection, ThirdPersonCamera, Vertex, constants::*,
    is_visible_sphere, line_batch::ReusableLineBatch, primitive_cache::PrimitiveCache,
};

pub struct GraphicsEngine {
    surface: wgpu::Surface<'static>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    line_renderer: InstancedLineRenderer,
    bloom_renderer: BloomRenderer,
    collision_compute: CollisionCompute,
    camera: ThirdPersonCamera,
    projection: Projection,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
    frame_count: u32,
    primitive_cache: PrimitiveCache,
    line_batch: ReusableLineBatch,
    particles: ParticleSystem,
}

impl GraphicsEngine {
    pub async fn new(window: Arc<Window>) -> Result<Self, Box<dyn std::error::Error>> {
        let size = window.inner_size();

        // Create instance
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface
        let surface = instance.create_surface(window)?;

        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find suitable adapter");

        // Request device
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Render Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: Default::default(),
            })
            .await?;

        // Wrap in Arc for sharing
        let device = Arc::new(device);
        let queue: Arc<wgpu::Queue> = Arc::new(queue);

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        // Choose best present mode for high refresh rate displays
        let present_mode = if surface_caps
            .present_modes
            .contains(&wgpu::PresentMode::Mailbox)
        {
            wgpu::PresentMode::Mailbox // Triple buffering - tear-free + low latency for high refresh
        } else if surface_caps
            .present_modes
            .contains(&wgpu::PresentMode::Fifo)
        {
            wgpu::PresentMode::Fifo // Standard VSync fallback
        } else {
            surface_caps.present_modes[0] // Last resort fallback
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/basic.wgsl").into()),
        });

        // Setup camera
        let camera = ThirdPersonCamera::new();
        let projection = Projection::new(
            size.width,
            size.height,
            90.0_f32.to_radians(),
            0.1,
            1000.0, // 10x increase in rendering distance (was 100.0)
        );
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        // Create depth buffer
        let depth_format = wgpu::TextureFormat::Depth32Float;
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: depth_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create pipeline
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // No culling for wireframe
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        // Create instanced line renderer
        let line_renderer = InstancedLineRenderer::new(
            device.clone(),
            queue.clone(),
            &camera_bind_group_layout,
            surface_format,
        );

        // Create bloom post-processing renderer
        let bloom_renderer = BloomRenderer::new(
            device.clone(),
            queue.clone(),
            surface_format,
            size.width,
            size.height,
        );

        // Create collision compute system
        let collision_compute = CollisionCompute::new(&device);

        // Create particle system (without physics buffers for now - they'll be set later)
        let particles = ParticleSystem::new(&device, &camera_bind_group_layout, None, None);

        println!("Graphics engine initialized:");
        println!("- Surface format: {:?}", surface_format);
        println!(
            "- Present mode: {:?} ({})",
            present_mode,
            match present_mode {
                wgpu::PresentMode::Mailbox => "Triple buffering - optimal for high refresh",
                wgpu::PresentMode::Fifo => "VSync - locked to refresh rate",
                wgpu::PresentMode::Immediate => "No VSync - may tear",
                _ => "Other present mode",
            }
        );
        println!("- Bloom post-processing: DISABLED");
        println!("- Depth testing: ENABLED for proper Z-ordering");

        Ok(Self {
            surface,
            device: device.clone(),
            queue: queue.clone(),
            config,
            size,
            render_pipeline,
            line_renderer,
            bloom_renderer,
            collision_compute,
            camera,
            projection,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            depth_texture,
            depth_view,
            frame_count: 0,
            primitive_cache: PrimitiveCache::new(),
            line_batch: ReusableLineBatch::new(1000), // Estimate 1000 primitives per frame
            particles,
        })
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            self.projection.resize(new_size.width, new_size.height);
            self.camera_uniform
                .update_view_proj(&self.camera, &self.projection);
            self.queue.write_buffer(
                &self.camera_buffer,
                0,
                bytemuck::cast_slice(&[self.camera_uniform]),
            );

            // Resize bloom renderer
            self.bloom_renderer
                .resize(new_size.width, new_size.height, self.config.format);

            // Recreate depth buffer for new size
            let depth_format = wgpu::TextureFormat::Depth32Float;
            self.depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Depth Texture"),
                size: wgpu::Extent3d {
                    width: new_size.width,
                    height: new_size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: depth_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            self.depth_view = self
                .depth_texture
                .create_view(&wgpu::TextureViewDescriptor::default());
        }
    }

    pub fn update_camera(&mut self, target_pos: nalgebra::Point3<f32>) {
        self.camera.update(target_pos);
        self.camera_uniform
            .update_view_proj(&self.camera, &self.projection);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    pub fn camera(&self) -> &ThirdPersonCamera {
        &self.camera
    }

    pub fn camera_mut(&mut self) -> &mut ThirdPersonCamera {
        &mut self.camera
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn device_arc(&self) -> Arc<wgpu::Device> {
        Arc::clone(&self.device)
    }

    pub fn queue_arc(&self) -> Arc<wgpu::Queue> {
        Arc::clone(&self.queue)
    }

    pub fn collision_compute(&mut self) -> &mut CollisionCompute {
        &mut self.collision_compute
    }

    /// Dispatch collision detection and return results
    pub fn dispatch_and_read_collisions(
        &mut self,
    ) -> Result<Vec<crate::graphics::CollisionPair>, Box<dyn std::error::Error>> {
        // Dispatch collision detection
        self.collision_compute
            .dispatch_collision_detection(&self.device, &self.queue)?;

        // Read results synchronously
        self.collision_compute
            .read_collision_results_sync(&self.device, &self.queue)
    }

    pub fn render(&mut self, primitives: &[Primitive]) -> Result<(), wgpu::SurfaceError> {
        self.frame_count = self.frame_count.wrapping_add(1);
        let output = self.surface.get_current_texture()?;
        let final_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Step 1: Render scene to bloom's scene texture (for post-processing)
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Scene Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &final_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0, // Pure black background for space/Geometry Wars aesthetic
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            // Render wireframes using instanced line renderer
            self.render_wireframes_instanced(&mut render_pass, primitives);

            // Render particles
            self.particles
                .render(&mut render_pass, &self.camera_bind_group);
        }

        // Step 2: Bloom post-processing disabled - rendering directly to final surface

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Get approximate bounding radius for primitive types
    fn get_primitive_bounding_radius(primitive_type: PrimitiveType) -> f32 {
        match primitive_type {
            PrimitiveType::Cube => CUBE_BOUNDING_RADIUS,
            PrimitiveType::Sphere => SPHERE_BOUNDING_RADIUS,
            PrimitiveType::Pyramid => PYRAMID_BOUNDING_RADIUS,
            PrimitiveType::Tetrahedron => TETRAHEDRON_BOUNDING_RADIUS,
            PrimitiveType::Cone => CONE_BOUNDING_RADIUS,
            PrimitiveType::Cylinder => CYLINDER_BOUNDING_RADIUS,
            PrimitiveType::Octahedron => OCTAHEDRON_BOUNDING_RADIUS,
            PrimitiveType::Torus => TORUS_BOUNDING_RADIUS,
            PrimitiveType::Ellipsoid => ELLIPSOID_BOUNDING_RADIUS,
            PrimitiveType::Icosahedron => ICOSAHEDRON_BOUNDING_RADIUS,
            PrimitiveType::Dodecahedron => DODECAHEDRON_BOUNDING_RADIUS,
            PrimitiveType::Capsule => CAPSULE_BOUNDING_RADIUS,
            PrimitiveType::Plane => PLANE_BOUNDING_RADIUS,
            PrimitiveType::Hemisphere => HEMISPHERE_BOUNDING_RADIUS,
            // 2D Primitives
            PrimitiveType::Circle2D => CIRCLE2D_BOUNDING_RADIUS,
            PrimitiveType::Square2D => SQUARE2D_BOUNDING_RADIUS,
            PrimitiveType::Triangle2D => TRIANGLE2D_BOUNDING_RADIUS,
            PrimitiveType::Pentagon2D => PENTAGON2D_BOUNDING_RADIUS,
            PrimitiveType::Hexagon2D => HEXAGON2D_BOUNDING_RADIUS,
            PrimitiveType::Diamond2D => DIAMOND2D_BOUNDING_RADIUS,
            PrimitiveType::Cross2D => CROSS2D_BOUNDING_RADIUS,
            PrimitiveType::Star2D => STAR2D_BOUNDING_RADIUS,
            PrimitiveType::Arrow2D => ARROW2D_BOUNDING_RADIUS,
        }
    }

    /// Render wireframes using instanced line rendering (Pure ECS)
    fn render_wireframes_instanced(
        &mut self,
        render_pass: &mut wgpu::RenderPass,
        primitives: &[Primitive],
    ) {
        // Start a new batch for this frame
        self.line_batch.start_frame();

        // Create frustum for culling from current camera view-projection matrix
        let view_proj_matrix = self.projection.calc_matrix() * self.camera.calc_matrix();
        let frustum = Frustum::from_view_proj_matrix(&view_proj_matrix);

        // Performance counters
        let mut _total_entities = 0;
        let mut _culled_entities = 0;

        // Get all entities with both transform and render components
        // Use cached render data - only arena lookups needed (just ~2 per frame)
        // Collect all visible primitives into the batch
        for data in primitives {
            _total_entities += 1;

            // Create transform struct for frustum culling
            let transform = crate::graphics::Transform {
                position: data.position,
                rotation: data.rotation,
                scale: data.scale,
            };

            // Frustum culling - skip entities outside camera view
            let bounding_radius = Self::get_primitive_bounding_radius(data.primitive_type);
            if !is_visible_sphere(&frustum, &transform, bounding_radius) {
                _culled_entities += 1;
                continue;
            }

            // Use default line thickness for all primitives
            let thickness = DEFAULT_LINE_THICKNESS;

            // Add primitive to batch with full rotation support
            self.line_batch.add_primitive_with_rotation(
                data.primitive_type,
                data.position,
                data.rotation,
                data.scale,
                data.color,
                thickness,
            );
        }

        // Generate all lines using optimized systems with full rotation support
        // Lines are now batched by primitive type for better cache locality
        let mut lines = self.line_batch.finish_frame(&self.primitive_cache);

        // Arena wireframes now use the standard rendering pipeline above
        // They are handled like any other entity with RenderComponent + ArenaMarkerComponent

        // Ensure we don't exceed the instanced renderer's capacity to prevent buffer overflow
        let max_capacity = self.line_renderer.max_instance_count();
        if lines.len() > max_capacity {
            println!(
                "Warning: {} lines exceed capacity {}, truncating",
                lines.len(),
                max_capacity
            );
            lines.truncate(max_capacity);
        }

        // Update GPU buffers BEFORE starting the render pass to prevent stalls
        self.line_renderer.update_buffers(&lines);

        // Always render - even empty to maintain consistent frame timing
        self.line_renderer
            .render_lines(render_pass, &self.camera_bind_group, &lines);
    }

    /// Spawn particles with full collision event data
    pub fn spawn_particles(
        &mut self,
        position: crate::graphics::Vec3,
        velocity: crate::graphics::Vec3,
        count: u32,
        lifetime: f32,
        color: crate::graphics::Color,
    ) {
        self.particles
            .spawn_particles(position, velocity, count, lifetime, color);
    }

    /// Spawn particles at the given position with default values (backwards compatibility)
    pub fn spawn_particles_simple(&mut self, position: crate::graphics::Vec3) {
        self.particles.spawn_particles_simple(position);
    }

    /// Set physics buffers for gravitational particle effects
    pub fn set_physics_buffers(
        &mut self,
        gravitational_bodies_buffer: Option<&wgpu::Buffer>,
        body_count_buffer: Option<&wgpu::Buffer>,
    ) {
        self.particles.set_physics_buffers(
            &self.device,
            gravitational_bodies_buffer,
            body_count_buffer,
        );
    }

    /// Update particle system (call before render)
    pub fn update_particles(&mut self, delta_time: f32) {
        self.particles.update(&self.device, &self.queue, delta_time);
    }
}
