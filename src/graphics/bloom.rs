//! Bloom post-processing system for neon glow effects
//!
//! This module implements a multi-pass bloom effect:
//! 1. Render scene to texture
//! 2. Extract bright areas above threshold
//! 3. Apply Gaussian blur (horizontal + vertical)
//! 4. Composite with original scene

use std::sync::Arc;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct BlurUniforms {
    direction: [f32; 2], // (1,0) for horizontal, (0,1) for vertical
    blur_strength: f32,  // Controls glow spread
    _padding: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CompositionUniforms {
    bloom_intensity: f32, // Controls overall glow strength
    bloom_radius: f32,    // Controls glow spread
    exposure: f32,        // HDR exposure adjustment
    _padding: f32,
}

pub struct BloomRenderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,

    // Scene render target
    scene_texture: wgpu::Texture,
    scene_texture_view: wgpu::TextureView,

    // Bloom render targets
    bright_texture: wgpu::Texture,
    bright_texture_view: wgpu::TextureView,
    blur_texture_1: wgpu::Texture,
    blur_texture_1_view: wgpu::TextureView,
    blur_texture_2: wgpu::Texture,
    blur_texture_2_view: wgpu::TextureView,

    // Render pipelines
    brightness_pipeline: wgpu::RenderPipeline,
    blur_pipeline: wgpu::RenderPipeline,
    composition_pipeline: wgpu::RenderPipeline,

    // Bind groups
    brightness_bind_group: wgpu::BindGroup,
    blur_horizontal_bind_group: wgpu::BindGroup,
    blur_vertical_bind_group: wgpu::BindGroup,
    composition_bind_group: wgpu::BindGroup,

    // Uniform buffers
    blur_uniform_buffer: wgpu::Buffer,
    composition_uniform_buffer: wgpu::Buffer,
    blur_bind_group: wgpu::BindGroup,
    composition_settings_bind_group: wgpu::BindGroup,

    // Sampler
    sampler: wgpu::Sampler,

    width: u32,
    height: u32,
}

impl BloomRenderer {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        surface_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Bloom Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Create textures
        let scene_texture =
            Self::create_render_texture(&device, width, height, surface_format, "Scene");
        let scene_texture_view = scene_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bright_texture =
            Self::create_render_texture(&device, width / 2, height / 2, surface_format, "Bright");
        let bright_texture_view =
            bright_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let blur_texture_1 =
            Self::create_render_texture(&device, width / 2, height / 2, surface_format, "Blur1");
        let blur_texture_1_view =
            blur_texture_1.create_view(&wgpu::TextureViewDescriptor::default());

        let blur_texture_2 =
            Self::create_render_texture(&device, width / 2, height / 2, surface_format, "Blur2");
        let blur_texture_2_view =
            blur_texture_2.create_view(&wgpu::TextureViewDescriptor::default());

        // Load shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Bloom Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/bloom.wgsl").into()),
        });

        // Create bind group layouts
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Bloom Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let composition_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Composition Bind Group Layout"),
                entries: &[
                    // Original texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // Bloom texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let uniforms_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Uniforms Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        // Create uniform buffers
        let blur_uniforms = BlurUniforms {
            direction: [1.0, 0.0], // Will be updated per pass
            blur_strength: 0.6,    // Slightly more diffuse for final look
            _padding: 0.0,
        };

        let blur_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Blur Uniform Buffer"),
            contents: bytemuck::cast_slice(&[blur_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let composition_uniforms = CompositionUniforms {
            bloom_intensity: 0.5, // Toned down neon effect
            bloom_radius: 1.0,
            exposure: 1.1, // Slight exposure boost for neon brightness
            _padding: 0.0,
        };

        let composition_uniform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Composition Uniform Buffer"),
                contents: bytemuck::cast_slice(&[composition_uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        // Create bind groups
        let brightness_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Brightness Bind Group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&scene_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let blur_horizontal_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Blur Horizontal Bind Group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&bright_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let blur_vertical_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Blur Vertical Bind Group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&blur_texture_1_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let composition_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Composition Bind Group"),
            layout: &composition_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&scene_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&blur_texture_2_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let blur_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Blur Settings Bind Group"),
            layout: &uniforms_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: blur_uniform_buffer.as_entire_binding(),
            }],
        });

        let composition_settings_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Composition Settings Bind Group"),
                layout: &uniforms_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: composition_uniform_buffer.as_entire_binding(),
                }],
            });

        // Create render pipelines
        let brightness_pipeline = Self::create_fullscreen_pipeline(
            &device,
            &shader,
            "fs_brightness_extract",
            surface_format,
            &texture_bind_group_layout,
            None,
            "Brightness Pipeline",
        );

        let blur_pipeline = Self::create_fullscreen_pipeline(
            &device,
            &shader,
            "fs_blur",
            surface_format,
            &texture_bind_group_layout,
            Some(&uniforms_bind_group_layout),
            "Blur Pipeline",
        );

        let composition_pipeline = Self::create_fullscreen_pipeline(
            &device,
            &shader,
            "fs_composition",
            surface_format,
            &composition_bind_group_layout,
            Some(&uniforms_bind_group_layout),
            "Composition Pipeline",
        );

        Self {
            device,
            queue,
            scene_texture,
            scene_texture_view,
            bright_texture,
            bright_texture_view,
            blur_texture_1,
            blur_texture_1_view,
            blur_texture_2,
            blur_texture_2_view,
            brightness_pipeline,
            blur_pipeline,
            composition_pipeline,
            brightness_bind_group,
            blur_horizontal_bind_group,
            blur_vertical_bind_group,
            composition_bind_group,
            blur_uniform_buffer,
            composition_uniform_buffer,
            blur_bind_group,
            composition_settings_bind_group,
            sampler,
            width,
            height,
        }
    }

    fn create_render_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        label: &str,
    ) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
    }

    fn create_fullscreen_pipeline(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        fragment_entry: &str,
        format: wgpu::TextureFormat,
        bind_group_layout_0: &wgpu::BindGroupLayout,
        bind_group_layout_1: Option<&wgpu::BindGroupLayout>,
        label: &str,
    ) -> wgpu::RenderPipeline {
        let mut layouts = vec![bind_group_layout_0];
        if let Some(layout_1) = bind_group_layout_1 {
            layouts.push(layout_1);
        }

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{} Layout", label)),
            bind_group_layouts: &layouts,
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_fullscreen"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some(fragment_entry),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        })
    }

    pub fn get_scene_texture_view(&self) -> &wgpu::TextureView {
        &self.scene_texture_view
    }

    pub fn render_bloom(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        final_target: &wgpu::TextureView,
    ) {
        // Pass 1: Extract bright areas
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Brightness Extract Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.bright_texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.brightness_pipeline);
            render_pass.set_bind_group(0, &self.brightness_bind_group, &[]);
            render_pass.draw(0..3, 0..1); // Fullscreen triangle
        }

        // Pass 2: Horizontal blur
        {
            // Update blur direction for horizontal pass
            let horizontal_blur = BlurUniforms {
                direction: [1.0, 0.0],
                blur_strength: 0.6,
                _padding: 0.0,
            };
            self.queue.write_buffer(
                &self.blur_uniform_buffer,
                0,
                bytemuck::cast_slice(&[horizontal_blur]),
            );

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Horizontal Blur Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.blur_texture_1_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.blur_pipeline);
            render_pass.set_bind_group(0, &self.blur_horizontal_bind_group, &[]);
            render_pass.set_bind_group(1, &self.blur_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        // Pass 3: Vertical blur
        {
            // Update blur direction for vertical pass
            let vertical_blur = BlurUniforms {
                direction: [0.0, 1.0],
                blur_strength: 0.6,
                _padding: 0.0,
            };
            self.queue.write_buffer(
                &self.blur_uniform_buffer,
                0,
                bytemuck::cast_slice(&[vertical_blur]),
            );

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Vertical Blur Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.blur_texture_2_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.blur_pipeline);
            render_pass.set_bind_group(0, &self.blur_vertical_bind_group, &[]);
            render_pass.set_bind_group(1, &self.blur_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        // Pass 4: Final composition
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Bloom Composition Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: final_target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.composition_pipeline);
            render_pass.set_bind_group(0, &self.composition_bind_group, &[]);
            render_pass.set_bind_group(1, &self.composition_settings_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32, surface_format: wgpu::TextureFormat) {
        self.width = new_width;
        self.height = new_height;

        // Recreate all textures and bind groups with new size
        // (This is a simplified version - in practice you'd want to avoid recreating everything)
        *self = Self::new(
            self.device.clone(),
            self.queue.clone(),
            surface_format,
            new_width,
            new_height,
        );
    }
}
