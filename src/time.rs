use std::time::{Duration, Instant}; 

pub struct Timer {
  pub start: Instant,
  pub duration: Duration,
}

impl Timer {
  pub fn new(duration: Duration) -> Timer {
    Timer {
      start: Instant::now(),
      duration,
    }
  }

  pub fn remaining(&self) -> Duration {
    self.duration.checked_sub(self.start.elapsed()).unwrap_or_default()
  }

  // check if elapsed time is less than a specified fraction of remaining time
  pub fn is_time_remaining(&self, frac: u32) -> bool {
    self.start.elapsed() < (self.remaining() * frac / 100)
  }

  pub fn is_time_remaining_5(&self) -> bool { // > 5% remaining time
    self.is_time_remaining(5)
  }
}