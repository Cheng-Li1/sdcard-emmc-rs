use sdmmc_protocol::sdmmc_os::{Log, Sleep, process_wait_unreliable};
use sel4_panicking_env::__debug_print_macro_helper;

pub mod odroidc4;

const NS_IN_US: u64 = 1000;

/// Wrapper to work around Rust's orphan rule
pub struct TimerOps {}

impl TimerOps {
    pub const fn new() -> Self {
        TimerOps {}
    }
}

impl Sleep for TimerOps {
    fn usleep(&mut self, time_us: u32) {
        process_wait_unreliable(time_us as u64 * NS_IN_US);
        // self.timer.set_timeout(time_us as u64 * NS_IN_US);
    }
}

pub struct SerialOps {}

impl SerialOps {
    pub const fn new() -> Self {
        SerialOps {}
    }
}

impl Log for SerialOps {
    fn log(&self, args: core::fmt::Arguments) {
        __debug_print_macro_helper(args);
    }
}
