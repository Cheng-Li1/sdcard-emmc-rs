#![no_std]  // Don't link the standard library
#![no_main] // Don't use the default entry point

use sdmmc_hal::meson_gx_mmc::SdioRegisters;

use sdmmc_protocol::sdmmc::{SdmmcCmd, SdmmcHalError, MMC_RSP_NONE, MMC_RSP_R7};
use sel4_microkit::{debug_println, protection_domain, Handler, Infallible};

const SDMMC_BASE_ADDR: *mut SdioRegisters = 0xffe05000 as *mut SdioRegisters;

#[protection_domain]
fn init() -> HandlerImpl {
    debug_println!("Driver init!");
    let clock_register: u32;
    let mut point_addr: *const u32;
    let cfg_register: u32;
    let mut cmd: SdmmcCmd;

    let cmd0 = SdmmcCmd {
        cmdidx: 0,
        resp_type: MMC_RSP_NONE,
        cmdarg: 0,
        response: [0; 4],
    };

    let mut cmd8 = SdmmcCmd {
        cmdidx: 8,
        resp_type: MMC_RSP_R7,
        cmdarg: 0x000001AA, // Voltage supply and check pattern
        response: [0; 4],
    };

    unsafe {
        let sdmmc = &mut *SDMMC_BASE_ADDR;

        sdmmc.meson_mmc_config_clock();
        
        // Read the value from the clock register
        clock_register = sdmmc.read_clock(); // Assuming this function returns u32
        point_addr = &sdmmc.clock as *const _; // Pointer to the clock register
    
        // Print the pointer address
        debug_println!("Clock register address: {:p}", point_addr);

        parse_clock(clock_register);
        parse_clock(clock_register);
        parse_clock(clock_register);
        parse_clock(clock_register);
        parse_clock(clock_register);
        parse_clock(clock_register);
        parse_clock(clock_register);
        parse_clock(clock_register);
        parse_clock(clock_register);
        parse_clock(clock_register);
        parse_clock(clock_register);
        parse_clock(clock_register);

        cfg_register = sdmmc.read_cfg();
        point_addr = &sdmmc.cfg as *const _; // Pointer to the clock register

        // Print the pointer address
        debug_println!("Cfg register address: {:p}", point_addr);

        parse_cfg(cfg_register);

        let _ = sdmmc.meson_sdmmc_send_cmd(&cmd0, None);

        debug_println!("CMD0 sent: GO_IDLE_STATE");

        let _ = sdmmc.meson_sdmmc_send_cmd(&cmd8, None);

        let mut attempts: u64 = 0;

        loop {
            attempts += 1;
            debug_println!("Polling attempt {}", attempts);

            match sdmmc.meson_sdmmc_receive_response(&mut cmd8) {
                Ok(_) => {
                    debug_println!("Response received after {} attempts", attempts);
                    // Process the CMD8 response
                    let cmd8_response = cmd8.response[0];
                    debug_println!("CMD8 Response: {:#034b} (binary), {:#X} (hex)", cmd8_response, cmd8_response);

                    // Validate the response
                    if (cmd8_response & 0xFF) != 0xAA {
                        debug_println!("CMD8 Error Response: {:#034b} (binary), {:#X} (hex)", cmd8_response, cmd8_response);
                        debug_println!("Invalid CMD8 response check pattern.");
                    }

                    // Optionally, check voltage acceptance
                    let voltage_accepted = (cmd8_response >> 8) & 0xF;
                    if voltage_accepted != 0x1 {
                        debug_println!("CMD8 Error Response: {:#034b} (binary), {:#X} (hex)", cmd8_response, cmd8_response);
                        debug_println!("Unsupported voltage range.");
                    }
                    break;
                },
                Err(SdmmcHalError::EBUSY) => {
                    debug_println!("Attempt {}: STATUS_END_OF_CHAIN not set. Card is busy.", attempts);
                },
                Err(SdmmcHalError::ETIMEDOUT) => {
                    debug_println!("Attempt {}: STATUS_RESP_TIMEOUT set.", attempts);
                },
                Err(_e) => {
                    debug_println!("Attempt {}: Received error", attempts);
                },
            }

            // Check for overall timeout
            if attempts > 5 {
                debug_println!("Polling timed out after {} attempts.", attempts);
                break;
            }
        }
    }

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