use bitflags::bitflags;

// Linux protocol layer use two u32 to represent all capabilities
// In this Rust based protocol layer we use u128.
// I have thought about whether I should seperate the SdmmcHostCapability to two structs, one for sdcard, one for eMMC
// But give up this idea because I do not know if there are such host that support both sdcard and eMMC
pub(crate) struct SdmmcHostCapability(pub u128);

bitflags! {
    /// Represents the host capabilities for SD/MMC controllers
    impl SdmmcHostCapability: u128 {
        // Timing modes
        const MMC_TIMING_LEGACY       = MMC_TIMING_LEGACY;
        const MMC_TIMING_MMC_HS       = MMC_TIMING_MMC_HS;
        const MMC_TIMING_SD_HS        = MMC_TIMING_SD_HS;
        const MMC_TIMING_UHS_SDR12    = MMC_TIMING_UHS_SDR12;
        const MMC_TIMING_UHS_SDR25    = MMC_TIMING_UHS_SDR25;
        const MMC_TIMING_UHS_SDR50    = MMC_TIMING_UHS_SDR50;
        const MMC_TIMING_UHS_SDR104   = MMC_TIMING_UHS_SDR104;
        const MMC_TIMING_UHS_DDR50    = MMC_TIMING_UHS_DDR50;
        const MMC_TIMING_MMC_DDR52    = MMC_TIMING_MMC_DDR52;
        const MMC_TIMING_MMC_HS200    = MMC_TIMING_MMC_HS200;
        const MMC_TIMING_MMC_HS400    = MMC_TIMING_MMC_HS400;
        const MMC_TIMING_SD_EXP       = MMC_TIMING_SD_EXP;
        const MMC_TIMING_SD_EXP_1_2V  = MMC_TIMING_SD_EXP_1_2V;

        // Capabilities
        const MMC_CAP_4_BIT_DATA      = MMC_CAP_4_BIT_DATA;
        const MMC_CAP_8_BIT_DATA      = MMC_CAP_8_BIT_DATA;
        const MMC_CAP_BUS_WIDTH_TEST  = MMC_CAP_BUS_WIDTH_TEST;

        const MMC_CAP_VOLTAGE_TUNE    = MMC_CAP_VOLTAGE_TUNE;
        const MMC_CAP_CMD23           = MMC_CAP_CMD23;
        const MMC_CAP_AUTO_STOP       = MMC_CAP_AUTO_STOP;
    }
}

// Timing modes (starting from bit 0)
pub const MMC_TIMING_LEGACY: u128 = 1 << 0;
pub const MMC_TIMING_MMC_HS: u128 = 1 << 1;
pub const MMC_TIMING_SD_HS: u128 = 1 << 2;
pub const MMC_TIMING_UHS_SDR12: u128 = 1 << 3;
pub const MMC_TIMING_UHS_SDR25: u128 = 1 << 4;
pub const MMC_TIMING_UHS_SDR50: u128 = 1 << 5;
pub const MMC_TIMING_UHS_DDR50: u128 = 1 << 6;
pub const MMC_TIMING_UHS_SDR104: u128 = 1 << 7;
pub const MMC_TIMING_MMC_DDR52: u128 = 1 << 8;
pub const MMC_TIMING_MMC_HS200: u128 = 1 << 9;
pub const MMC_TIMING_MMC_HS400: u128 = 1 << 10;

pub const MMC_TIMING_UHS: u128 = MMC_TIMING_UHS_SDR12
    | MMC_TIMING_UHS_SDR25
    | MMC_TIMING_UHS_SDR50
    | MMC_TIMING_UHS_SDR104
    | MMC_TIMING_UHS_DDR50;

pub const MMC_TIMING_SD_EXP: u128 = 1 << 11;
pub const MMC_TIMING_SD_EXP_1_2V: u128 = 1 << 12;

// Capabilities
pub const MMC_CAP_4_BIT_DATA: u128 = 1 << 16;
pub const MMC_CAP_8_BIT_DATA: u128 = 1 << 17;

pub const MMC_CAP_BUS_WIDTH_TEST: u128 = 1 << 18;

pub const MMC_CAP_VOLTAGE_TUNE: u128 = 1 << 29;
pub const MMC_CAP_CMD23: u128 = 1 << 30;
pub const MMC_CAP_AUTO_STOP: u128 = 1 << 31;

// Interrupt related
pub const MMC_INTERRUPT_END_OF_CHAIN: u32 = 1 << 0;
pub const MMC_INTERRUPT_SUCCESSFUL_REQUEST: u32 = 1 << 1;
pub const MMC_INTERRUPT_SUCCESSFUL_READ_WRITE_REQUEST: u32 = 1 << 2;
pub const MMC_INTERRUPT_ERROR: u32 = 1 << 3;
pub const MMC_INTERRUPT_SDIO: u32 = 1 << 4;
