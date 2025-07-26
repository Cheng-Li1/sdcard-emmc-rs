#![no_std] // Don't link the standard library
#![no_main] // Don't use the default entry point

extern crate alloc;

mod sel4_microkit_os;

use sdhci::sdhci::SdhciHost;
use sdmmc_hal::sdhci_arasan::SdhciArasan;
use sdmmc_protocol::sdmmc_traits::SdmmcHardware;
use sdmmc_protocol::{
    sdmmc::SdmmcProtocol,
    sdmmc_os::{Sleep, VoltageOps},
};
use sel4_microkit::{Handler, Infallible, debug_print, debug_println, protection_domain};

use crate::sel4_microkit_os::TimerOps;
use crate::sel4_microkit_os::{SerialOps, odroidc4::Odroidc4VoltageSwitch};

const TIMER: TimerOps = TimerOps::new();
const SERIAL: SerialOps = SerialOps::new();

// Debug function for printing out content in one block
#[allow(dead_code)]
unsafe fn print_one_block(ptr: *const u8, num: usize) {
    unsafe {
        // Iterate over the number of bytes and print each one in hexadecimal format
        for i in 0..num {
            let byte = *ptr.add(i);
            if i.is_multiple_of(16) {
                debug_print!("\n{:04x}: ", i);
            }
            debug_print!("{:02x} ", byte);
        }
        debug_println!();
    }
}

/// Since in .system file, the page we are providing to tune_performance function is uncached
/// we do not need to provide a real cache invalidate function
fn dummy_cache_invalidate_function() {}

#[allow(dead_code)]
unsafe fn set_one_block(ptr: *mut u8, num: usize) {
    unsafe {
        // Iterate over the number of bytes and print each one in hexadecimal format
        for i in 0..num {
            *ptr.add(i) = i as u8;
        }
    }
}

#[protection_domain(heap_size = 0x1000)]
fn init() -> impl Handler {
    debug_println!("Driver init!");
    // Enable the debug print
    unsafe {
        sdmmc_protocol::sdmmc_os::set_logger(&SERIAL).unwrap();
    }

    // This line of code actually is very unsafe!
    // Considering the memory is stolen from the memory that has sdcard registers mapped in
    let unsafe_stolen_memory: *mut [u8; 384] = 0x70000000 as *mut [u8; 384];
    let physical_memory_addr: u64 = 0x70000000;

    assert!((physical_memory_addr as usize).is_multiple_of(8));

    unsafe { set_one_block(0x70010000 as *mut u8, 512) };
    unsafe { print_one_block(0x70010000 as *mut u8, 512) };

    let hal = unsafe {
        SdhciHost::new(
            0xff170000,
            unsafe_stolen_memory,
            dummy_cache_invalidate_function,
            physical_memory_addr as u32,
            SdhciArasan::new()
        )
    };

    // Handling result in two different ways, by matching and unwrap_or_else
    let res = SdmmcProtocol::new(hal, TIMER, None::<Odroidc4VoltageSwitch>);
    let mut sdmmc_host = match res {
        Ok(host) => host,
        Err(err) => panic!("SDMMC: Error at init {:?}", err),
    };

    sdmmc_host
        .setup_card()
        .unwrap_or_else(|error| panic!("SDMMC: Error at setup {:?}", error));

    let unsafe_stolen_memory2 = 0x70019000;
    if let Err(_) = sdmmc_host.get_hal().read_one_block_no_dma(0, unsafe_stolen_memory2) {
        panic!("SDMMC: read_one_block_no_dma failed");
    }
    unsafe {
        print_one_block(unsafe_stolen_memory2 as *const u8, 512);
    }

    // Print the card info after the init process
    sdmmc_host.print_card_info();

    let _ = sdmmc_host.config_interrupt(false, false);

    // Print out one block to check if read works
    sdmmc_host.test_read_one_block(0, 0x70010000);

    todo!();

    /*
    unsafe {
        sdmmc_host
            .tune_performance(
                unsafe_stolen_memory,
                dummy_cache_invalidate_function,
                physical_memory_addr,
            )
            .unwrap_or_else(|error| panic!("SDMMC: Error at tuning performance {:?}", error));
    }

    unsafe {
        print_one_block(unsafe_stolen_memory as *const u8, 64);
    }

    // Should always succeed, at least for odroid C4
    sdmmc_host.config_interrupt(true, false).unwrap();
    */

    HandlerImpl {
        sdmmc: Some(sdmmc_host),
    }
}

struct HandlerImpl<T: SdmmcHardware, S: Sleep, V: VoltageOps> {
    sdmmc: Option<SdmmcProtocol<T, S, V>>,
}

impl<T: SdmmcHardware + 'static, S: Sleep + 'static, V: VoltageOps + 'static> Handler
    for HandlerImpl<T, S, V>
{
    type Error = Infallible;
}
