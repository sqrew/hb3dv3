//! Instanced line rendering system for high-performance wireframe rendering
//! 
//! This module provides instanced rendering of cylindrical lines, allowing
//! thousands of lines to be rendered efficiently in a single draw call.

use std::sync::Arc;
use wgpu::util::DeviceExt;
use super::vertex::{CylinderVertex, LineInstance};
use super::primitives::generate_line_cylinder;

pub struct InstancedLineRenderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    
    // Cylinder mesh (shared for all lines)
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    
    // Double-buffered instance data
    instance_buffer_a: wgpu::Buffer,
    instance_buffer_b: wgpu::Buffer,
    current_buffer: bool, // false = A, true = B
    max_instances: usize,
    
    // Render pipeline
    render_pipeline: wgpu::RenderPipeline,
}

impl InstancedLineRenderer {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        // Load line shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Line Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/lines.wgsl").into()),
        });
        
        // Generate cylinder mesh
        let (vertices, indices) = generate_line_cylinder();
        let index_count = indices.len() as u32;
        
        // Create vertex buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Line Cylinder Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        // Create index buffer
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Line Cylinder Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        // Create double-buffered instance buffers (space for 3M lines for extreme fractal bullet spam)
        // 3M instances × 44 bytes = 132MB per buffer × 2 buffers = 264MB total
        let max_instances = 3_000_000;
        let instance_buffer_size = (std::mem::size_of::<LineInstance>() * max_instances) as u64;
        
        let instance_buffer_a = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Line Instance Buffer A"),
            size: instance_buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let instance_buffer_b = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Line Instance Buffer B"),
            size: instance_buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Create render pipeline layout
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Line Render Pipeline Layout"),
            bind_group_layouts: &[camera_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // Create render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Line Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    CylinderVertex::desc(), // Vertex data (location 0-1)
                    LineInstance::desc(),   // Instance data (location 2-5)
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_emissive"), // Use emissive fragment for Geometry Wars style
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING), // Enable alpha blending for glow
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // Cylinder is made of triangles
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back), // Cull back faces for performance
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        
        Self {
            device,
            queue,
            vertex_buffer,
            index_buffer,
            index_count,
            instance_buffer_a,
            instance_buffer_b,
            current_buffer: false,
            max_instances,
            render_pipeline,
        }
    }
    
    /// Update GPU buffers with line data (call before render pass)
    pub fn update_buffers(&mut self, lines: &[LineInstance]) {
        // Ensure we don't exceed buffer capacity
        let line_count = lines.len().min(self.max_instances);
        let lines_to_render = &lines[..line_count];
        
        // Switch buffer for next frame to prevent GPU stalls
        self.current_buffer = !self.current_buffer;
        let current_instance_buffer = if self.current_buffer { 
            &self.instance_buffer_b 
        } else { 
            &self.instance_buffer_a 
        };
        
        // Update the current buffer with this frame's data
        if !lines_to_render.is_empty() {
            self.queue.write_buffer(
                current_instance_buffer,
                0,
                bytemuck::cast_slice(lines_to_render),
            );
        }
    }
    
    /// Render a batch of lines using instanced rendering (buffers must be updated first)
    pub fn render_lines(
        &mut self,
        render_pass: &mut wgpu::RenderPass,
        camera_bind_group: &wgpu::BindGroup,
        lines: &[LineInstance],
    ) {
        let line_count = lines.len().min(self.max_instances);
        if line_count == 0 {
            return;
        }

        let current_instance_buffer = if self.current_buffer {
            &self.instance_buffer_b
        } else {
            &self.instance_buffer_a
        };

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, current_instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.draw_indexed(0..self.index_count, 0, 0..line_count as u32);
    }
    
    /// Get the maximum number of lines that can be rendered in one batch
    pub fn max_instance_count(&self) -> usize {
        self.max_instances
    }
    
    /// Helper function to create a LineInstance from start/end points
    pub fn create_line_instance(
        start: [f32; 3],
        end: [f32; 3],
        thickness: f32,
        color: [f32; 4],
    ) -> LineInstance {
        LineInstance {
            start_pos: start,
            end_pos: end,
            thickness,
            color,
        }
    }
}