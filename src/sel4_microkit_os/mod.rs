use sddf_timer::timer::Timer;
use sdmmc_protocol::sdmmc_os::Sleep;

pub mod odroidc4;

const NS_IN_US: u64 = 1000;

/// Wrapper to work around Rust's orphan rule
pub struct TimerOps {
    timer: Timer,
}

impl TimerOps {
    pub fn new(timer: Timer) -> Self {
        TimerOps { timer }
    }
}

impl Sleep for TimerOps {
    fn usleep(&mut self, time_us: u32) {
        self.timer.set_timeout(time_us as u64 * NS_IN_US);
    }
}
