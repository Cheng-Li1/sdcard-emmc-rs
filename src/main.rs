#![no_std]  // Don't link the standard library
#![no_main] // Don't use the default entry point

use core::{future::Future, pin::Pin, task::{Context, Poll, RawWaker, RawWakerVTable, Waker}};

use sdmmc_hal::meson_gx_mmc::MesonSdmmcRegisters;

use sdmmc_protocol::sdmmc::{MmcData, MmcDataFlag, SdmmcCmd, SdmmcHalError, SdmmcProtocol, MMC_RSP_NONE, MMC_RSP_R1, MMC_RSP_R7};
use sel4_microkit::{debug_print, debug_println, protection_domain, Handler, Infallible};

const SDMMC_BASE_ADDR: *mut MesonSdmmcRegisters = 0xffe05000 as *mut MesonSdmmcRegisters;
const DATA_ADDR: *mut u8 = 0x50000000 as *mut u8;

fn print_one_block(ptr: *const u8) {
    unsafe {
        // Iterate over the 512 bytes and print each one in hexadecimal format
        for i in 0..512 {
            let byte = *ptr.add(i);
            if i % 16 == 0 {
                debug_print!("\n{:04x}: ", i);
            }
            debug_print!("{:02x} ", byte);
        }
        debug_println!();
    }
}

// No-op waker implementations, they do nothing.
unsafe fn noop(_data: *const ()) {}
unsafe fn noop_clone(_data: *const ()) -> RawWaker {
    RawWaker::new(_data, &VTABLE)
}

// A VTable that points to the no-op functions
static VTABLE: RawWakerVTable = RawWakerVTable::new(
    noop_clone,
    noop,
    noop,
    noop,
);

// Function to create a dummy Waker
fn create_dummy_waker() -> Waker {
    let raw_waker = RawWaker::new(core::ptr::null(), &VTABLE);
    unsafe { Waker::from_raw(raw_waker) }
}

#[protection_domain]
fn init() -> HandlerImpl {
    debug_println!("Driver init!");
    let meson_hal = MesonSdmmcRegisters::new();
    let mut protocol = SdmmcProtocol::new(meson_hal);
    let mut future = protocol.read_block(1, 0, 0x50000000);
    // It is safe for now as our activities are only in this init block so future will not be moved
    // But this is only for testing
    let mut pinned_future;
    unsafe {
        pinned_future = Pin::new_unchecked(&mut future);
    }
    
    // Create a context with a dummy waker
    let waker = create_dummy_waker();
    let mut cx = Context::from_waker(&waker);

    loop {
        match pinned_future.as_mut().poll(&mut cx) {
            Poll::Ready(result) => {
                debug_println!("Future completed with result");
                break;
            }
            Poll::Pending => {
                debug_println!("Future is not ready, polling again...");
                // In a real case, you would block or wait here
            }
        }
    }
    
    print_one_block(DATA_ADDR);

    HandlerImpl
}

struct HandlerImpl;

impl Handler for HandlerImpl {
    type Error = Infallible;
}

fn parse_cfg(cfg_register: u32) {
    // Bits 31:28 - Cfg_ip_txd_adj
    let ip_txd_adj = (cfg_register >> 28) & 0xF; // 4-bit field
    debug_println!("Cfg_ip_txd_adj (bits 31:28): {}", ip_txd_adj);

    // Bit 27 - Cfg_err_abort
    let err_abort = (cfg_register >> 27) & 0x1;
    debug_println!("Cfg_err_abort (bit 27): {}", err_abort);

    // Bit 26 - Cfg_irq_ds
    let irq_ds = (cfg_register >> 26) & 0x1;
    debug_println!("Cfg_irq_ds (bit 26): {}", irq_ds);

    // Bit 25 - Cfg_txd_retry
    let txd_retry = (cfg_register >> 25) & 0x1;
    debug_println!("Cfg_txd_retry (bit 25): {}", txd_retry);

    // Bit 24 - Cfg_txd_add_err
    let txd_add_err = (cfg_register >> 24) & 0x1;
    debug_println!("Cfg_txd_add_err (bit 24): {}", txd_add_err);

    // Bit 23 - Cfg_auto_clk
    let auto_clk = (cfg_register >> 23) & 0x1;
    debug_println!("Cfg_auto_clk (bit 23): {}", auto_clk);

    // Bit 22 - Cfg_stop_clk
    let stop_clk = (cfg_register >> 22) & 0x1;
    debug_println!("Cfg_stop_clk (bit 22): {}", stop_clk);

    // Bit 21 - Cfg_cmd_low
    let cmd_low = (cfg_register >> 21) & 0x1;
    debug_println!("Cfg_cmd_low (bit 21): {}", cmd_low);

    // Bit 20 - Reserved (skip this)

    // Bit 19 - Cfg_ignore_owner
    let ignore_owner = (cfg_register >> 19) & 0x1;
    debug_println!("Cfg_ignore_owner (bit 19): {}", ignore_owner);

    // Bit 18 - Cfg_sdclk_always_on
    let sdclk_always_on = (cfg_register >> 18) & 0x1;
    debug_println!("Cfg_sdclk_always_on (bit 18): {}", sdclk_always_on);

    // Bit 17 - Cfg_blk_gap_ip
    let blk_gap_ip = (cfg_register >> 17) & 0x1;
    debug_println!("Cfg_blk_gap_ip (bit 17): {}", blk_gap_ip);

    // Bit 16 - Cfg_out_fall
    let out_fall = (cfg_register >> 16) & 0x1;
    debug_println!("Cfg_out_fall (bit 16): {}", out_fall);

    // Bits 15:12 - Cfg_rc_cc
    let rc_cc = (cfg_register >> 12) & 0xF; // 4-bit field
    debug_println!("Cfg_rc_cc (bits 15:12): {}", rc_cc);

    // Bits 11:8 - Cfg_resp_timeout
    let resp_timeout = (cfg_register >> 8) & 0xF; // 4-bit field
    debug_println!("Cfg_resp_timeout (bits 11:8): {}", resp_timeout);

    // Bits 7:4 - Cfg_bl_len
    let bl_len = (cfg_register >> 4) & 0xF; // 4-bit field
    debug_println!("Cfg_bl_len (bits 7:4): {}", bl_len);

    // Bit 3 - Cfg_dc_ugt
    let dc_ugt = (cfg_register >> 3) & 0x1;
    debug_println!("Cfg_dc_ugt (bit 3): {}", dc_ugt);

    // Bit 2 - Cfg_ddr
    let ddr = (cfg_register >> 2) & 0x1;
    debug_println!("Cfg_ddr (bit 2): {}", ddr);

    // Bits 1:0 - Cfg_bus_width
    let bus_width = cfg_register & 0x3; // 2-bit field
    debug_println!("Cfg_bus_width (bits 1:0): {}", bus_width);
}

fn parse_clock(clock_register: u32) {
        // Print the raw clock register value
        debug_println!("Clock register value: {:#034b} (binary), {:#X} (hex)", clock_register, clock_register);
        
        // Extracting individual fields based on the documentation:
    
        // Bit 30 - Cfg_irq_sdio_sleep_ds
        let irq_sdio_sleep_ds = (clock_register >> 30) & 0x1;
        debug_println!("Cfg_irq_sdio_sleep_ds (bit 30): {}", irq_sdio_sleep_ds);
    
        // Bit 29 - Cfg_irq_sdio_sleep
        let irq_sdio_sleep = (clock_register >> 29) & 0x1;
        debug_println!("Cfg_irq_sdio_sleep (bit 29): {}", irq_sdio_sleep);
    
        // Bit 28 - Cfg_always_on
        let always_on = (clock_register >> 28) & 0x1;
        debug_println!("Cfg_always_on (bit 28): {}", always_on);
    
        // Bits 27:22 - Cfg_rx_delay
        let rx_delay = (clock_register >> 22) & 0x3F; // 6-bit field
        debug_println!("Cfg_rx_delay (bits 27:22): {}", rx_delay);
    
        // Bits 21:16 - Cfg_tx_delay
        let tx_delay = (clock_register >> 16) & 0x3F; // 6-bit field
        debug_println!("Cfg_tx_delay (bits 21:16): {}", tx_delay);
    
        // Bit 15:14 - Cfg_sram_pd
        let sram_pd = (clock_register >> 14) & 0x3; // 2-bit field
        debug_println!("Cfg_sram_pd (bits 15:14): {}", sram_pd);
    
        // Bit 13:12 - Cfg_rx_phase
        let rx_phase = (clock_register >> 12) & 0x3; // 2-bit field
        debug_println!("Cfg_rx_phase (bits 13:12): {}", rx_phase);
    
        // Bit 11:10 - Cfg_tx_phase
        let tx_phase = (clock_register >> 10) & 0x3; // 2-bit field
        debug_println!("Cfg_tx_phase (bits 11:10): {}", tx_phase);
    
        // Bit 9:8 - Cfg_co_phase
        let co_phase = (clock_register >> 8) & 0x3; // 2-bit field
        debug_println!("Cfg_co_phase (bits 9:8): {}", co_phase);
    
        // Bit 7:6 - Cfg_src
        let cfg_src = (clock_register >> 6) & 0x3; // 2-bit field
        debug_println!("Cfg_src (bits 7:6): {}", cfg_src);
    
        // Bit 5:0 - Cfg_div
        let cfg_div = clock_register & 0x3F; // 6-bit field
        debug_println!("Cfg_div (bits 5:0): {}", cfg_div);
}