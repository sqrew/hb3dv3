//! Lightning effects system for creating animated electrical arcs
//!
//! This module provides tools for generating realistic lightning bolts
//! with animation, branching, and integration with the existing line renderer.

use crate::engine::Vec3;
use crate::graphics::{Color, vertex::LineInstance};
use rand::rng;

/// Configuration for enhanced lightning bolt generation
#[derive(Debug, Clone)]
pub struct LightningConfig {
    /// Number of segments to create between start and end points
    pub segment_count: usize,
    /// Maximum random offset from straight line (as fraction of total distance)
    pub chaos_factor: f32,
    /// Thickness of the main lightning bolt
    pub thickness: f32,
    /// Number of smaller branches to create
    pub branch_count: usize,
    /// Color of the lightning bolt
    pub color: Color,
    /// How long the lightning effect lasts (seconds)
    pub duration: f32,
    /// How fast the lightning animates (flicker speed)
    pub animation_speed: f32,

    // Enhanced features
    /// Maximum number of branch generations (branches off branches)
    pub max_branch_generations: usize,
    /// Probability of creating a branch at each segment (0.0 to 1.0)
    pub branch_probability: f32,
    /// How much branches diminish in thickness each generation
    pub branch_thickness_decay: f32,
    /// How much branches diminish in length each generation
    pub branch_length_decay: f32,
    /// Minimum branch length as fraction of main bolt
    pub min_branch_length: f32,
    /// Enable fractal sub-branching
    pub enable_fractal_branching: bool,
    /// Enable progressive bolt formation animation
    pub enable_progressive_formation: bool,
    /// Speed of progressive formation (segments per second)
    pub formation_speed: f32,

    // Seeking tendril features
    /// Number of seeking tendrils that branch off and terminate as dead ends
    pub seeking_tendril_count: usize,
    /// Length of seeking tendrils as fraction of main bolt length
    pub seeking_tendril_length: f32,
    /// How much thinner seeking tendrils are compared to main bolt
    pub seeking_tendril_thickness: f32,
    /// How chaotic/random seeking tendrils are (higher = more exploration)
    pub seeking_tendril_chaos: f32,
    /// Duration multiplier for seeking tendrils (how much longer they last compared to main bolt)
    pub seeking_tendril_duration_multiplier: f32,
}

impl Default for LightningConfig {
    fn default() -> Self {
        Self {
            segment_count: 16,
            chaos_factor: 0.4,
            thickness: 0.10,
            branch_count: 4,
            color: Color::new(0.1, 0.8, 1.0, 0.9), // Electric blue
            duration: 0.5,                         // Much longer base duration
            animation_speed: 100.0,

            // Enhanced defaults for spectacular effects
            max_branch_generations: 3,
            branch_probability: 0.6,
            branch_thickness_decay: 0.6,
            branch_length_decay: 0.7,
            min_branch_length: 0.1,
            enable_fractal_branching: true,
            enable_progressive_formation: true,
            formation_speed: 100.0,

            // Seeking tendril defaults - longer and more prominent
            seeking_tendril_count: 5, // More tendrils for better visibility
            seeking_tendril_length: 0.8, // Much longer - almost as long as main bolt
            seeking_tendril_thickness: 0.8, // Much thicker for better visibility
            seeking_tendril_chaos: 1.2,
            seeking_tendril_duration_multiplier: 2.0, // Last three times as long as main bolt
        }
    }
}

impl LightningConfig {
    /// Create a spectacular high-intensity lightning configuration
    pub fn spectacular() -> Self {
        Self {
            segment_count: 64,
            chaos_factor: 0.5,
            thickness: 0.12,
            branch_count: 16,
            color: Color::new(0.1, 0.8, 1.0, 0.95), // Brighter electric blue
            duration: 1.0,                          // Even longer for spectacular mode
            animation_speed: 50.0,

            max_branch_generations: 4,
            branch_probability: 0.8,
            branch_thickness_decay: 0.5,
            branch_length_decay: 0.6,
            min_branch_length: 0.05,
            enable_fractal_branching: true,
            enable_progressive_formation: true,
            formation_speed: 40.0,

            // Enhanced seeking tendrils for spectacular mode - dramatic and long
            seeking_tendril_count: 8, // Even more tendrils in spectacular mode
            seeking_tendril_length: 0.9, // Nearly as long as main bolt
            seeking_tendril_thickness: 0.7, // Much more visible
            seeking_tendril_chaos: 1.8, // More dramatic exploration
            seeking_tendril_duration_multiplier: 1.0, // Last much longer in spectacular mode
        }
    }
}

/// A single lightning segment with animation data
#[derive(Debug, Clone)]
struct LightningSegment {
    start: Vec3,
    end: Vec3,
    original_start: Vec3,
    original_end: Vec3,
    thickness_multiplier: f32,
    flicker_phase: f32,
    is_branch: bool,
    is_seeking_tendril: bool,
}

/// A complete lightning bolt effect
pub struct LightningBolt {
    segments: Vec<LightningSegment>,
    config: LightningConfig,
    time_alive: f32,
}

impl LightningBolt {
    /// Create a new lightning bolt between two points
    pub fn new(start: Vec3, end: Vec3, config: LightningConfig) -> Self {
        let mut bolt = Self {
            segments: Vec::new(),
            config,
            time_alive: 0.0,
        };

        bolt.generate_segments(start, end);
        bolt
    }

    /// Generate the lightning segments between start and end points
    fn generate_segments(&mut self, start: Vec3, end: Vec3) {
        let mut rng = rng();

        // Create main lightning path
        let main_segments = self.create_main_path(start, end, &mut rng);
        self.segments.extend(main_segments);

        if self.config.enable_fractal_branching {
            // Enhanced fractal branching system
            self.generate_fractal_branches(&mut rng);
        } else {
            // Simple branch creation for fallback
            for _ in 0..self.config.branch_count {
                if let Some(branch_segments) = self.create_simple_branch(&mut rng) {
                    self.segments.extend(branch_segments);
                }
            }
        }

        // Generate seeking tendrils - dead-end branches that explore alternative paths
        self.generate_seeking_tendrils(start, end, &mut rng);
    }

    /// Create the main lightning path with controlled randomness
    fn create_main_path(
        &self,
        start: Vec3,
        end: Vec3,
        rng: &mut impl rand::Rng,
    ) -> Vec<LightningSegment> {
        let mut segments = Vec::new();
        let mut current_pos = start;

        let total_distance = (end - start).magnitude();
        let step_size = 1.0 / self.config.segment_count as f32;

        for i in 0..self.config.segment_count {
            let t = (i + 1) as f32 * step_size;
            let target_pos = start + (end - start) * t;

            // Add controlled randomness with full 3D variation
            let main_direction = (end - start).normalize();
            let (perpendicular_x, perpendicular_y) =
                self.get_3d_perpendicular_vectors(main_direction);

            let offset_magnitude = total_distance * self.config.chaos_factor;

            // Create 3D random offset with Y-axis variation
            let x_offset = perpendicular_x * offset_magnitude * rng.random_range(-1.0..1.0);
            let y_offset = perpendicular_y * offset_magnitude * rng.random_range(-1.0..1.0);
            let z_offset =
                Vec3::new(0.0, 1.0, 0.0) * offset_magnitude * rng.random_range(-0.7..0.7); // Vertical variation

            let random_offset = x_offset + y_offset + z_offset;

            // Reduce randomness near endpoints for better connection
            let endpoint_damping = (0.5 - (t - 0.5).abs()) * 2.0; // 0 at endpoints, 1 at center
            let final_pos = target_pos + random_offset * endpoint_damping;

            segments.push(LightningSegment {
                start: current_pos,
                end: final_pos,
                original_start: current_pos,
                original_end: final_pos,
                thickness_multiplier: 1.0,
                flicker_phase: rng.random_range(0.0..std::f32::consts::PI * 2.0),
                is_branch: false,
                is_seeking_tendril: false,
            });

            current_pos = final_pos;
        }

        segments
    }

    /// Generate fractal branching system with multiple generations
    fn generate_fractal_branches(&mut self, rng: &mut impl rand::Rng) {
        let main_segment_count = self.segments.len().min(self.config.segment_count);

        // Start with the main path segments for branching
        let mut branch_candidates: Vec<(usize, usize)> = (0..main_segment_count)
            .map(|i| (i, 0)) // (segment_index, generation)
            .collect();

        // Generate branches for each generation
        for generation in 0..self.config.max_branch_generations {
            let mut new_candidates = Vec::new();
            let generation_thickness = self.config.branch_thickness_decay.powi(generation as i32);
            let generation_length = self.config.branch_length_decay.powi(generation as i32);

            for (segment_idx, _) in &branch_candidates {
                if *segment_idx >= self.segments.len() {
                    continue;
                }

                // Check if we should create a branch at this segment
                if rng.random_range(0.0..1.0) < self.config.branch_probability {
                    if let Some(branch_segments) = self.create_advanced_branch(
                        *segment_idx,
                        generation,
                        generation_thickness,
                        generation_length,
                        rng,
                    ) {
                        let start_idx = self.segments.len();
                        self.segments.extend(branch_segments);
                        let end_idx = self.segments.len();

                        // Add new branch segments as candidates for next generation
                        for i in start_idx..end_idx {
                            new_candidates.push((i, generation + 1));
                        }
                    }
                }
            }

            branch_candidates = new_candidates;

            // Stop if no new branches were created or we've reached minimum branch length
            if branch_candidates.is_empty() || generation_length < self.config.min_branch_length {
                break;
            }
        }
    }

    /// Create an advanced branch with enhanced physics and visuals
    fn create_advanced_branch(
        &self,
        parent_segment_idx: usize,
        generation: usize,
        thickness_multiplier: f32,
        length_multiplier: f32,
        rng: &mut impl rand::Rng,
    ) -> Option<Vec<LightningSegment>> {
        if parent_segment_idx >= self.segments.len() {
            return None;
        }

        let parent_segment = &self.segments[parent_segment_idx];

        // Branch from a random point along the parent segment (not just midpoint)
        let branch_point_t = rng.random_range(0.2..0.8);
        let branch_start =
            parent_segment.start + (parent_segment.end - parent_segment.start) * branch_point_t;

        // Create more realistic branch direction based on electrical physics with true 3D branching
        let parent_direction = (parent_segment.end - parent_segment.start).normalize();

        // Create a proper 3D coordinate system around the parent direction
        let (perpendicular_x, perpendicular_y) =
            self.get_3d_perpendicular_vectors(parent_direction);

        // Create branch direction with controlled randomness in full 3D space
        let angular_deviation = rng.random_range(0.3..1.2); // Wider angle range for more dramatic branches
        let azimuthal_angle = rng.random_range(0.0..std::f32::consts::PI * 2.0);
        let polar_angle = rng.random_range(-std::f32::consts::PI * 0.5..std::f32::consts::PI * 0.5); // Full vertical range

        // Enhanced 3D branching with vertical component
        let branch_direction = (
            parent_direction * (1.0 - angular_deviation) +
            perpendicular_x * (angular_deviation * azimuthal_angle.cos() * polar_angle.cos()) +
            perpendicular_y * (angular_deviation * azimuthal_angle.sin() * polar_angle.cos()) +
            // Add explicit vertical component for dramatic Y-axis branching
            Vec3::new(0.0, 1.0, 0.0) * (angular_deviation * polar_angle.sin() * rng.random_range(0.5..1.5))
        ).normalize();

        // Calculate branch length based on generation and randomness
        let base_length = (parent_segment.end - parent_segment.start).magnitude() * 0.7;
        let random_length_factor = rng.random_range(0.6..1.4);
        let final_length = base_length * length_multiplier * random_length_factor;

        // Ensure minimum branch length
        if final_length < base_length * self.config.min_branch_length {
            return None;
        }

        let branch_end = branch_start + branch_direction * final_length;

        // Create more segments for longer branches, fewer for shorter ones
        let segment_count = ((final_length / base_length) * 4.0).max(2.0).min(8.0) as usize;
        let mut segments = Vec::new();
        let mut current_pos = branch_start;

        for i in 0..segment_count {
            let t = (i + 1) as f32 / segment_count as f32;
            let target_pos = branch_start + (branch_end - branch_start) * t;

            // Add progressive randomness that increases toward the tip
            let randomness_factor = 0.15 * (1.0 + generation as f32 * 0.3); // More chaos in higher generations
            let chaos_amount = final_length * randomness_factor * t; // Increase toward tip

            // Enhanced 3D chaos for advanced branches
            let x_chaos = perpendicular_x * chaos_amount * rng.random_range(-1.0..1.0);
            let y_chaos = perpendicular_y * chaos_amount * rng.random_range(-1.0..1.0);
            let z_chaos = Vec3::new(0.0, 1.0, 0.0) * chaos_amount * rng.random_range(-0.7..0.7);

            let random_offset = x_chaos + y_chaos + z_chaos;

            let final_pos = target_pos + random_offset;

            segments.push(LightningSegment {
                start: current_pos,
                end: final_pos,
                original_start: current_pos,
                original_end: final_pos,
                thickness_multiplier: thickness_multiplier * rng.random_range(0.8..1.2), // Add thickness variation
                flicker_phase: rng.random_range(0.0..std::f32::consts::PI * 2.0),
                is_branch: true,
                is_seeking_tendril: false,
            });

            current_pos = final_pos;
        }

        Some(segments)
    }

    /// Create a simple branch lightning path (fallback method)
    fn create_simple_branch(&self, rng: &mut impl rand::Rng) -> Option<Vec<LightningSegment>> {
        if self.segments.is_empty() {
            return None;
        }

        // Pick a random main segment to branch from
        let main_segment_idx =
            rng.random_range(0..self.segments.len().min(self.config.segment_count));
        let main_segment = &self.segments[main_segment_idx];

        // Branch from midpoint of the segment
        let branch_start = (main_segment.start + main_segment.end) * 0.5;

        // Create a shorter branch in a random 3D direction
        let main_direction = (main_segment.end - main_segment.start).normalize();
        let (perpendicular_x, perpendicular_y) = self.get_3d_perpendicular_vectors(main_direction);
        let branch_length = (main_segment.end - main_segment.start).magnitude() * 0.6;

        // Enhanced 3D branch direction with Y-axis variation
        let branch_direction = (
            main_direction * rng.random_range(-0.5..0.5)
                + perpendicular_x * rng.random_range(-1.0..1.0)
                + perpendicular_y * rng.random_range(-1.0..1.0)
                + Vec3::new(0.0, 1.0, 0.0) * rng.random_range(-0.8..0.8)
            // Vertical branching
        )
        .normalize();
        let branch_end = branch_start + branch_direction * branch_length;

        // Create 2-3 segments for the branch
        let branch_segments = 3;
        let mut segments = Vec::new();
        let mut current_pos = branch_start;

        for i in 0..branch_segments {
            let t = (i + 1) as f32 / branch_segments as f32;
            let target_pos = branch_start + (branch_end - branch_start) * t;

            // Add 3D randomness to branch segments
            let chaos_amount = branch_length * 0.2;
            let x_chaos = perpendicular_x * chaos_amount * rng.random_range(-1.0..1.0);
            let y_chaos = perpendicular_y * chaos_amount * rng.random_range(-1.0..1.0);
            let z_chaos = Vec3::new(0.0, 1.0, 0.0) * chaos_amount * rng.random_range(-0.6..0.6);

            let random_offset = x_chaos + y_chaos + z_chaos;
            let final_pos = target_pos + random_offset * (1.0 - t); // Less randomness toward end

            segments.push(LightningSegment {
                start: current_pos,
                end: final_pos,
                original_start: current_pos,
                original_end: final_pos,
                thickness_multiplier: 0.4, // Branches are thinner
                flicker_phase: rng.random_range(0.0..std::f32::consts::PI * 2.0),
                is_branch: true,
                is_seeking_tendril: false,
            });

            current_pos = final_pos;
        }

        Some(segments)
    }

    /// Generate seeking tendrils - dead-end branches that explore alternative paths
    fn generate_seeking_tendrils(&mut self, start: Vec3, end: Vec3, rng: &mut impl rand::Rng) {
        if self.config.seeking_tendril_count == 0 {
            return;
        }

        let main_direction = (end - start).normalize();
        let main_length = (end - start).magnitude();
        let tendril_length = main_length * self.config.seeking_tendril_length;

        // Generate tendrils from random points along the main path
        for _i in 0..self.config.seeking_tendril_count {
            // Pick a random position along the main path to branch from
            let branch_point_t = rng.random_range(0.1..0.9);
            let branch_start = start + (end - start) * branch_point_t;

            // Create a seeking direction that's somewhat influenced by the main direction
            // but also explores in random directions
            let (perpendicular_x, perpendicular_y) =
                self.get_3d_perpendicular_vectors(main_direction);

            // Create a "seeking" direction - partially random, partially influenced by main direction
            let main_influence = rng.random_range(0.1..0.4); // Low influence - tendrils explore independently
            let random_x = rng.random_range(-1.0..1.0);
            let random_y = rng.random_range(-1.0..1.0);
            let random_z = rng.random_range(-1.0..1.0); // Full 3D exploration

            let seeking_direction = (main_direction * main_influence
                + perpendicular_x * random_x * self.config.seeking_tendril_chaos
                + perpendicular_y * random_y * self.config.seeking_tendril_chaos
                + Vec3::new(0.0, 1.0, 0.0) * random_z * self.config.seeking_tendril_chaos * 0.8)
                .normalize();

            // Add some length variation for more organic feel - longer ranges
            let actual_length = tendril_length * rng.random_range(0.8..1.4); // Much longer variation
            let tendril_end = branch_start + seeking_direction * actual_length;

            // Create segments for this seeking tendril
            if let Some(tendril_segments) =
                self.create_seeking_tendril_segments(branch_start, tendril_end, rng)
            {
                self.segments.extend(tendril_segments);
            }
        }
    }

    /// Create segments for a seeking tendril with extra chaos and organic feel
    fn create_seeking_tendril_segments(
        &self,
        start: Vec3,
        end: Vec3,
        rng: &mut impl rand::Rng,
    ) -> Option<Vec<LightningSegment>> {
        let tendril_direction = (end - start).normalize();
        let tendril_length = (end - start).magnitude();
        let (perpendicular_x, perpendicular_y) =
            self.get_3d_perpendicular_vectors(tendril_direction);

        // Create 5-8 segments for seeking tendrils - more detailed and longer
        let segment_count = rng.random_range(5..=8);
        let mut segments = Vec::new();
        let mut current_pos = start;

        for i in 0..segment_count {
            let t = (i + 1) as f32 / segment_count as f32;
            let target_pos = start + (end - start) * t;

            // Add extra chaos for seeking behavior - tendrils are more exploratory
            let chaos_amount = tendril_length * self.config.seeking_tendril_chaos * 0.3;
            let chaos_increase = t * 0.5; // More chaos toward the tip

            let x_chaos = perpendicular_x
                * chaos_amount
                * (1.0 + chaos_increase)
                * rng.random_range(-1.0..1.0);
            let y_chaos = perpendicular_y
                * chaos_amount
                * (1.0 + chaos_increase)
                * rng.random_range(-1.0..1.0);
            let z_chaos = Vec3::new(0.0, 1.0, 0.0)
                * chaos_amount
                * (1.0 + chaos_increase)
                * rng.random_range(-0.8..0.8);

            // Add enhanced "seeking" behavior - more exploratory wandering
            let seeking_offset = Vec3::new(
                rng.random_range(-1.0..1.0),
                rng.random_range(-0.3..1.2), // Enhanced upward bias for dramatic effect
                rng.random_range(-1.0..1.0),
            ) * chaos_amount
                * 0.6; // Increased seeking strength

            // Add progressive "wandering" - tendrils get more exploratory toward the tip
            let wander_strength = t * t; // Quadratic increase toward tip
            let wander_offset = Vec3::new(
                rng.random_range(-1.0..1.0),
                rng.random_range(-0.8..1.0),
                rng.random_range(-1.0..1.0),
            ) * chaos_amount
                * wander_strength
                * 0.8;

            let total_chaos = x_chaos + y_chaos + z_chaos + seeking_offset + wander_offset;
            let final_pos = target_pos + total_chaos;

            segments.push(LightningSegment {
                start: current_pos,
                end: final_pos,
                original_start: current_pos,
                original_end: final_pos,
                thickness_multiplier: self.config.seeking_tendril_thickness,
                flicker_phase: rng.random_range(0.0..std::f32::consts::PI * 2.0),
                is_branch: true,          // Mark as branch for visual distinction
                is_seeking_tendril: true, // Mark as seeking tendril for extended duration
            });

            current_pos = final_pos;
        }

        Some(segments)
    }

    /// Get a perpendicular vector for creating offsets (legacy method)
    fn get_perpendicular_vector(&self, direction: Vec3) -> Vec3 {
        // Find a vector perpendicular to the direction
        let up = Vec3::new(0.0, 1.0, 0.0);
        let side = direction.cross(&up);

        if side.magnitude() < 0.01 {
            // Direction is nearly vertical, use a different perpendicular
            Vec3::new(1.0, 0.0, 0.0)
        } else {
            side.normalize()
        }
    }

    /// Get two perpendicular vectors for true 3D branching
    fn get_3d_perpendicular_vectors(&self, direction: Vec3) -> (Vec3, Vec3) {
        // Create a robust 3D coordinate system
        let dir_normalized = direction.normalize();

        // Choose a reference vector that's not parallel to direction
        let reference = if dir_normalized.y.abs() < 0.9 {
            Vec3::new(0.0, 1.0, 0.0) // Use up if direction isn't mostly vertical
        } else {
            Vec3::new(1.0, 0.0, 0.0) // Use right if direction is mostly vertical
        };

        // Create orthonormal basis
        let perpendicular_x = dir_normalized.cross(&reference).normalize();
        let perpendicular_y = dir_normalized.cross(&perpendicular_x).normalize();

        (perpendicular_x, perpendicular_y)
    }

    /// Update the lightning animation
    pub fn update(&mut self, delta_time: f32) {
        self.time_alive += delta_time;

        // Progressive formation animation
        let formation_progress = if self.config.enable_progressive_formation {
            let formation_time = self.segments.len() as f32 / self.config.formation_speed;
            (self.time_alive / formation_time).min(1.0)
        } else {
            1.0 // Instant formation
        };

        let visible_segments = (self.segments.len() as f32 * formation_progress) as usize;

        // Animate each segment with enhanced flickering
        for (i, segment) in self.segments.iter_mut().enumerate() {
            if i >= visible_segments {
                // Hide segments that haven't formed yet
                segment.thickness_multiplier = 0.0;
                continue;
            }

            // Restore thickness for visible segments
            let base_thickness = if segment.is_branch { 0.4 } else { 1.0 };

            // Multi-layered flickering for more realistic effect
            let primary_flicker =
                (self.time_alive * self.config.animation_speed + segment.flicker_phase).sin();
            let secondary_flicker =
                (self.time_alive * self.config.animation_speed * 1.7 + segment.flicker_phase + 1.0)
                    .sin();
            let tertiary_flicker =
                (self.time_alive * self.config.animation_speed * 2.3 + segment.flicker_phase + 2.0)
                    .sin();

            let combined_flicker =
                primary_flicker * 0.6 + secondary_flicker * 0.3 + tertiary_flicker * 0.1;
            let flicker_intensity = 0.08 + 0.04 * combined_flicker; // More subtle but complex flickering

            // Apply realistic electrical disturbance
            let direction = (segment.original_end - segment.original_start).normalize();
            let perpendicular = Self::calculate_perpendicular_vector(direction);
            let perpendicular2 = direction.cross(&perpendicular).normalize();

            // Multi-dimensional flickering for more realistic electrical behavior
            let offset_1 = perpendicular * flicker_intensity * combined_flicker;
            let offset_2 = perpendicular2 * flicker_intensity * (combined_flicker * 0.7 + 0.3);
            let total_offset = offset_1 + offset_2;

            segment.start = segment.original_start + total_offset;
            segment.end = segment.original_end + total_offset;

            // Dynamic thickness variation
            let thickness_variation = 1.0 + 0.2 * primary_flicker;
            segment.thickness_multiplier = base_thickness * thickness_variation;

            // Formation edge effect - segments near the formation edge have enhanced energy
            if formation_progress < 1.0 {
                let edge_distance = (visible_segments as f32 - i as f32).abs();
                if edge_distance < 3.0 {
                    let edge_intensity = (3.0 - edge_distance) / 3.0;
                    segment.thickness_multiplier *= 1.0 + edge_intensity * 0.5; // Brighter at formation edge
                }
            }
        }
    }

    /// Calculate a perpendicular vector for creating offsets (static version)
    fn calculate_perpendicular_vector(direction: Vec3) -> Vec3 {
        // Find a vector perpendicular to the direction
        let up = Vec3::new(0.0, 1.0, 0.0);
        let side = direction.cross(&up);

        if side.magnitude() < 0.01 {
            // Direction is nearly vertical, use a different perpendicular
            Vec3::new(1.0, 0.0, 0.0)
        } else {
            side.normalize()
        }
    }

    /// Check if the lightning bolt is still alive
    pub fn is_alive(&self) -> bool {
        // Check if any segments are still alive
        let main_duration = self.config.duration;
        let tendril_duration = main_duration * self.config.seeking_tendril_duration_multiplier;
        let has_tendrils = self.segments.iter().any(|s| s.is_seeking_tendril);

        // Main bolt is alive
        if self.time_alive < main_duration {
            return true;
        }

        // Check if any seeking tendrils are still alive
        if self.time_alive < tendril_duration {
            // Only alive if we have seeking tendrils
            return has_tendrils;
        }

        false
    }

    /// Get the current opacity based on lifetime with enhanced effects
    pub fn get_opacity(&self) -> f32 {
        if self.time_alive >= self.config.duration {
            return 0.0;
        }

        // Enhanced opacity curve with initial flash and gradual fade
        let lifetime_progress = self.time_alive / self.config.duration;

        if lifetime_progress < 0.1 {
            // Initial bright flash
            let flash_progress = lifetime_progress / 0.1;
            0.3 + 0.7 * (1.0 - flash_progress)
        } else if lifetime_progress < 0.7 {
            // Stable middle phase with subtle pulsing
            let pulse = (self.time_alive * self.config.animation_speed * 0.5).sin() * 0.1;
            0.9 + pulse
        } else {
            // Gradual fade with occasional flickers
            let fade_progress = (lifetime_progress - 0.7) / 0.3;
            let flicker = if (self.time_alive * 15.0).sin() > 0.8 {
                0.3
            } else {
                0.0
            };
            (1.0 - fade_progress * 0.8) + flicker * (1.0 - fade_progress)
        }
    }

    /// Get dynamic color variation for enhanced visual effects
    fn get_dynamic_color(
        &self,
        base_color: crate::graphics::Color,
        segment: &LightningSegment,
    ) -> crate::graphics::Color {
        let time_factor = self.time_alive * self.config.animation_speed;

        // Create color variations based on segment properties
        let energy_pulse = (time_factor + segment.flicker_phase).sin() * 0.5 + 0.5;
        let secondary_pulse = (time_factor * 1.3 + segment.flicker_phase + 1.0).sin() * 0.5 + 0.5;

        // Different color for seeking tendrils vs branches vs main bolt
        let (red_component, green_component, blue_component) = if segment.is_seeking_tendril {
            // Seeking tendrils: slightly cyan/blue-white for distinction
            let cyan_shift = 0.2 * energy_pulse;
            (
                (base_color.r - cyan_shift * 0.3).max(0.0),
                (base_color.g + cyan_shift * 0.1).min(1.0),
                (base_color.b + cyan_shift).min(1.0),
            )
        } else if segment.is_branch {
            // Branch segments have slightly different color temperature
            let color_shift = 0.1;
            (
                (base_color.r + color_shift * energy_pulse).min(1.0),
                base_color.g,
                (base_color.b + 0.2 * secondary_pulse).min(1.0),
            )
        } else {
            // Main bolt uses base color with subtle variations
            (
                base_color.r,
                base_color.g,
                (base_color.b + 0.2 * secondary_pulse).min(1.0),
            )
        };

        crate::graphics::Color::new(red_component, green_component, blue_component, base_color.a)
    }

    /// Get opacity for a specific segment based on its type and duration
    fn get_segment_opacity(&self, segment: &LightningSegment) -> f32 {
        if segment.is_seeking_tendril {
            // Seeking tendrils have extended duration
            let tendril_duration =
                self.config.duration * self.config.seeking_tendril_duration_multiplier;
            if self.time_alive >= tendril_duration {
                return 0.0;
            }

            let lifetime_progress = self.time_alive / tendril_duration;

            if lifetime_progress < 0.1 {
                // Initial bright flash
                let flash_progress = lifetime_progress / 0.1;
                0.6 + 0.4 * (1.0 - flash_progress) // Much brighter for better visibility
            } else if lifetime_progress < 0.6 {
                // Stable middle phase with subtle pulsing
                let pulse = (self.time_alive * self.config.animation_speed * 0.3).sin() * 0.08;
                0.9 + pulse // Almost as bright as main bolt
            } else {
                // Gradual fade with occasional flickers
                let fade_progress = (lifetime_progress - 0.6) / 0.4;
                let flicker = if (self.time_alive * 12.0).sin() > 0.85 {
                    0.3
                } else {
                    0.0
                };
                (0.9 - fade_progress * 0.7) + flicker * (1.0 - fade_progress)
            }
        } else {
            // Main bolt and regular branches use original opacity
            self.get_opacity()
        }
    }

    /// Generate line instances for rendering with advanced visual effects
    pub fn get_line_instances(&self) -> Vec<LineInstance> {
        let mut instances = Vec::new();

        for segment in &self.segments {
            // Skip invisible segments (thickness = 0 means not yet formed)
            if segment.thickness_multiplier <= 0.0 {
                continue;
            }

            // Calculate opacity based on segment type and duration
            let segment_opacity = self.get_segment_opacity(segment);

            // Skip completely transparent segments
            if segment_opacity <= 0.0 {
                continue;
            }

            let thickness = self.config.thickness * segment.thickness_multiplier;

            // Apply dynamic color variations
            let dynamic_color = self.get_dynamic_color(self.config.color, segment);
            let mut final_color = dynamic_color;
            final_color.a *= segment_opacity;

            // Create core lightning bolt with full intensity
            instances.push(LineInstance {
                start_pos: [segment.start.x, segment.start.y, segment.start.z],
                end_pos: [segment.end.x, segment.end.y, segment.end.z],
                thickness,
                color: [final_color.r, final_color.g, final_color.b, final_color.a],
            });

            // Add outer glow effect for spectacular appearance
            if thickness > 0.02 && final_color.a > 0.3 {
                let glow_thickness = thickness * 2.5;
                let mut glow_color = final_color;
                glow_color.a *= 0.2; // Much more transparent glow

                // Slightly blue-shifted glow for electrical realism
                glow_color.b = (glow_color.b + 0.3).min(1.0);
                glow_color.r *= 0.7;

                instances.push(LineInstance {
                    start_pos: [segment.start.x, segment.start.y, segment.start.z],
                    end_pos: [segment.end.x, segment.end.y, segment.end.z],
                    thickness: glow_thickness,
                    color: [glow_color.r, glow_color.g, glow_color.b, glow_color.a],
                });
            }

            // Add inner hot core for main segments (not branches)
            if !segment.is_branch && thickness > 0.05 && final_color.a > 0.5 {
                let core_thickness = thickness * 0.3;
                let mut core_color = crate::graphics::Color::new(1.0, 1.0, 0.9, final_color.a); // Bright white-yellow core
                core_color.a *= 1.2; // Extra bright core

                instances.push(LineInstance {
                    start_pos: [segment.start.x, segment.start.y, segment.start.z],
                    end_pos: [segment.end.x, segment.end.y, segment.end.z],
                    thickness: core_thickness,
                    color: [
                        core_color.r,
                        core_color.g,
                        core_color.b,
                        core_color.a.min(1.0),
                    ],
                });
            }
        }

        instances
    }
}

/// Manager for multiple lightning effects
pub struct LightningEffectManager {
    active_bolts: Vec<LightningBolt>,
    line_instances_buffer: Vec<LineInstance>, // Reused buffer to avoid allocations
}

impl LightningEffectManager {
    pub fn new() -> Self {
        Self {
            active_bolts: Vec::new(),
            line_instances_buffer: Vec::new(),
        }
    }

    /// Spawn a new lightning bolt
    pub fn spawn_lightning(&mut self, start: Vec3, end: Vec3, config: Option<LightningConfig>) {
        let config = config.unwrap_or_default();
        let bolt = LightningBolt::new(start, end, config);
        self.active_bolts.push(bolt);
    }

    /// Update all active lightning bolts
    pub fn update(&mut self, delta_time: f32) {
        // Update all bolts
        for bolt in &mut self.active_bolts {
            bolt.update(delta_time);
        }

        // Remove expired bolts
        self.active_bolts.retain(|bolt| bolt.is_alive());
    }

    /// Get all line instances for rendering - this feeds into ReusableLineBatch
    /// Reuses internal buffer to avoid allocations
    pub fn get_line_instances(&mut self) -> Vec<LineInstance> {
        self.line_instances_buffer.clear();

        for bolt in &self.active_bolts {
            self.line_instances_buffer.extend(bolt.get_line_instances());
        }

        // Return a copy (this is still faster than allocating every time)
        self.line_instances_buffer.clone()
    }

    /// Get the number of active lightning bolts
    pub fn active_count(&self) -> usize {
        self.active_bolts.len()
    }
}
