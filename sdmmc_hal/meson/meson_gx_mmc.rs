use core::ptr;

use sdmmc_protocol::sdmmc::{
    mmc_struct::{MmcBusWidth, MmcTiming, TuningState},
    sdmmc_capability::{
        MMC_CAP_4_BIT_DATA, MMC_CAP_CMD23, MMC_INTERRUPT_END_OF_CHAIN, MMC_INTERRUPT_ERROR,
        MMC_INTERRUPT_SDIO, MMC_TIMING_LEGACY, MMC_TIMING_SD_HS, MMC_TIMING_UHS,
    },
    HostInfo, MmcData, MmcDataFlag, MmcIos, MmcPowerMode, MmcSignalVoltage, SdmmcCmd,
    SdmmcHalError, SdmmcHardware,
};

// `sel4-microkit` specific implementation
#[cfg(feature = "sel4-microkit")]
#[macro_use]
mod os_layer {
    macro_rules! debug_log {
        ($($arg:tt)*) => {
            sel4_microkit::debug_println!($($arg)*);
        };
    }
}

/// Bare metal
#[cfg(not(feature = "sel4-microkit"))]
#[macro_use]
mod os_layer {
    // No operation, would be optimized out
    macro_rules! debug_log {
        ($($arg:tt)*) => {};
    }
}

const SDIO_BASE: u64 = 0xffe05000; // Base address from DTS

macro_rules! div_round_up {
    ($n:expr, $d:expr) => {
        (($n + $d - 1) / $d)
    };
}

const SD_EMMC_ADJ: u32 = 16;
const SD_EMMC_ADJUST_ADJ_DELAY_MASK: u32 = 0x3F << SD_EMMC_ADJ;
const SD_EMMC_ADJ_ENABLE: u32 = 0x2000;

// Constants translated from the C version
// Clock related constant
const SD_EMMC_CLKSRC_24M: u32 = 24000000; // 24 MHz
const SD_EMMC_CLKSRC_DIV2: u32 = 1000000000; // 1 GHz
const MESON_MIN_FREQUENCY: u32 = div_round_up!(SD_EMMC_CLKSRC_24M, CLK_MAX_DIV);
const MESON_MAX_FREQUENCY: u64 = 200000000; // 200 Mhz
const CLK_MAX_DIV: u32 = 63;
const CLK_SRC_MASK: u32 = 0b11000000;
const CLK_SRC_24M: u32 = 0 << 6;
const CLK_SRC_DIV2: u32 = 1 << 6;
const CLK_CO_PHASE_000: u32 = 0 << 8;
const CLK_CO_PHASE_090: u32 = 1 << 8;
const CLK_CO_PHASE_180: u32 = 2 << 8;
const CLK_CO_PHASE_270: u32 = 3 << 8;
const CLK_TX_PHASE_000: u32 = 0 << 10;
const CLK_TX_PHASE_180: u32 = 2 << 10;
const CLK_ALWAYS_ON: u32 = 1 << 24;

pub const CFG_BUS_WIDTH_MASK: u32 = 0b11;
pub const CFG_BUS_WIDTH_1: u32 = 0;
pub const CFG_BUS_WIDTH_4: u32 = 1;
pub const CFG_BUS_WIDTH_8: u32 = 2;
pub const CFG_BL_LEN_MASK: u32 = 0b1111 << 4;
pub const CFG_BL_LEN_SHIFT: u32 = 4;
pub const CFG_BL_LEN_512: u32 = 0b1001 << 4;
pub const CFG_RESP_TIMEOUT_MASK: u32 = 0b1111 << 8;
pub const CFG_RESP_TIMEOUT_256: u32 = 0b1000 << 8;
pub const CFG_RC_CC_MASK: u32 = 0b1111 << 12;
pub const CFG_RC_CC_16: u32 = 0b0100 << 12;
pub const CFG_RC_CC_256: u32 = 0b1000 << 12;
pub const CFG_SDCLK_ALWAYS_ON: u32 = 1 << 18;
pub const CFG_AUTO_CLK: u32 = 1 << 23;
pub const CFG_ERR_ABORT: u32 = 1 << 27;

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
const STATUS_MASK: u32 = 0xFFFF; // GENMASK(15, 0)
const STATUS_ERR_MASK: u32 = 0x1FFF; // GENMASK(12, 0)
const STATUS_RXD_ERR_MASK: u32 = 0xFF; // GENMASK(7, 0)
const STATUS_TXD_ERR: u32 = 1 << 8; // BIT(8)
const STATUS_DESC_ERR: u32 = 1 << 9; // BIT(9)
const STATUS_RESP_ERR: u32 = 1 << 10; // BIT(10)
const STATUS_RESP_TIMEOUT: u32 = 1 << 11; // BIT(11)
const STATUS_DESC_TIMEOUT: u32 = 1 << 12; // BIT(12)
const STATUS_END_OF_CHAIN: u32 = 1 << 13; // BIT(13)
const STATUS_BUSY: u32 = 1 << 31;
const STATUS_DESC_BUSY: u32 = 1 << 30;
const STATUS_DATI: u32 = 0xFF << 16; // Equivalent to GENMASK(23, 16)

// IRQ enable register masks and flags
const IRQ_RXD_ERR_MASK: u32 = 0xFF; // Equivalent to GENMASK(7, 0)
const IRQ_TXD_ERR: u32 = 1 << 8;
const IRQ_DESC_ERR: u32 = 1 << 9;
const IRQ_RESP_ERR: u32 = 1 << 10;
// Equivalent to (IRQ_RXD_ERR_MASK | IRQ_TXD_ERR | IRQ_DESC_ERR | IRQ_RESP_ERR)
const IRQ_CRC_ERR: u32 = IRQ_RXD_ERR_MASK | IRQ_TXD_ERR | IRQ_DESC_ERR | IRQ_RESP_ERR;
const IRQ_RESP_TIMEOUT: u32 = 1 << 11;
const IRQ_DESC_TIMEOUT: u32 = 1 << 12;
// Equivalent to (IRQ_RESP_TIMEOUT | IRQ_DESC_TIMEOUT)
const IRQ_TIMEOUTS: u32 = IRQ_RESP_TIMEOUT | IRQ_DESC_TIMEOUT;
const IRQ_END_OF_CHAIN: u32 = 1 << 13;
const IRQ_RESP_STATUS: u32 = 1 << 14;
const IRQ_SDIO: u32 = 1 << 15;
const IRQ_ERR_MASK: u32 = IRQ_CRC_ERR | IRQ_TIMEOUTS;

const IRQ_EN_MASK: u32 = IRQ_CRC_ERR | IRQ_TIMEOUTS | IRQ_END_OF_CHAIN;

const MESON_SDCARD_SECTOR_SIZE: u32 = 512;

pub const MAX_BLOCK_PER_TRANSFER: u32 = 0x1FF;

const WRITE_ADDR_UPPER: u32 = 0xFFFE0000;

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
struct MesonSdmmcRegisters {
    clock: u32,            // 0x00: Clock control register
    delay1: u32,           // 0x04: Delay register one
    delay2: u32,           // 0x08: Delay register two
    adjust: u32,           // 0x0C: Adjust register
    _reserved0: [u32; 12], // Padding for other unused registers (0x04 - 0x3C)
    start: u32,            // 0x40: Start register
    cfg: u32,              // 0x44: Configuration register
    status: u32,           // 0x48: Status register
    irq_en: u32,           // 0x4C: Interrupt enable register
    cmd_cfg: u32,          // 0x50: Command configuration register
    cmd_arg: u32,          // 0x54: Command argument register
    cmd_dat: u32,          // 0x58: Command data register (for DMA address)
    cmd_rsp: u32,          // 0x5C: Command response register
    cmd_rsp1: u32,         // 0x60: Command response register 1
    cmd_rsp2: u32,         // 0x64: Command response register 2
    cmd_rsp3: u32,         // 0x68: Command response register 3
    _reserved1: [u32; 9],  // Padding for other unused registers (0x6C - 0x90)
    rxd: u32,              // 0x94: Receive data register (not used)
    txd: u32,              // 0x94: Transmit data register (not used, same as RXD)
                           // Add other registers as needed
}

impl MesonSdmmcRegisters {
    /// This function is unsafe because it tries to
    unsafe fn new() -> &'static mut MesonSdmmcRegisters {
        &mut *(SDIO_BASE as *mut MesonSdmmcRegisters)
    }
}

struct DelayConfig {
    timing: MmcTiming,  // Clock rate in Hz
    current_delay: u32, // Delay value in some unit, e.g., nanoseconds
    tried_lowest_delay: u32,
    tried_highest_delay: u32,
}

pub struct SdmmcMesonHardware {
    register: &'static mut MesonSdmmcRegisters,
    delay: Option<DelayConfig>,
    timing: MmcTiming,
    frequency: u32,
    // Put other variables here
}

impl SdmmcMesonHardware {
    pub unsafe fn new() -> Self {
        let register = MesonSdmmcRegisters::new();

        // TODO: Call reset function here
        SdmmcMesonHardware {
            register,
            delay: None,
            timing: MmcTiming::CardSetup,
            frequency: MESON_MIN_FREQUENCY,
        }
    }

    /// It is quite obvious that this meson_reset function does not attempt to reset every registers as there are a lot of them
    /// So there is one assumption here that the bootloader did not leave the host register in a very strange state
    /// that may breaks the driver
    fn meson_reset(&mut self) {
        // Stop execution
        unsafe {
            ptr::write_volatile(&mut self.register.start, 0);
        }

        // Disable interrupt
        unsafe {
            ptr::write_volatile(&mut self.register.irq_en, 0);
        }

        // Acknowledge interrupt
        unsafe {
            ptr::write_volatile(&mut self.register.status, IRQ_EN_MASK | IRQ_SDIO);
        }

        // Reset delay and adjust registers
        unsafe {
            ptr::write_volatile(&mut self.register.delay1, 0);
        }
        unsafe {
            ptr::write_volatile(&mut self.register.delay2, 0);
        }
        unsafe {
            ptr::write_volatile(&mut self.register.adjust, 0);
        }

        // Set clock to a low freq
        if self.sdmmc_config_clock(MESON_MIN_FREQUENCY as u64).is_err() {
            panic!("Fatal fault in setting frequency when resetting");
        }

        // Reset config register
        let mut cfg: u32 = 0;

        // Set timeout bits
        cfg |= CFG_RESP_TIMEOUT_256 & CFG_RESP_TIMEOUT_MASK;

        // Set cmd interval
        // TODO: Both Linux and uboot use 16 cycle interval but is it the right value?
        cfg |= CFG_RC_CC_16 & CFG_RC_CC_MASK;

        // Set block len to 512
        cfg |= CFG_BL_LEN_512 & CFG_BL_LEN_MASK;

        // Set task abort bit when error is encountered
        cfg |= CFG_ERR_ABORT;

        unsafe {
            ptr::write_volatile(&mut self.register.cfg, cfg);
        }
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
        } else {
            meson_mmc_cmd |= CMD_CFG_NO_RESP;
        }

        if let Some(data) = data {
            let mut cfg: u32 = unsafe { ptr::read_volatile(&self.register.cfg) };

            cfg &= !CFG_BL_LEN_MASK;

            cfg |= ilog2(data.blocksize) << CFG_BL_LEN_SHIFT;

            // TODO: Maybe add blocksize is power of 2 check here?
            // debug_log!("Configure register value: 0x{:08x}", cfg);

            unsafe {
                ptr::write_volatile(&mut self.register.cfg, cfg);
            };

            if let MmcDataFlag::SdmmcDataWrite = data.flags {
                meson_mmc_cmd |= CMD_CFG_DATA_WR;
            }

            meson_mmc_cmd |= CMD_CFG_DATA_IO | CMD_CFG_BLOCK_MODE | data.blockcnt;
        }

        meson_mmc_cmd |= CMD_CFG_TIMEOUT_4S | CMD_CFG_OWNER | CMD_CFG_END_OF_CHAIN;

        unsafe {
            ptr::write_volatile(&mut self.register.cmd_cfg, meson_mmc_cmd);
        }
    }

    fn meson_read_response(&self, cmd: &SdmmcCmd, response: &mut [u32; 4]) {
        let [rsp0, rsp1, rsp2, rsp3] = response;

        // Assign values by reading the respective registers
        if cmd.resp_type & MMC_RSP_136 != 0 {
            unsafe {
                // Yes, this is in a reverse order as rsp0 and self.cmd_rsp3 is the least significant
                // Check uboot read response code for more details
                *rsp0 = ptr::read_volatile(&self.register.cmd_rsp3);
                *rsp1 = ptr::read_volatile(&self.register.cmd_rsp2);
                *rsp2 = ptr::read_volatile(&self.register.cmd_rsp1);
                *rsp3 = ptr::read_volatile(&self.register.cmd_rsp);
            }
            // debug_println!("Meson received 4 response back!");
        } else if cmd.resp_type & MMC_RSP_PRESENT != 0 {
            unsafe {
                *rsp0 = ptr::read_volatile(&self.register.cmd_rsp);
                // debug_println!("Meson response value: {:#034b} (binary), {:#X} (hex)", *rsp0, *rsp0);
                // debug_println!("Meson received 1 response back!");
            }
        }
    }

    /// The result that such function exists instead of using generic frequency is that different
    /// hosts may have preferred frequency for speed classes. For example, in meson, the frequency for
    /// UhsSdr104 would be 200Mhz instead of 208Mhz
    fn meson_frequency(timing: MmcTiming) -> u64 {
        match timing {
            MmcTiming::Legacy => 25000000,
            MmcTiming::MmcHs => 26000000,
            MmcTiming::SdHs => 50000000,
            MmcTiming::UhsSdr12 => 25000000,
            MmcTiming::UhsSdr25 => 50000000,
            MmcTiming::UhsSdr50 => 100000000,
            MmcTiming::UhsSdr104 => 200000000,
            MmcTiming::UhsDdr50 => 50000000,
            MmcTiming::MmcDdr52 => 52000000,
            MmcTiming::MmcHs200 => 200000000,
            MmcTiming::MmcHs400 => 200000000,
            MmcTiming::SdExp => 985000000,
            MmcTiming::SdExp12V => 985000000,
            MmcTiming::CardSetup => 400000, // Typical low frequency for card initialization
        }
    }
}

impl SdmmcHardware for SdmmcMesonHardware {
    fn sdmmc_init(&mut self) -> Result<(MmcIos, HostInfo, u128), SdmmcHalError> {
        let cap: u128 = MMC_TIMING_LEGACY
            | MMC_TIMING_SD_HS
            | MMC_TIMING_UHS
            | MMC_CAP_4_BIT_DATA
            | MMC_CAP_CMD23;

        // Reset host state
        self.meson_reset();

        let ios: MmcIos = MmcIos {
            clock: MESON_MIN_FREQUENCY as u64,
            // On odroid c4, the operating voltage is default to 3.3V
            vdd: 330,
            // TODO, figure out the correct value when we can power the card on and off
            power_delay_ms: 10,
            power_mode: MmcPowerMode::On,
            bus_width: MmcBusWidth::Width1,
            signal_voltage: MmcSignalVoltage::Voltage330,
            enabled_irq: 0,
            bus_mode: None,
            emmc: None,
            spi: None,
        };

        let info: HostInfo = HostInfo {
            max_frequency: MESON_MAX_FREQUENCY,
            min_frequency: MESON_MIN_FREQUENCY as u64,
            max_block_per_req: MAX_BLOCK_PER_TRANSFER,
            irq_capability: MMC_INTERRUPT_END_OF_CHAIN | MMC_INTERRUPT_ERROR | MMC_INTERRUPT_SDIO,
        };

        return Ok((ios, info, cap));
    }

    fn sdmmc_tune_sampling(&mut self, state: TuningState) -> Result<(), SdmmcHalError> {
        match state {
            TuningState::TuningStart => {
                if let Some(ref mut delay_config) = self.delay {
                    delay_config.tried_highest_delay = delay_config.current_delay;
                    delay_config.tried_lowest_delay = delay_config.current_delay;
                }
                return Ok(())
            }
            TuningState::TuningContinue => (),
            TuningState::TuningComplete => return Ok(()),
        }

        let mut delay_config = match self.delay.take() {
            Some(config) if config.timing == self.timing => config,
            _ => DelayConfig {
                timing: self.timing,
                current_delay: 0,
                tried_lowest_delay: 0,
                tried_highest_delay: 0,
            },
        };

        let mut adjust: u32 = SD_EMMC_ADJ_ENABLE;
        let clk: u32;

        unsafe {
            clk = ptr::read_volatile(&self.register.clock);
        }

        let clk_src: u32 = clk & CLK_SRC_MASK;

        let mux_clk_freq: u32 = match clk_src {
            CLK_SRC_24M => SD_EMMC_CLKSRC_24M,
            CLK_SRC_DIV2 => SD_EMMC_CLKSRC_DIV2,
            _ => return Err(SdmmcHalError::EUNDEFINED),
        };

        let max_div: u32 = div_round_up!(mux_clk_freq, self.frequency);

        if max_div - delay_config.tried_highest_delay >= delay_config.tried_lowest_delay {
            if max_div - delay_config.tried_highest_delay == 0 {
                return Err(SdmmcHalError::EUNSUPPORTEDCARD);
            }
            delay_config.tried_highest_delay += 1;
            delay_config.current_delay = delay_config.tried_highest_delay;
        } else {
            delay_config.tried_lowest_delay -= 1;
            delay_config.current_delay = delay_config.tried_lowest_delay;
        }

        adjust |= (delay_config.current_delay << SD_EMMC_ADJ) & SD_EMMC_ADJUST_ADJ_DELAY_MASK;

        sel4_microkit::debug_println!("Tuning sampling function: Current delay: {}, tried lowest delay: {}, tried highest delay: {}, final register value: 0x{:08x}", delay_config.current_delay, delay_config.tried_lowest_delay, delay_config.tried_highest_delay, adjust);

        self.delay = Some(delay_config);

        unsafe {
            ptr::write_volatile(&mut self.register.adjust, adjust);
        }

        Ok(())
    }

    fn sdmmc_config_clock(&mut self, freq: u64) -> Result<u64, SdmmcHalError> {
        if freq > MESON_MAX_FREQUENCY || freq < MESON_MIN_FREQUENCY as u64 {
            return Err(SdmmcHalError::EINVAL);
        }
        // #define DIV_ROUND_UP(n,d) (((n) + (d) - 1) / (d))
        let mut meson_mmc_clk: u32 = 0;

        // Valid clock freq range:
        // f_min = div_round_up!(SD_EMMC_CLKSRC_24M, CLK_MAX_DIV);
        // f_max = 100000000; /* 100 MHz */
        let clk: u32;
        let clk_src: u32;
        // 400 khz for init the card
        let clock_freq: u32 = freq as u32;
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
         * But Linux default to use CLK_CO_PHASE_180
         */
        meson_mmc_clk |= CLK_CO_PHASE_180;
        // meson_mmc_clk |= CLK_CO_PHASE_270;
        meson_mmc_clk |= CLK_TX_PHASE_000;

        meson_mmc_clk |= clk_src;
        meson_mmc_clk |= clk_div;

        unsafe {
            ptr::write_volatile(&mut self.register.clock, meson_mmc_clk);
        }

        Ok((clk / clk_div) as u64)
    }

    fn sdmmc_config_bus_width(&mut self, bus_width: MmcBusWidth) -> Result<(), SdmmcHalError> {
        let mut meson_mmc_cfg: u32;
        unsafe {
            meson_mmc_cfg = ptr::read_volatile(&self.register.cfg);
        }
        match bus_width {
            MmcBusWidth::Width1 => meson_mmc_cfg |= CFG_BUS_WIDTH_1,
            MmcBusWidth::Width4 => meson_mmc_cfg |= CFG_BUS_WIDTH_4,
            MmcBusWidth::Width8 => meson_mmc_cfg |= CFG_BUS_WIDTH_8,
        }
        unsafe {
            ptr::write_volatile(&mut self.register.cfg, meson_mmc_cfg);
        }
        Ok(())
    }

    fn sdmmc_send_command(
        &mut self,
        cmd: &SdmmcCmd,
        data: Option<&MmcData>,
    ) -> Result<(), SdmmcHalError> {
        // Set up the data addr
        let mut data_addr: u32 = 0u32;
        if let Some(mmc_data) = data {
            // TODO: Check what if the addr is u32::MAX, will the sdcard still working?
            if mmc_data.addr >= (WRITE_ADDR_UPPER as u64)
                || mmc_data.blockcnt == 0
                || mmc_data.blockcnt > MAX_BLOCK_PER_TRANSFER
            {
                debug_log!("SDMMC: INVALID INPUT VARIABLE!");
                return Err(SdmmcHalError::EINVAL);
            }
            // Depend on the flag and hardware, the cache should be flushed accordingly
            data_addr = mmc_data.addr as u32;
        }

        // Stop data transfer
        unsafe {
            ptr::write_volatile(&mut self.register.start, 0u32);
        }

        unsafe {
            ptr::write_volatile(&mut self.register.cmd_dat, data_addr);
        }

        self.meson_mmc_set_up_cmd_cfg_and_cfg(&cmd, data);

        // I am still keeping this line of code here but I think it is not necessary
        unsafe {
            ptr::write_volatile(&mut self.register.status, STATUS_MASK);
        }

        // Clear the response register, for testing & debugging
        unsafe {
            ptr::write_volatile(&mut self.register.cmd_rsp, 0u32);
        }

        unsafe {
            ptr::write_volatile(&mut self.register.cmd_arg, cmd.cmdarg);
        }

        Ok(())
    }

    fn sdmmc_receive_response(
        &self,
        cmd: &SdmmcCmd,
        response: &mut [u32; 4],
    ) -> Result<(), SdmmcHalError> {
        let status: u32;

        unsafe {
            status = ptr::read_volatile(&self.register.status);
        }

        if (status & STATUS_END_OF_CHAIN) == 0 {
            return Err(SdmmcHalError::EBUSY);
        }

        if (status & STATUS_RESP_TIMEOUT) != 0 {
            debug_log!(
                "SDMMC: CARD TIMEOUT! Host status register: 0x{:08x}",
                status
            );
            return Err(SdmmcHalError::ETIMEDOUT);
        }

        let mut return_val: Result<(), SdmmcHalError> = Ok(());

        if (status & STATUS_ERR_MASK) != 0 {
            debug_log!(
                "SDMMC: CARD IO ERROR! Host status register: 0x{:08x}",
                status
            );
            return_val = Err(SdmmcHalError::EIO);
        }

        self.meson_read_response(cmd, response);

        return_val
    }

    /// This function is meant to clear, acknowledge and then reenable the interrupt
    fn sdmmc_enable_interrupt(&mut self, irq_to_enable: &mut u32) -> Result<(), SdmmcHalError> {
        // Disable interrupt
        unsafe {
            ptr::write_volatile(&mut self.register.irq_en, 0);
        }

        // Acknowledge interrupt
        unsafe {
            ptr::write_volatile(&mut self.register.status, IRQ_EN_MASK | IRQ_SDIO);
        }

        let mut irq_bits_to_set: u32 = 0;
        if *irq_to_enable & MMC_INTERRUPT_END_OF_CHAIN > 0 {
            irq_bits_to_set |= IRQ_END_OF_CHAIN;
        }
        if *irq_to_enable & MMC_INTERRUPT_ERROR > 0 {
            irq_bits_to_set |= IRQ_ERR_MASK;
        }
        if *irq_to_enable & MMC_INTERRUPT_SDIO > 0 {
            irq_bits_to_set |= IRQ_SDIO;
        }
        unsafe {
            ptr::write_volatile(&mut self.register.irq_en, irq_bits_to_set);
        }
        return Ok(());
    }

    fn sdmmc_ack_interrupt(&mut self, irq_enabled: &u32) -> Result<(), SdmmcHalError> {
        let mut irq_bits_to_set: u32 = 0;
        if *irq_enabled & MMC_INTERRUPT_END_OF_CHAIN > 0 {
            irq_bits_to_set |= IRQ_END_OF_CHAIN;
        }
        if *irq_enabled & MMC_INTERRUPT_ERROR > 0 {
            irq_bits_to_set |= IRQ_ERR_MASK;
        }
        if *irq_enabled & MMC_INTERRUPT_SDIO > 0 {
            irq_bits_to_set |= IRQ_SDIO;
        }
        unsafe {
            ptr::write_volatile(&mut self.register.status, irq_bits_to_set);
        }
        return Ok(());
    }
}
