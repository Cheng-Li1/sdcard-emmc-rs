use core::ptr;

const SDIO_BASE: u64 = 0xffe03000; // Base address from DTS

// Constants translated from the C version
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
const CLK_ALWAYS_ON: u32 = 1 << 24;

macro_rules! div_round_up {
    ($n:expr, $d:expr) => {
        (($n + $d - 1) / $d)
    };
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
struct SdioRegisters {
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
    fn read_clock(&mut self) -> u32 {
        unsafe { ptr::read_volatile(&mut self.clock) }
    }

    fn write_clock(&mut self, value: u32) {
        unsafe { ptr::write_volatile(&mut self.clock, value); }
    }

    // Read a value from the status register
    fn read_status(&self) -> u32 {
        unsafe { ptr::read_volatile(&self.status) }
    }

    // Configure the command register
    fn set_cmd_cfg(&mut self, value: u32) {
        unsafe {
            ptr::write_volatile(&mut self.cmd_cfg, value);
        }
    }
    // Add more methods for interacting with the SDIO registers as needed
    fn meson_mmc_request(&mut self, value: u32) {
        
    }

    /// Configures the SDIO clock based on the requested clock frequency and SoC type.
    ///
    /// # Arguments
    ///
    /// * `mmc_clock` - The desired clock frequency in Hz.
    /// * `is_sm1_soc` - A boolean indicating whether the SoC is an SM1 variant.
    /// * For odorid C4, this is_sm1_soc is true
    fn meson_mmc_config_clock(&mut self) {
        // #define DIV_ROUND_UP(n,d) (((n) + (d) - 1) / (d))
        let mut meson_mmc_clk:u32 = 0;

        // Valid clock freq range:
        // f_min = div_round_up!(SD_EMMC_CLKSRC_24M, CLK_MAX_DIV);
	    // f_max = 100000000; /* 100 MHz */
        let clk: u32; 
        let clk_src: u32;
        let clock_freq: u32 = 4000000;
        if clock_freq > 16000000 {
            clk = SD_EMMC_CLKSRC_DIV2;
            clk_src = CLK_SRC_DIV2;
        } else {
            clk = SD_EMMC_CLKSRC_24M;
            clk_src = CLK_SRC_24M;
        }

        let clk_div = div_round_up!(clk, clk_src);
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

    fn meson_sdmmc_send_cmd(&mut self) {
        
    }
}
