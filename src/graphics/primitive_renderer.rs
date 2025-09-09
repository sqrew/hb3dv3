//! Instanced primitive rendering system for high-performance particle rendering
//! 
//! This module provides instanced rendering of primitives (tetrahedrons, spheres, etc.),
//! allowing thousands of primitives to be rendered efficiently in a single draw call.

use std::sync::Arc;
use std::collections::HashMap;
use wgpu::util::DeviceExt;
use super::vertex::{Vertex, PrimitiveInstance};
use super::primitive_cache::{PrimitiveCache, CachedPrimitive};
use super::primitives::PrimitiveType;

pub struct InstancedPrimitiveRenderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    
    // Shared primitive cache
    primitive_cache: Arc<PrimitiveCache>,
    
    // Per-primitive-type instance buffers (double buffered)
    instance_buffers_a: HashMap<PrimitiveType, wgpu::Buffer>,
    instance_buffers_b: HashMap<PrimitiveType, wgpu::Buffer>,
    current_buffer: bool, // false = A, true = B
    max_instances_per_type: usize,
    
    // Render pipeline
    render_pipeline: wgpu::RenderPipeline,
}

impl InstancedPrimitiveRenderer {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        primitive_cache: Arc<PrimitiveCache>,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        surface_format: wgpu::TextureFormat,
        max_instances_per_type: usize,
    ) -> Self {
        // Load instanced primitive shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Instanced Primitive Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/instanced_primitives.wgsl").into()),
        });
        
        // Create render pipeline
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Instanced Primitive Pipeline Layout"),
            bind_group_layouts: &[camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Instanced Primitive Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::desc(),
                    PrimitiveInstance::desc(),
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
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
        });
        
        // Create instance buffers for all primitive types
        let primitive_types = vec![
            PrimitiveType::Tetrahedron,
            PrimitiveType::Sphere,
            PrimitiveType::Cube,
            PrimitiveType::Octahedron,
            PrimitiveType::Pyramid,
            PrimitiveType::Cone,
            PrimitiveType::Cylinder,
            PrimitiveType::Torus,
        ];
        
        let mut instance_buffers_a = HashMap::new();
        let mut instance_buffers_b = HashMap::new();
        
        for primitive_type in primitive_types {
            let buffer_size = max_instances_per_type * std::mem::size_of::<PrimitiveInstance>();
            
            let buffer_a = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("{:?} Instance Buffer A", primitive_type)),
                size: buffer_size as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            
            let buffer_b = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("{:?} Instance Buffer B", primitive_type)),
                size: buffer_size as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            
            instance_buffers_a.insert(primitive_type, buffer_a);
            instance_buffers_b.insert(primitive_type, buffer_b);
        }
        
        Self {
            device,
            queue,
            primitive_cache,
            instance_buffers_a,
            instance_buffers_b,
            current_buffer: false,
            max_instances_per_type,
            render_pipeline,
        }
    }
    
    /// Submit instances for a specific primitive type to be rendered
    pub fn submit_instances(&mut self, primitive_type: PrimitiveType, instances: &[PrimitiveInstance]) {
        if instances.is_empty() || instances.len() > self.max_instances_per_type {
            return;
        }
        
        let instance_buffer = if self.current_buffer {
            &self.instance_buffers_b[&primitive_type]
        } else {
            &self.instance_buffers_a[&primitive_type]
        };
        
        // Upload instance data
        self.queue.write_buffer(
            instance_buffer,
            0,
            bytemuck::cast_slice(instances),
        );
    }
    
    /// Render all submitted instances
    pub fn render(&mut self, render_pass: &mut wgpu::RenderPass, camera_bind_group: &wgpu::BindGroup) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        
        // Render each primitive type that has instances
        for (primitive_type, primitive_data) in self.primitive_cache.get_all_primitives() {
            let instance_buffer = if self.current_buffer {
                &self.instance_buffers_b[primitive_type]
            } else {
                &self.instance_buffers_a[primitive_type]
            };
            
            // Set primitive vertex and index buffers
            render_pass.set_vertex_buffer(0, primitive_data.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
            render_pass.set_index_buffer(primitive_data.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            
            // Draw instances (we'll need to track instance count per type)
            // For now, we'll assume max instances, but we should optimize this
            render_pass.draw_indexed(
                0..primitive_data.index_count,
                0,
                0..self.max_instances_per_type as u32,
            );
        }
    }
    
    /// Swap buffers for next frame
    pub fn swap_buffers(&mut self) {
        self.current_buffer = !self.current_buffer;
    }
}