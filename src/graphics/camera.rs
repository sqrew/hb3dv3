use bytemuck::{Pod, Zeroable};
use nalgebra::{Matrix4, Point3, Vector3};

// Camera constants
pub const NEW_CAMERA_DISTANCE: f32 = 15.0;
pub const NEW_CAMERA_HEIGHT_OFFSET: f32 = 5.0;
pub const NEW_CAMERA_ANGLE_HORIZONTAL: f32 = 0.0;
pub const NEW_CAMERA_ANGLE_VERTICAL: f32 = -0.3;

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
}

pub struct ThirdPersonCamera {
    // Target tracking
    pub target: Point3<f32>,

    // Camera positioning
    pub distance: f32,
    pub height_offset: f32,
    pub angle_horizontal: f32, // Yaw around target
    pub angle_vertical: f32,   // Pitch up/down

    // Camera state
    pub position: Point3<f32>,
    pub up: Vector3<f32>,
    pub mode: CameraMode,
}

impl ThirdPersonCamera {
    pub fn new() -> Self {
        Self {
            target: Point3::new(0.0, 0.0, 0.0),

            distance: NEW_CAMERA_DISTANCE,
            height_offset: NEW_CAMERA_HEIGHT_OFFSET,
            angle_horizontal: NEW_CAMERA_ANGLE_HORIZONTAL,
            angle_vertical: NEW_CAMERA_ANGLE_VERTICAL,

            position: Point3::new(0.0, 2.0, 20.0), // Match distance
            up: Vector3::y(),
            mode: CameraMode::Follow,
        }
    }

    /// Update camera with target position (call this every frame)
    pub fn update(&mut self, target_pos: Point3<f32>) {
        self.target = target_pos;

        // Calculate new camera position based on mode
        match self.mode {
            CameraMode::Follow => {
                self.update_orbit_position();
            }
        }
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
