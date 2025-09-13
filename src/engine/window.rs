use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{Event, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use crate::engine::{Engine, Vec3};
use crate::graphics::GraphicsEngine;
use crate::input::{Action, InputManager};

pub struct WindowManager {
    window: Option<Arc<Window>>,
    graphics_engine: Option<GraphicsEngine>,
    engine: Engine,
    input_manager: InputManager,
    last_frame: std::time::Instant,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            window: None,
            graphics_engine: None,
            engine: Engine::new(),
            input_manager: InputManager::new(),
            last_frame: std::time::Instant::now(),
        }
    }

    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);

        println!("Starting HB3D Game Engine...");

        event_loop.run_app(&mut self)?;
        Ok(())
    }
}

impl ApplicationHandler for WindowManager {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("HB3D Game Engine")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 960));

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        let graphics_engine =
            pollster::block_on(async { GraphicsEngine::new(Arc::clone(&window)).await.unwrap() });

        self.window = Some(window);
        self.graphics_engine = Some(graphics_engine);

        // Initialize physics GPU system now that graphics context is available
        if let Some(graphics_engine) = &self.graphics_engine {
            self.engine.initialize_physics_gpu(graphics_engine.device());
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let winit_event = Event::WindowEvent {
            window_id: _window_id,
            event: event.clone(),
        };
        self.input_manager.handle_event(&winit_event);

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Calculate delta time
                let now = std::time::Instant::now();
                let delta_time = now.duration_since(self.last_frame).as_secs_f32();
                self.last_frame = now;

                // Update input state
                self.input_manager.update();

                // Get camera vectors for movement (including Y axis)
                let (camera_forward, camera_right, camera_up) =
                    if let Some(graphics_engine) = &self.graphics_engine {
                        let camera = graphics_engine.camera();
                        let forward = camera.forward();
                        let right = camera.right();
                        let up = camera.up();
                        (
                            Vec3::new(forward.x, forward.y, forward.z), // Keep full 3D direction
                            Vec3::new(right.x, right.y, right.z),
                            Vec3::new(up.x, up.y, up.z),
                        )
                    } else {
                        // Fallback vectors if no graphics engine
                        (
                            Vec3::new(0.0, 0.0, -1.0),
                            Vec3::new(1.0, 0.0, 0.0),
                            Vec3::new(0.0, 1.0, 0.0),
                        )
                    };

                // Single engine update that handles everything with GPU physics
                let (primitives, player_pos, graphics_events) =
                    if let Some(graphics_engine) = &self.graphics_engine {
                        // Pass GPU context for physics calculations
                        self.engine.update_with_gpu(
                            delta_time,
                            &self.input_manager,
                            camera_forward,
                            camera_right,
                            camera_up,
                            Some(graphics_engine.device()),
                            Some(graphics_engine.queue()),
                        )
                    } else {
                        // Fallback without GPU context
                        self.engine.update(
                            delta_time,
                            &self.input_manager,
                            camera_forward,
                            camera_right,
                            camera_up,
                        )
                    };

                // Process graphics events from dispatcher
                if let Some(graphics_engine) = &mut self.graphics_engine {
                    for event in graphics_events {
                        match event {
                            crate::engine::dispatcher::GraphicsEvent::SpawnParticles {
                                position,
                                velocity,
                                count,
                                lifetime,
                                color,
                            } => {
                                graphics_engine
                                    .spawn_particles(position, velocity, count, lifetime, color);
                            }
                            crate::engine::dispatcher::GraphicsEvent::DrawLine {
                                start,
                                end,
                                color,
                                duration,
                            } => {
                                // Debug line drawing - could be implemented with line renderer
                                println!(
                                    "📐 Debug line: {:?} -> {:?} (color: {:?}, duration: {})",
                                    start, end, color, duration
                                );
                            }
                            crate::engine::dispatcher::GraphicsEvent::ScreenShake {
                                intensity,
                                duration,
                            } => {
                                // Screen shake - could be implemented by modifying camera offset
                                println!(
                                    "📳 Screen shake: intensity {}, duration {}",
                                    intensity, duration
                                );
                            }
                        }
                    }
                }

                // GPU collision detection
                // Always use GPU collision detection
                let use_gpu_collisions = true;

                if use_gpu_collisions {
                    if let Some(graphics_engine) = &mut self.graphics_engine {
                        // Update collision compute system with current entity states
                        let player = self.engine.scheduler.player().player();
                        let enemies = self.engine.scheduler.enemies().enemies();
                        let bullets = self.engine.scheduler.bullets().bullets();
                        let metabullets = self.engine.scheduler.bullets().metabullets();
                        let large_bodies = self.engine.scheduler.large_bodies().bodies();

                        graphics_engine.collision_compute().update_entities(
                            player,
                            enemies,
                            bullets,
                            metabullets,
                            large_bodies,
                        );

                        // Dispatch and read GPU collision results
                        match graphics_engine.dispatch_and_read_collisions() {
                            Ok(collision_pairs) => {
                                if !collision_pairs.is_empty() {
                                    // Convert CollisionPair to simple tuples
                                    let pairs: Vec<(u32, u32)> = collision_pairs
                                        .iter()
                                        .map(|p| (p.entity_a, p.entity_b))
                                        .collect();

                                    // Print a copy of every single collision pair
                                    // println!("GPU collision pairs: {:?}", pairs);

                                    // Process GPU collision pairs
                                    self.engine.process_gpu_collisions(&pairs);
                                }
                            }
                            Err(e) => {
                                eprintln!("GPU collision error: {}, falling back to CPU", e);
                                self.engine.scheduler.check_collisions_cpu();
                            }
                        }
                        
                        // Process large body collisions every frame (independent of GPU collisions)
                        self.engine.process_large_body_collisions();
                        
                        // Handle collision cleanup and event dispatch every frame
                        self.engine.process_collision_cleanup();
                    }
                } else {
                    // CPU collision detection fallback
                    self.engine.scheduler.check_collisions_cpu();
                }

                if let Some(graphics_engine) = &mut self.graphics_engine {
                    // Handle camera movement from right thumbstick and mouse
                    let camera_look_h = self
                        .input_manager
                        .get_action_value(Action::CameraLookHorizontal);
                    let camera_look_v = self
                        .input_manager
                        .get_action_value(Action::CameraLookVertical);

                    // Get mouse movement for camera control
                    let (mouse_dx, mouse_dy) = self.input_manager.mouse.delta();
                    let mouse_look = self
                        .input_manager
                        .mouse
                        .is_button_pressed(winit::event::MouseButton::Right);

                    // Apply camera rotation
                    let camera = graphics_engine.camera_mut();

                    // Right thumbstick camera controls
                    if camera_look_h != 0.0 || camera_look_v != 0.0 {
                        camera.angle_horizontal -= camera_look_h * delta_time * 2.0; // Negated to fix left/right
                        camera.angle_vertical = (camera.angle_vertical
                            - camera_look_v * delta_time * 1.5)
                            .clamp(-1.4, 1.4);
                    }

                    // Mouse look (when right mouse button is held)
                    if mouse_look && (mouse_dx != 0.0 || mouse_dy != 0.0) {
                        camera.angle_horizontal -= (mouse_dx * 0.005) as f32; // Negated to fix left/right
                        camera.angle_vertical =
                            (camera.angle_vertical - (mouse_dy * 0.005) as f32).clamp(-1.4, 1.4);
                    }

                    // Update camera to follow player
                    graphics_engine.update_camera(nalgebra::Point3::new(
                        player_pos.x,
                        player_pos.y,
                        player_pos.z,
                    ));

                    // Update particle system
                    graphics_engine.update_particles(delta_time);

                    // Render the primitives
                    if let Err(e) = graphics_engine.render(&primitives) {
                        eprintln!("Render error: {}", e);
                    }
                }

                // Clear transient input state
                self.input_manager.end_frame();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(graphics_engine) = &mut self.graphics_engine {
                    graphics_engine.resize(new_size);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
