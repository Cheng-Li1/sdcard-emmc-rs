use core::ptr;

use sdmmc_protocol::sdmmc::{MmcData, MmcDataFlag, SdmmcCmd, SdmmcHalError};
use sel4_microkit::debug_println;

const SDIO_BASE: u64 = 0xffe05000; // Base address from DTS

// Constants translated from the C version
// Clock related constant
const SD_EMMC_CLKSRC_24M: u32 = 24000000;       // 24 MHz
const SD_EMMC_CLKSRC_DIV2: u32 = 1000000000;    // 1 GHz

const CLK_MAX_DIV: u32 = 63;
const CLK_SRC_24M: u32 = 0 << 6;
const CLK_SRC_DIV2: u32 = 1 << 6;
const CLK_CO_PHASE_000: u32 = 0 << 8;
const CLK_CO_PHASE_090: u32 = 1 << 8;
const CLK_CO_PHASE_180: u32 = 2 << 8;
const CLK_CO_PHASE_270: u32 = 3 << 8;
const CLK_TX_PHASE_000: u32 = 0 << 10;
const CLK_TX_PHASE_180: u32 = 2 << 10;
const CLK_ALWAYS_ON: u32 = 1 << 24;

macro_rules! div_round_up {
    ($n:expr, $d:expr) => {
        (($n + $d - 1) / $d)
    };
}

// CMD_CFG constants
const CMD_CFG_CMD_INDEX_SHIFT: u32 = 24;
const CMD_CFG_RESP_128: u32 = 1 << 21;
const CMD_CFG_R1B: u32 = 1 << 10;
const CMD_CFG_RESP_NOCRC: u32 = 1 << 20;
const CMD_CFG_NO_RESP: u32 = 1 << 16;
const CMD_CFG_DATA_WR: u32 = 1 << 19;
const CMD_CFG_DATA_IO: u32 = 1 << 18;
const CMD_CFG_BLOCK_MODE: u32 = 1 << 9;
const CMD_CFG_TIMEOUT_4S: u32 = 12 << 12;
const CMD_CFG_OWNER: u32 = 1 << 31;
const CMD_CFG_END_OF_CHAIN: u32 = 1 << 11;

// MMC_RSP constants
const MMC_RSP_PRESENT: u32 = 1 << 0;
const MMC_RSP_136: u32 = 1 << 1;
const MMC_RSP_CRC: u32 = 1 << 2;
const MMC_RSP_BUSY: u32 = 1 << 3;

// STATUS register masks and flags
const STATUS_MASK: u32 = 0xFFFF;               // GENMASK(15, 0)
const STATUS_ERR_MASK: u32 = 0x1FFF;           // GENMASK(12, 0)
const STATUS_RXD_ERR_MASK: u32 = 0xFF;         // GENMASK(7, 0)
const STATUS_TXD_ERR: u32 = 1 << 8;            // BIT(8)
const STATUS_DESC_ERR: u32 = 1 << 9;           // BIT(9)
const STATUS_RESP_ERR: u32 = 1 << 10;          // BIT(10)
const STATUS_RESP_TIMEOUT: u32 = 1 << 11;      // BIT(11)
const STATUS_DESC_TIMEOUT: u32 = 1 << 12;      // BIT(12)
const STATUS_END_OF_CHAIN: u32 = 1 << 13;      // BIT(13)

// Configuration constants (assuming based on context)
const CFG_BL_LEN_MASK: u32 = 0xF << 4; // Bits 4-7
const CFG_BL_LEN_SHIFT: u32 = 4;

// const VALID_DMA_ADDR_LOWER: u32 = 0x2000000;
// const VALID_DMA_ADDR_UPPER: u32 = 0x10000000;
// const VALID_DMA_ADDR_MASK: u32 = 0x80000003;

fn ilog2(x: u32) -> u32 {
    assert!(x > 0);
    31 - x.leading_zeros()
}

// Structure representing the SDIO controller's registers
/* 
 *  Those register mapping are taken from meson-gx-mmc.c in Linux source code, 
 *  meson_gx_mmc.h in uboot source code and S905X3 datasheet.
 *  Despite Odroid C4 belong to Meson GX Family, the sdmmc register mapping
 *  seems to be the same with the register mapping for meson_axg according to documentation
 *  and the register mapping defined in Linux kernel.
 *  
 *  #define MESON_SD_EMMC_CLOCK		0x00
 *  #define SD_EMMC_START           0x40
 *  #define MESON_SD_EMMC_CFG		0x44
 *  #define MESON_SD_EMMC_STATUS	0x48
 *  #define MESON_SD_EMMC_IRQ_EN	0x4c
 *  #define MESON_SD_EMMC_CMD_CFG	0x50
 *  #define MESON_SD_EMMC_CMD_ARG	0x54
 *  #define MESON_SD_EMMC_CMD_DAT	0x58
 *  #define MESON_SD_EMMC_CMD_RSP	0x5c
 *  #define MESON_SD_EMMC_CMD_RSP1	0x60
 *  #define MESON_SD_EMMC_CMD_RSP2	0x64
 *  #define MESON_SD_EMMC_CMD_RSP3	0x68
 *  #define SD_EMMC_RXD             0x94
 *  #define SD_EMMC_TXD             0x94
 *  #define SD_EMMC_LAST_REG        SD_EMMC_TXD
 */
// I think I find a bug in Linux, the odroid C4 delay register mapping are the same with meson-axg but it belongs to meson-gx
/* 
 *  There are some assumptions that I have made for this driver:
 *  1. The card is already powered on by U-Boot, so I do not need to manually manipulate 
 *  gpio pins or regulator to turn it on or off.
 *  2. The clocks are already enabled by U-Boot and there is no implicit clock shutdown when the uboot start to run
 *  my image, so I do not need to turn on the clocks that the sd card needs by myself.
 *  
 */
#[repr(C)]
pub struct SdioRegisters {
    pub clock: u32,           // 0x00: Clock control register
    _reserved0: [u32; 15],    // Padding for other unused registers (0x04 - 0x3C)
    pub start: u32,           // 0x40: Start register
    pub cfg: u32,             // 0x44: Configuration register
    pub status: u32,          // 0x48: Status register
    pub irq_en: u32,          // 0x4C: Interrupt enable register
    pub cmd_cfg: u32,         // 0x50: Command configuration register
    pub cmd_arg: u32,         // 0x54: Command argument register
    pub cmd_dat: u32,         // 0x58: Command data register (for DMA address)
    pub cmd_rsp: u32,         // 0x5C: Command response register
    pub cmd_rsp1: u32,        // 0x60: Command response register 1
    pub cmd_rsp2: u32,        // 0x64: Command response register 2
    pub cmd_rsp3: u32,        // 0x68: Command response register 3
    _reserved1: [u32; 9],     // Padding for other unused registers (0x6C - 0x90)
    pub rxd: u32,             // 0x94: Receive data register (not used)
    pub txd: u32,             // 0x94: Transmit data register (not used, same as RXD)
    // Add other registers as needed
}

impl SdioRegisters {
    // The current hardware layer assumes that the sd card is being powered on by uboot, not by the driver
    // fn powerup(&mut self) {}
    // Read a value from the clock register
    pub fn read_clock(&mut self) -> u32 {
        unsafe { ptr::read_volatile(&mut self.clock) }
    }

    fn write_clock(&mut self, value: u32) {
        unsafe { ptr::write_volatile(&mut self.clock, value); }
    }

    // Read a value from the status register
    fn read_status(&self) -> u32 {
        unsafe { ptr::read_volatile(&self.status) }
    }

    pub fn read_cfg(&self) -> u32 {
        unsafe { ptr::read_volatile(&self.cfg) }
    }

    // Configure the command register
    fn set_cmd_cfg(&mut self, value: u32) {
        unsafe {
            ptr::write_volatile(&mut self.cmd_cfg, value);
        }
    }
    // Add more methods for interacting with the SDIO registers as needed
    fn meson_mmc_request(&mut self, value: u32) {
        unsafe {
            ptr::write_volatile(&mut self.cmd_cfg, value);

        }

    }

    /// Configures the SDIO clock based on the requested clock frequency and SoC type.
    ///
    /// # Arguments
    ///
    /// * `mmc_clock` - The desired clock frequency in Hz.
    /// * `is_sm1_soc` - A boolean indicating whether the SoC is an SM1 variant.
    /// * For odorid C4, this is_sm1_soc is true
    pub fn meson_mmc_config_clock(&mut self) {
        // #define DIV_ROUND_UP(n,d) (((n) + (d) - 1) / (d))
        let mut meson_mmc_clk:u32 = 0;

        // Valid clock freq range:
        // f_min = div_round_up!(SD_EMMC_CLKSRC_24M, CLK_MAX_DIV);
	    // f_max = 100000000; /* 100 MHz */
        let clk: u32; 
        let clk_src: u32;
        // 400 khz for init the card
        let clock_freq: u32 = 400000;
        if clock_freq > 16000000 {
            clk = SD_EMMC_CLKSRC_DIV2;
            clk_src = CLK_SRC_DIV2;
        } else {
            clk = SD_EMMC_CLKSRC_24M;
            clk_src = CLK_SRC_24M;
        }

        let clk_div = div_round_up!(clk, clock_freq);
        /* 
        * From uboot meson_gx_mmc.c
        * SM1 SoCs doesn't work fine over 50MHz with CLK_CO_PHASE_180
        * If CLK_CO_PHASE_270 is used, it's more stable than other.
        * Other SoCs use CLK_CO_PHASE_180 by default.
        * It needs to find what is a proper value about each SoCs.
        * Since we are using Odroid C4, we set phase to 270
        */
        meson_mmc_clk |= CLK_CO_PHASE_270;
        meson_mmc_clk |= CLK_TX_PHASE_000;

        meson_mmc_clk |= clk_src;
        meson_mmc_clk |= clk_div;

        self.write_clock(meson_mmc_clk);
    }

    fn meson_set_ios(&mut self) {
        
    }

    // This function can be seen as a Rust version of meson_mmc_setup_cmd function in uboot
    fn meson_mmc_set_up_cmd_cfg_and_cfg(&mut self, cmd: &SdmmcCmd, data: Option<&MmcData>) {
        let mut meson_mmc_cmd: u32 = 0u32;

        meson_mmc_cmd |= (cmd.cmdidx & 0x3F) << CMD_CFG_CMD_INDEX_SHIFT;

        if cmd.resp_type & MMC_RSP_PRESENT != 0 {
            if cmd.resp_type & MMC_RSP_136 != 0 {
                meson_mmc_cmd |= CMD_CFG_RESP_128;
            }

            if cmd.resp_type & MMC_RSP_BUSY != 0 {
                meson_mmc_cmd |= CMD_CFG_R1B;
            }

            if cmd.resp_type & MMC_RSP_CRC == 0 {
                meson_mmc_cmd |= CMD_CFG_RESP_NOCRC;
            }
        }
        else {
            meson_mmc_cmd |= CMD_CFG_NO_RESP;
        }

        if let Some(data) = data {
            let mut cfg: u32 = unsafe { ptr::read_volatile(&self.cfg) };

            cfg &= !CFG_BL_LEN_MASK;

            cfg |= ilog2(data.blocksize) << CFG_BL_LEN_SHIFT;
            
            // This value should only be 9
            assert!(ilog2(data.blocksize) == 9);

            unsafe { ptr::write_volatile(&mut self.cfg, cfg); };
            
            if let MmcDataFlag::SdmmcDataWrite = data.flags {
                meson_mmc_cmd |= CMD_CFG_DATA_WR;
            }
            
            meson_mmc_cmd |= CMD_CFG_DATA_IO | CMD_CFG_BLOCK_MODE | data.blocks;
        }

        meson_mmc_cmd |= CMD_CFG_TIMEOUT_4S | CMD_CFG_OWNER | CMD_CFG_END_OF_CHAIN;

        unsafe { ptr::write_volatile(&mut self.cmd_cfg, meson_mmc_cmd); }
    }

    pub fn meson_sdmmc_send_cmd(&mut self, cmd: &SdmmcCmd, data: Option<&MmcData>) -> Result<(), SdmmcHalError> {        
        // It seems that let Some(mmc_data) = data && mmc_data.blocksize != 512
        // is not stable on this nightly 2024.05.01 compiler
        // Set up the data addr
        let mut data_addr: u32 = 0u32;
        if let Some(mmc_data) = data {
            if mmc_data.blocksize != 512 {
                return Err(SdmmcHalError::EINVAL);
            }
            // Depend on the flag and hardware, the cache should be flushed accordingly
            data_addr = mmc_data.addr;
        }

        // Stop data transfer
        unsafe { ptr::write_volatile(&mut self.start, 0u32); }

        unsafe { ptr::write_volatile(&mut self.cmd_dat, data_addr); }
        
        self.meson_mmc_set_up_cmd_cfg_and_cfg(&cmd, data);

        // Reset status register before executing the cmd
        unsafe { ptr::write_volatile(&mut self.status, STATUS_MASK); }

        // For testing
        unsafe { ptr::write_volatile(&mut self.cmd_rsp, 0u32); }

        unsafe { ptr::write_volatile(&mut self.cmd_arg, cmd.cmdarg); }

        Ok(())
    }

    fn meson_read_response(&self, cmd: &mut SdmmcCmd) {
        let [rsp0, rsp1, rsp2, rsp3] = &mut cmd.response;

        // Assign values by reading the respective registers
        if cmd.resp_type & MMC_RSP_136 != 0 {
            unsafe {
                // Yes, this is in a reverse order as rsp0 and self.cmd_rsp3 is the least significant
                // Check uboot read response code for more details
                *rsp0 = ptr::read_volatile(&self.cmd_rsp3);
                *rsp1 = ptr::read_volatile(&self.cmd_rsp2);
                *rsp2 = ptr::read_volatile(&self.cmd_rsp1);
                *rsp3 = ptr::read_volatile(&self.cmd_rsp);
            }
            debug_println!("Meson received 4 response back!");
        } else if cmd.resp_type & MMC_RSP_PRESENT != 0 {
            unsafe { 
                *rsp0 = ptr::read_volatile(&self.cmd_rsp);
                debug_println!("Meson response value: {:#034b} (binary), {:#X} (hex)", *rsp0, *rsp0);
                debug_println!("Meson received 1 response back!");
            }
        }
    }

    pub fn meson_sdmmc_receive_response(&self, cmd: &mut SdmmcCmd) -> Result<(), SdmmcHalError> {
        let status: u32;
        unsafe { status = ptr::read_volatile(&self.status); }

        debug_println!("Meson status value: {:#034b} (binary), {:#X} (hex)", status, status);

        self.meson_read_response(cmd);

        if (status & STATUS_END_OF_CHAIN) == 0 {
            return Err(SdmmcHalError::EBUSY);
        }

        if (status & STATUS_RESP_TIMEOUT) != 0 {
            return Err(SdmmcHalError::ETIMEDOUT);
        }
        
        let mut return_val: Result<(), SdmmcHalError> = Ok(());

        if (status & STATUS_ERR_MASK) != 0 {
            return_val = Err(SdmmcHalError::EIO);
        }

        self.meson_read_response(cmd);

        return_val
    }
}
