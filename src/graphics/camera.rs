use crate::graphics::constants::*;
use bytemuck::{Pod, Zeroable};
use nalgebra::{Matrix4, Point3, Vector3};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &ThirdPersonCamera, projection: &Projection) {
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into();
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CameraMode {
    Follow, // Camera follows target at fixed offset
    Orbit,  // Camera orbits around target
    Free,   // Free camera movement
}

pub struct ThirdPersonCamera {
    // Target tracking
    pub target: Point3<f32>,
    pub target_velocity: Vector3<f32>,

    // Camera positioning
    pub distance: f32,
    pub height_offset: f32,
    pub angle_horizontal: f32, // Yaw around target
    pub angle_vertical: f32,   // Pitch up/down

    // Camera state
    pub position: Point3<f32>,
    pub up: Vector3<f32>,
    pub mode: CameraMode,

    // Movement settings
    pub rotation_speed: f32,
    pub zoom_speed: f32,
    pub follow_speed: f32,
    pub smooth_factor: f32,

    // Constraints
    pub min_distance: f32,
    pub max_distance: f32,
    pub min_pitch: f32,
    pub max_pitch: f32,

    // Input sensitivity
    pub look_sensitivity: f32,
    pub zoom_sensitivity: f32,
}

impl ThirdPersonCamera {
    pub fn new() -> Self {
        Self {
            target: Point3::new(0.0, 0.0, 0.0),
            target_velocity: Vector3::zeros(),

            distance: 15.0, // Good distance to view primitive showcase
            height_offset: 5.0,
            angle_horizontal: 0.0,
            angle_vertical: -0.3, // Look down slightly

            position: Point3::new(0.0, 2.0, 20.0), // Match distance
            up: Vector3::y(),
            mode: CameraMode::Follow,

            rotation_speed: 2.0,
            zoom_speed: CAMERA_ZOOM_SPEED,
            follow_speed: CAMERA_SMOOTHING / 2.0,
            smooth_factor: 0.5, // Much faster camera movement

            min_distance: CAMERA_ZOOM_MIN,
            max_distance: CAMERA_ZOOM_MAX,
            min_pitch: -1.5,
            max_pitch: 1.2,

            look_sensitivity: 15.0, // Much higher for responsive camera
            zoom_sensitivity: 3.0,
        }
    }

    /// Update camera with target position (call this every frame)
    pub fn update(&mut self, target_pos: Point3<f32>) {
        self.target = target_pos;

        // Calculate new camera position based on mode
        match self.mode {
            CameraMode::Follow | CameraMode::Orbit => {
                self.update_orbit_position();
            }
            CameraMode::Free => {
                // For free mode, position is controlled directly via public methods
                // No automatic update needed
            }
        }
    }

    /// Rotate camera (call from input handling)
    /// horizontal: left/right rotation (-1.0 to 1.0)  
    /// vertical: up/down rotation (-1.0 to 1.0)
    pub fn rotate(&mut self, horizontal: f32, vertical: f32, delta_time: f32) {
        if horizontal.abs() > 0.001 || vertical.abs() > 0.001 {
            self.angle_horizontal -= horizontal * self.look_sensitivity * delta_time;
            self.angle_vertical -= vertical * self.look_sensitivity * delta_time;

            // Clamp vertical angle
            self.angle_vertical = self.angle_vertical.clamp(self.min_pitch, self.max_pitch);
        }
    }

    /// Zoom camera (call from input handling)
    /// zoom_delta: positive = zoom out, negative = zoom in
    pub fn zoom(&mut self, zoom_delta: f32) {
        if zoom_delta.abs() > 0.01 {
            self.distance =
                (self.distance + zoom_delta).clamp(self.min_distance, self.max_distance);
        }
    }

    /// Move camera in free mode (call from input handling)
    /// movement: (forward/back, left/right, up/down)
    pub fn move_free(&mut self, movement: Vector3<f32>, delta_time: f32) {
        if self.mode != CameraMode::Free {
            return;
        }

        // Calculate forward and right vectors
        let forward = (self.target - self.position).normalize();
        let right = forward.cross(&self.up).normalize();
        let up = right.cross(&forward).normalize();

        // Apply movement
        let velocity = forward * movement.x + right * movement.y + up * movement.z;
        self.position += velocity * self.rotation_speed * delta_time;

        // Update target to maintain look direction
        self.target = self.position + forward;
    }

    /// Reset camera to default position
    pub fn reset(&mut self) {
        self.distance = 10.0;
        self.height_offset = 2.0;
        self.angle_horizontal = 0.0;
        self.angle_vertical = -0.3;
    }

    fn update_orbit_position(&mut self) {
        // Calculate position in spherical coordinates around target
        let cos_pitch = self.angle_vertical.cos();
        let sin_pitch = self.angle_vertical.sin();
        let cos_yaw = self.angle_horizontal.cos();
        let sin_yaw = self.angle_horizontal.sin();

        // Position relative to target
        let relative_pos = Vector3::new(
            cos_pitch * sin_yaw * self.distance,
            sin_pitch * self.distance + self.height_offset,
            cos_pitch * cos_yaw * self.distance,
        );

        let desired_position = self.target + relative_pos;

        // Direct camera positioning - no smoothing
        self.position = desired_position;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(&self.position, &self.target, &self.up)
    }

    pub fn set_target(&mut self, target: Point3<f32>) {
        self.target = target;
    }

    pub fn set_mode(&mut self, mode: CameraMode) {
        self.mode = mode;
    }

    // Get camera vectors for movement calculations
    pub fn forward(&self) -> Vector3<f32> {
        (self.target - self.position).normalize()
    }

    pub fn right(&self) -> Vector3<f32> {
        self.forward().cross(&self.up).normalize()
    }

    pub fn up(&self) -> Vector3<f32> {
        self.up
    }

    pub fn get_position(&self) -> Point3<f32> {
        self.position
    }

    pub fn get_target(&self) -> Point3<f32> {
        self.target
    }
}

impl Default for ThirdPersonCamera {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Projection {
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new(width: u32, height: u32, fovy: f32, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy,
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        Matrix4::new_perspective(self.aspect, self.fovy, self.znear, self.zfar)
    }
}
