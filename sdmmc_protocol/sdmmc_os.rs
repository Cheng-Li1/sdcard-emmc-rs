#[cfg(feature = "sel4-microkit")]
pub use sel4_microkit_support::process_wait_unreliable;

#[cfg(feature = "sel4-microkit")]
pub use sel4_microkit_support::debug_log;

use crate::sdmmc::MmcPowerMode;
use crate::sdmmc::MmcSignalVoltage;
use crate::sdmmc::SdmmcError;

#[cfg(not(feature = "sel4-microkit"))]
#[inline]
pub fn process_wait_unreliable(time_ns: u64) {
    for _ in 0..time_ns {
        hint::spin_loop(); // Use spin loop hint to reduce contention during the wait
    }
}

/// Bare metal
#[cfg(not(feature = "sel4-microkit"))]
#[macro_export]
// No operation, would be optimized out
macro_rules! debug_log {
    ($($arg:tt)*) => {};
}

pub trait Sleep {
    /// For putting the process to sleep for a while,
    /// The default spinning implementation is a very unreliable way to put the process to sleep
    fn usleep(&mut self, time_us: u32) {
        let time_ns: u64 = time_us as u64 * 1000;
        for _ in 0..time_ns {
            core::hint::spin_loop(); // Use spin loop hint to reduce contention during the wait
        }
    }
}

pub trait VoltageSwitch {
    fn card_voltage_switch(&mut self, voltage: MmcSignalVoltage) -> Result<(), SdmmcError> {
        core::panic!("Voltage switch not implemented!");
    }

    fn card_set_power(&mut self, power_mode: MmcPowerMode) -> Result<(), SdmmcError> {
        core::panic!("Power cycling not implemented!");
    }
}
