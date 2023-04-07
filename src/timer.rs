//! Used to control the update interval of modules.
use std::time::Duration;

use tokio::time::{interval_at, Instant, Interval, MissedTickBehavior};

#[derive(Clone, Debug)]
pub struct Timer {
    period_seconds: u64,
}

impl Default for Timer {
    fn default() -> Self {
        Self { period_seconds: 10 }
    }
}

impl Timer {
    pub fn new(period: u64) -> Self {
        Self {
            period_seconds: period,
        }
    }

    pub fn start(self) -> Interval {
        let start = Instant::now() + Duration::from_secs(self.period_seconds);
        let mut timer = interval_at(start, Duration::from_secs(self.period_seconds));
        timer.set_missed_tick_behavior(MissedTickBehavior::Delay);
        timer
    }
}
