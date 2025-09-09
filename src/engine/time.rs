use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct TimeManager {
    start_time: Instant,
    last_frame_time: Instant,
    delta_time: f32,
    total_time: f32,
    frame_count: u64,
    fps_timer: f32,
    fps: f32,
}

impl TimeManager {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_frame_time: now,
            delta_time: 0.0,
            total_time: 0.0,
            frame_count: 0,
            fps_timer: 0.0,
            fps: 0.0,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.delta_time = delta_time;
        self.total_time += delta_time;
        self.frame_count += 1;
        
        self.fps_timer += delta_time;
        if self.fps_timer >= 1.0 {
            // Calculate FPS as frames per second over the actual elapsed time
            self.fps = self.frame_count as f32 / self.fps_timer;
            self.fps_timer = 0.0;
            self.frame_count = 0;
        }
    }

    pub fn tick(&mut self) -> f32 {
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;
        
        let delta_seconds = delta.as_secs_f32();
        self.update(delta_seconds);
        delta_seconds
    }

    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }

    pub fn total_time(&self) -> f32 {
        self.total_time
    }

    pub fn fps(&self) -> f32 {
        self.fps
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    pub fn elapsed_since_start(&self) -> Duration {
        self.start_time.elapsed()
    }
}