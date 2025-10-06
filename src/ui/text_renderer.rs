use bytemuck::{Pod, Zeroable};
use fontdue::{Font, FontSettings};
use std::collections::HashMap;
use wgpu::util::DeviceExt;

use crate::graphics::Color;

// Text vertex for rendering glyph quads
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct TextVertex {
    pub position: [f32; 2],   // 2D position (world/screen space)
    pub tex_coords: [f32; 2], // UV coordinates into glyph atlas
    pub color: [f32; 4],      // Text color RGBA
}

impl TextVertex {
    pub const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x2, // position
        1 => Float32x2, // tex_coords
        2 => Float32x4, // color
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

// Glyph information in the atlas
#[derive(Debug, Clone)]
pub struct GlyphInfo {
    pub uv: [f32; 4],      // UV coords: [min_x, min_y, max_x, max_y]
    pub size: [f32; 2],    // Glyph size in pixels
    pub bearing: [f32; 2], // Bearing (offset from baseline)
    pub advance: f32,      // Horizontal advance
}

// Glyph atlas for texture management
pub struct GlyphAtlas {
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub width: u32,
    pub height: u32,
    glyph_cache: HashMap<(char, u32), GlyphInfo>, // (character, font_size) -> GlyphInfo
    current_x: u32,
    current_y: u32,
    row_height: u32,
}

impl GlyphAtlas {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, initial_size: u32) -> Self {
        // Create a white texture for now so we can see colored quads
        let white_data: Vec<u8> = vec![255; (initial_size * initial_size) as usize];

        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("Glyph Atlas Texture"),
                size: wgpu::Extent3d {
                    width: initial_size,
                    height: initial_size,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::default(),
            &white_data,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Glyph Atlas Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            texture_view,
            sampler,
            width: initial_size,
            height: initial_size,
            glyph_cache: HashMap::new(),
            current_x: 0,
            current_y: 0,
            row_height: 0,
        }
    }

    pub fn get_or_cache_glyph(
        &mut self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        font: &Font,
        character: char,
        font_size: u32,
    ) -> Option<&GlyphInfo> {
        let key = (character, font_size);

        if self.glyph_cache.contains_key(&key) {
            return self.glyph_cache.get(&key);
        }

        // Rasterize the glyph
        let (metrics, bitmap) = font.rasterize(character, font_size as f32);

        if bitmap.is_empty() {
            return None;
        }

        // Check if we need to move to the next row
        if self.current_x + metrics.width as u32 > self.width {
            self.current_x = 0;
            self.current_y += self.row_height;
            self.row_height = 0;
        }

        // Check if we need more space vertically
        if self.current_y + metrics.height as u32 > self.height {
            // For now, just skip the glyph if we're out of space
            // In a real implementation, you'd want to resize the atlas
            eprintln!("Warning: Glyph atlas out of space");
            return None;
        }

        // Upload glyph bitmap to atlas using correct wgpu 26 API
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: self.current_x,
                    y: self.current_y,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &bitmap,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(metrics.width as u32),
                rows_per_image: Some(metrics.height as u32),
            },
            wgpu::Extent3d {
                width: metrics.width as u32,
                height: metrics.height as u32,
                depth_or_array_layers: 1,
            },
        );

        // Calculate UV coordinates
        let uv = [
            self.current_x as f32 / self.width as f32,
            self.current_y as f32 / self.height as f32,
            (self.current_x + metrics.width as u32) as f32 / self.width as f32,
            (self.current_y + metrics.height as u32) as f32 / self.height as f32,
        ];

        let glyph_info = GlyphInfo {
            uv,
            size: [metrics.width as f32, metrics.height as f32],
            bearing: [metrics.xmin as f32, metrics.ymin as f32],
            advance: metrics.advance_width,
        };

        // Update atlas position
        self.current_x += metrics.width as u32;
        self.row_height = self.row_height.max(metrics.height as u32);

        self.glyph_cache.insert(key, glyph_info);
        self.glyph_cache.get(&key)
    }
}

// Main text renderer
pub struct TextRenderer {
    font: Font,
    atlas: GlyphAtlas,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vertex_buffer_capacity: usize,
    index_buffer_capacity: usize,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    screen_uniform_buffer: wgpu::Buffer,
    screen_bind_group: wgpu::BindGroup,
    screen_size: [f32; 2],
    vertices: Vec<TextVertex>,
    indices: Vec<u16>,
}

impl TextRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        // Load embedded JetBrains Mono font
        let font_data = include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf");
        let font = Font::from_bytes(font_data.as_slice(), FontSettings::default())
            .expect("Failed to load JetBrains Mono font");

        // Create glyph atlas
        let atlas = GlyphAtlas::new(device, queue, 512); // 512x512 initial atlas

        // Create bind group layout for texture
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Text Bind Group Layout"),
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

        // Create screen uniform bind group layout
        let screen_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Screen Uniform Bind Group Layout"),
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
            });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Text Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas.texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&atlas.sampler),
                },
            ],
        });

        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Text Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("text.wgsl").into()),
        });

        // Create screen uniform buffer
        let screen_size = [config.width as f32, config.height as f32];
        let screen_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Screen Uniform Buffer"),
            contents: bytemuck::cast_slice(&screen_size),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create screen bind group
        let screen_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Screen Bind Group"),
            layout: &screen_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen_uniform_buffer.as_entire_binding(),
            }],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Text Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout, &screen_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Text Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[TextVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // Disable culling for UI text
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false, // UI text should not write to depth buffer
                depth_compare: wgpu::CompareFunction::Always, // Always pass depth test for UI
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

        // Create buffers (start with some reasonable size)
        let vertex_buffer_capacity = 1000;
        let index_buffer_capacity = 1500;

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Text Vertex Buffer"),
            size: (std::mem::size_of::<TextVertex>() * vertex_buffer_capacity) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Text Index Buffer"),
            size: (std::mem::size_of::<u16>() * index_buffer_capacity) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            font,
            atlas,
            vertex_buffer,
            index_buffer,
            vertex_buffer_capacity,
            index_buffer_capacity,
            pipeline,
            bind_group,
            screen_uniform_buffer,
            screen_bind_group,
            screen_size,
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn add_text(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        text: &str,
        position: [f32; 2],
        font_size: u32,
        color: Color,
    ) {
        let mut current_x = position[0];
        let y = position[1];

        for character in text.chars() {
            if character.is_whitespace() {
                // Handle whitespace - just advance the cursor
                if character == ' ' {
                    current_x += font_size as f32 * 0.5; // Approximate space width
                }
                continue;
            }

            // Get or cache the glyph from the atlas
            if let Some(glyph_info) = self
                .atlas
                .get_or_cache_glyph(device, queue, &self.font, character, font_size)
            {
                // Use actual glyph dimensions
                let char_width = glyph_info.size[0];
                let char_height = glyph_info.size[1];

                // Create quad vertices with world positions baked in
                let color_array = color.to_array4();
                let vertices = [
                    TextVertex {
                        position: [current_x, y],
                        tex_coords: [glyph_info.uv[0], glyph_info.uv[1]], // Top-left UV
                        color: color_array,
                    },
                    TextVertex {
                        position: [current_x + char_width, y],
                        tex_coords: [glyph_info.uv[2], glyph_info.uv[1]], // Top-right UV
                        color: color_array,
                    },
                    TextVertex {
                        position: [current_x + char_width, y + char_height],
                        tex_coords: [glyph_info.uv[2], glyph_info.uv[3]], // Bottom-right UV
                        color: color_array,
                    },
                    TextVertex {
                        position: [current_x, y + char_height],
                        tex_coords: [glyph_info.uv[0], glyph_info.uv[3]], // Bottom-left UV
                        color: color_array,
                    },
                ];

                // Add vertices
                let base_index = self.vertices.len() as u16;
                self.vertices.extend_from_slice(&vertices);

                // Add indices for two triangles (quad)
                let quad_indices = [
                    base_index,
                    base_index + 1,
                    base_index + 2, // First triangle
                    base_index,
                    base_index + 2,
                    base_index + 3, // Second triangle
                ];
                self.indices.extend_from_slice(&quad_indices);

                current_x += glyph_info.advance;
            }
        }
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass) {
        if self.vertices.is_empty() || self.indices.is_empty() {
            return;
        }

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_bind_group(1, &self.screen_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        // Draw all characters in a single batched draw call
        render_pass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);
    }

    fn resize_buffers_if_needed(&mut self, device: &wgpu::Device) {
        // Check if we need to resize vertex buffer
        if self.vertices.len() > self.vertex_buffer_capacity {
            self.vertex_buffer_capacity = (self.vertices.len() * 2).max(1000);
            self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Text Vertex Buffer"),
                size: (std::mem::size_of::<TextVertex>() * self.vertex_buffer_capacity) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        // Check if we need to resize index buffer
        if self.indices.len() > self.index_buffer_capacity {
            self.index_buffer_capacity = (self.indices.len() * 2).max(1500);
            self.index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Text Index Buffer"),
                size: (std::mem::size_of::<u16>() * self.index_buffer_capacity) as u64,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
    }

    pub fn update_buffers(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        // Resize buffers if needed before writing data
        self.resize_buffers_if_needed(device);

        if !self.vertices.is_empty() {
            queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
        }
        if !self.indices.is_empty() {
            queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&self.indices));
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }
}
