use sddf_timer::timer::Timer;
use sdmmc_protocol::sdmmc_os::Sleep;

pub mod odroidc4;

const NS_IN_US: u64 = 1000;

impl Sleep for Timer {
    fn usleep(&mut self, time_us: u32) {
        self.set_timeout(time_us as u64 * NS_IN_US);
    }
}
