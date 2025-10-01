//! Batched line generation system for optimized wireframe rendering
//!
//! Reduces allocations by collecting all line generation operations
//! into a single batch before creating the final line instances.

use super::vertex::LineInstance;
use super::primitive_cache::PrimitiveCache;
use crate::graphics::{PrimitiveType, Color, Vec3};
use rayon::prelude::*;

/// Represents a single primitive render request
#[derive(Debug, Clone)]
pub struct PrimitiveRenderRequest {
    pub primitive_type: PrimitiveType,
    pub world_position: Vec3,
    pub rotation: Vec3,        // ADD ROTATION SUPPORT!
    pub scale: Vec3,
    pub color: Color,
    pub thickness: f32,
}

/// Batched line generator that reduces allocations
pub struct LineBatch {
    requests: Vec<PrimitiveRenderRequest>,
    estimated_line_count: usize,
}

impl LineBatch {
    /// Create a new line batch with estimated capacity
    pub fn with_capacity(estimated_primitives: usize) -> Self {
        // Estimate line count: average primitive has ~12 lines (cube = 12, sphere = ~128)
        let estimated_line_count = estimated_primitives * 20; // Conservative estimate
        
        Self {
            requests: Vec::with_capacity(estimated_primitives),
            estimated_line_count,
        }
    }
    
    /// Add a primitive to the batch with rotation support
    pub fn add_primitive_with_rotation(
        &mut self,
        primitive_type: PrimitiveType,
        world_position: Vec3,
        rotation: Vec3,
        scale: Vec3,
        color: Color,
        thickness: f32,
    ) {
        self.requests.push(PrimitiveRenderRequest {
            primitive_type,
            world_position,
            rotation,
            scale,
            color,
            thickness,
        });
    }
    
    /// Add a primitive to the batch (legacy method without rotation)
    pub fn add_primitive(
        &mut self,
        primitive_type: PrimitiveType,
        world_position: Vec3,
        scale: Vec3,
        color: Color,
        thickness: f32,
    ) {
        self.add_primitive_with_rotation(
            primitive_type,
            world_position,
            Vec3::zeros(), // No rotation
            scale,
            color,
            thickness,
        );
    }
    
    /// Generate all line instances in a single allocation, batched by primitive type
    pub fn finish(self, primitive_cache: &PrimitiveCache) -> Vec<LineInstance> {
        use std::collections::HashMap;
        
        // Group requests by primitive type for better cache locality
        let mut grouped_requests: HashMap<PrimitiveType, Vec<PrimitiveRenderRequest>> = HashMap::new();
        let _total_requests = self.requests.len();
        
        for request in self.requests {
            grouped_requests.entry(request.primitive_type)
                .or_insert_with(Vec::new)
                .push(request);
        }
        
        
        let mut lines = Vec::with_capacity(self.estimated_line_count);
        
        // Process each primitive type as a batch
        // Order: Simple primitives first (fewer lines), complex ones last
        const PRIMITIVE_ORDER: [PrimitiveType; 23] = [
            PrimitiveType::Cube,
            PrimitiveType::Tetrahedron,
            PrimitiveType::Pyramid,
            PrimitiveType::Octahedron,
            PrimitiveType::Cone,
            PrimitiveType::Cylinder,
            PrimitiveType::Ellipsoid,
            PrimitiveType::Sphere,
            PrimitiveType::Torus,
            PrimitiveType::Icosahedron,
            PrimitiveType::Dodecahedron,
            PrimitiveType::Capsule,
            PrimitiveType::Plane,
            PrimitiveType::Hemisphere,
            // 2D Primitives (simple, few lines)
            PrimitiveType::Triangle2D,
            PrimitiveType::Square2D, 
            PrimitiveType::Circle2D,
            PrimitiveType::Pentagon2D,
            PrimitiveType::Hexagon2D,
            PrimitiveType::Diamond2D,
            PrimitiveType::Cross2D,
            PrimitiveType::Star2D,
            PrimitiveType::Arrow2D,
        ];
        
        for primitive_type in &PRIMITIVE_ORDER {
            if let Some(requests) = grouped_requests.get(primitive_type) {
                // Use parallel processing for batches with many instances
                if requests.len() > 10 {
                    // Parallel process large batches of the same primitive type
                    let batch_lines: Vec<Vec<LineInstance>> = requests
                        .par_iter()
                        .map(|request| {
                            primitive_cache.generate_line_instances_with_rotation(
                                request.primitive_type,
                                request.world_position,
                                request.rotation,
                                request.scale,
                                request.color,
                                request.thickness,
                            )
                        })
                        .collect();
                    
                    // Extend with all the batch results
                    for primitive_lines in batch_lines {
                        lines.extend(primitive_lines);
                    }
                } else {
                    // Sequential processing for small batches to avoid overhead
                    for request in requests {
                        let primitive_lines = primitive_cache.generate_line_instances_with_rotation(
                            request.primitive_type,
                            request.world_position,
                            request.rotation,
                            request.scale,
                            request.color,
                            request.thickness,
                        );
                        
                        lines.extend(primitive_lines);
                    }
                }
            }
        }
        
        lines
    }

    /// Generate all lines for this batch into a reusable buffer to avoid allocations
    pub fn finish_into(self, primitive_cache: &PrimitiveCache, buffer: &mut Vec<LineInstance>) {
        // Group requests by primitive type for better cache locality
        // MUST match the full list from finish() to avoid silently dropping primitives!
        for primitive_type in [
            PrimitiveType::Cube,
            PrimitiveType::Tetrahedron,
            PrimitiveType::Pyramid,
            PrimitiveType::Octahedron,
            PrimitiveType::Cone,
            PrimitiveType::Cylinder,
            PrimitiveType::Ellipsoid,
            PrimitiveType::Sphere,
            PrimitiveType::Torus,
            PrimitiveType::Icosahedron,
            PrimitiveType::Dodecahedron,
            PrimitiveType::Capsule,
            PrimitiveType::Plane,
            PrimitiveType::Hemisphere,
            // 2D Primitives
            PrimitiveType::Triangle2D,
            PrimitiveType::Square2D,
            PrimitiveType::Circle2D,
            PrimitiveType::Pentagon2D,
            PrimitiveType::Hexagon2D,
            PrimitiveType::Diamond2D,
            PrimitiveType::Cross2D,
            PrimitiveType::Star2D,
            PrimitiveType::Arrow2D,
        ] {
            // Collect all requests for this primitive type
            let requests: Vec<_> = self.requests.iter()
                .filter(|r| r.primitive_type == primitive_type)
                .collect();

            if !requests.is_empty() {
                // Process all requests of this type at once for better cache performance
                for request in requests {
                    let primitive_lines = primitive_cache.generate_line_instances_with_rotation(
                        request.primitive_type,
                        request.world_position,
                        request.rotation,
                        request.scale,
                        request.color,
                        request.thickness,
                    );

                    buffer.extend(primitive_lines);
                }
            }
        }
    }
    
    /// Get the number of primitives in this batch
    pub fn primitive_count(&self) -> usize {
        self.requests.len()
    }
    
    /// Clear the batch for reuse
    pub fn clear(&mut self) {
        self.requests.clear();
    }
}

/// Batched line generator that can be reused across frames
pub struct ReusableLineBatch {
    batch: LineBatch,
    direct_lines: Vec<LineInstance>,
}

impl ReusableLineBatch {
    /// Create a new reusable line batch
    pub fn new(estimated_primitives: usize) -> Self {
        Self {
            batch: LineBatch::with_capacity(estimated_primitives),
            direct_lines: Vec::new(),
        }
    }
    
    /// Start a new frame by clearing previous data
    pub fn start_frame(&mut self) {
        self.batch.clear();
        self.direct_lines.clear();
    }
    
    /// Add a primitive to the current batch
    pub fn add_primitive(
        &mut self,
        primitive_type: PrimitiveType,
        world_position: Vec3,
        scale: Vec3,
        color: Color,
        thickness: f32,
    ) {
        self.batch.add_primitive(primitive_type, world_position, scale, color, thickness);
    }
    
    /// Add a primitive to the current batch with rotation
    pub fn add_primitive_with_rotation(
        &mut self,
        primitive_type: PrimitiveType,
        world_position: Vec3,
        rotation: Vec3,
        scale: Vec3,
        color: Color,
        thickness: f32,
    ) {
        self.batch.add_primitive_with_rotation(primitive_type, world_position, rotation, scale, color, thickness);
    }

    /// Add pre-computed line instances directly to the batch (for dynamic effects like lightning)
    pub fn add_line_instances(&mut self, instances: Vec<LineInstance>) {
        self.direct_lines.extend(instances);
    }
    
    /// Generate all line instances for the current batch using a reusable buffer
    pub fn finish_frame_into(&mut self, primitive_cache: &PrimitiveCache, buffer: &mut Vec<LineInstance>) {
        buffer.clear(); // Reuse existing capacity

        // Take ownership of requests to avoid cloning
        let requests = std::mem::take(&mut self.batch.requests);
        let estimated_line_count = self.batch.estimated_line_count;

        // Create temporary batch that owns the data
        let batch = LineBatch { requests, estimated_line_count };

        // Generate lines from primitives directly into buffer
        batch.finish_into(primitive_cache, buffer);

        // Add direct line instances (e.g., from lightning effects) only if they exist
        if !self.direct_lines.is_empty() {
            buffer.extend(std::mem::take(&mut self.direct_lines));
        }
    }

    /// Generate all line instances for the current batch (consumes batch) - DEPRECATED
    /// Use finish_frame_into() instead to avoid allocations
    pub fn finish_frame(&mut self, primitive_cache: &PrimitiveCache) -> Vec<LineInstance> {
        let mut buffer = Vec::new();
        self.finish_frame_into(primitive_cache, &mut buffer);
        buffer
    }
    
    /// Get the number of primitives in the current batch
    pub fn primitive_count(&self) -> usize {
        self.batch.primitive_count()
    }
}