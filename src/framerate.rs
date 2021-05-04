use std::time::Instant;

#[derive(Debug)]
pub struct Framerate {
    pub current_fps: u32,
    last_time: Instant,
    delta_time: f64,
    frame_count: u32,
    frame_time: f64,
}

impl Framerate {
    pub fn new() -> Self {
        Self {
            current_fps: 0,
            last_time: Instant::now(),
            delta_time: 0.0,
            frame_count: 0,
            frame_time: 0.0,
        }
    }

    fn delta(&mut self) -> f64 {
        let current_time = Instant::now();
        let delta = current_time.duration_since(self.last_time).as_micros() as f64 * 0.001;
        self.last_time = current_time;
        self.delta_time = delta;
        delta
    }

    pub fn fps(&mut self) {
        self.delta();
        self.frame_count += 1;
        self.frame_time += self.delta_time;
        let tmp;

        // per second
        if self.frame_time >= 1000.0 {
            tmp = self.frame_count;
            self.frame_count = 0;
            self.frame_time = 0.0;
            self.current_fps = tmp;
        }
    }
}
