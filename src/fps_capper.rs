use std::time::{Duration, Instant};

pub struct FpsLimiter {
    // target_fps: u32,
    frame_duration: Duration,
    last_frame_time: Instant,
    target_fps: i32
}

impl FpsLimiter {
    pub fn new(target_fps: i32) -> Self {
        if target_fps >= 0 {        
            let frame_duration = Duration::from_secs_f64(1.0 / f64::from(target_fps as u32).max(0.0001)); // augh
            let last_frame_time = Instant::now();
            FpsLimiter {
                // target_fps,
                frame_duration,
                last_frame_time,
                target_fps
            }
        } else {
            FpsLimiter {
                frame_duration: std::time::Duration::from_secs(0),
                last_frame_time: std::time::Instant::now(),
                target_fps
            }
        }
    }

    pub fn limit_fps(&mut self) {
        if self.target_fps != 0 {
            let elapsed_time = self.last_frame_time.elapsed();
            if elapsed_time < self.frame_duration {
                let sleep_duration = self.frame_duration - elapsed_time;
                if sleep_duration > Duration::from_secs(0) {
                    std::thread::sleep(sleep_duration);
                }
            }
            self.last_frame_time = Instant::now();
        }
    }
}
