use std::time::{Duration, Instant}; 

pub struct Timer {
  start: Instant,
  duration: Duration,
}

impl Timer {
  fn new(duration: Duration) -> Timer {
    Timer {
      start: Instant::now(),
      duration,
    }
  }

  fn remaining(&self) -> Duration {
    self.duration.checked_sub(self.start.elapsed()).unwrap_or_default()
  }

  // check if elapsed time is less than a specified fraction of remaining time
  fn is_time_remaining(&self, frac: u32) -> bool {
    self.start.elapsed() < (self.remaining() * frac / 100)
  }
}