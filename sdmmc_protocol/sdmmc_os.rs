#[cfg(feature = "sel4-microkit")] 
#[inline]
pub fn process_wait_unreliable(time_ns: u64) {
    sel4_microkit_support::process_wait_unreliable(time_ns);
}

#[cfg(not(feature = "sel4-microkit"))]
#[inline]
pub fn process_wait_unreliable(time_ns: u64) {
    for _ in 0..time_ns {
        hint::spin_loop(); // Use spin loop hint to reduce contention during the wait
    }
}

// `sel4-microkit` specific implementation
#[cfg(feature = "sel4-microkit")]
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        sel4_microkit::debug_println!($($arg)*);
    }
}

/// Bare metal
#[cfg(not(feature = "sel4-microkit"))]
#[macro_export]
// No operation, would be optimized out
macro_rules! debug_log {
    ($($arg:tt)*) => {};
}