use std::time::{Duration, Instant};

pub struct FpsLimiter {
    // target_fps: u32,
    frame_duration: Duration,
    last_frame_time: Instant,
}

impl FpsLimiter {
    pub fn new(target_fps: u32) -> Self {
        assert!(target_fps > 0, "Target FPS must be greater than 0");
        let frame_duration = Duration::from_secs_f64(1.0 / f64::from(target_fps).max(0.0001)); // augh
        let last_frame_time = Instant::now();
        FpsLimiter {
            // target_fps,
            frame_duration,
            last_frame_time,
        }
    }

    pub fn limit_fps(&mut self) {
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
