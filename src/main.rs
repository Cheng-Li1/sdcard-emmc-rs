#![no_std]  // Don't link the standard library
#![no_main] // Don't use the default entry point

pub mod meson_gx_mmc;

use core::panic::PanicInfo;

const SDIO_BASE: u64 = 0xffe03000;

/// Entry point for the program. This function must not return.
#[no_mangle]
pub extern "C" fn init() -> ! {
    // Place initialization code here (e.g., setting up peripherals, memory, etc.)
    
    // The main loop where your program runs
    loop {
        // In a bare-metal environment, your program usually runs indefinitely.
    }
}

/// This function is called on panic, and since there's no OS, it typically just halts the CPU.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}