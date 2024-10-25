#![allow(dead_code)] // Allow dead code for the entire module

// Define constants for MMC data flags
pub const MMC_DATA_READ: u32 = 1;
pub const MMC_DATA_WRITE: u32 = 2;

// Define constants for MMC commands
pub const MMC_CMD_GO_IDLE_STATE: u32 = 0;
pub const MMC_CMD_SEND_OP_COND: u32 = 1;
pub const MMC_CMD_ALL_SEND_CID: u32 = 2;
pub const MMC_CMD_SET_RELATIVE_ADDR: u32 = 3;
pub const MMC_CMD_SET_DSR: u32 = 4;
pub const MMC_CMD_SWITCH: u32 = 6;
pub const MMC_CMD_SELECT_CARD: u32 = 7;
pub const MMC_CMD_SEND_EXT_CSD: u32 = 8;
pub const MMC_CMD_SEND_CSD: u32 = 9;
pub const MMC_CMD_SEND_CID: u32 = 10;
pub const MMC_CMD_STOP_TRANSMISSION: u32 = 12;
pub const MMC_CMD_SEND_STATUS: u32 = 13;
pub const MMC_CMD_SET_BLOCKLEN: u32 = 16;
pub const MMC_CMD_READ_SINGLE_BLOCK: u32 = 17;
pub const MMC_CMD_READ_MULTIPLE_BLOCK: u32 = 18;
pub const MMC_CMD_SEND_TUNING_BLOCK: u32 = 19;
pub const MMC_CMD_SEND_TUNING_BLOCK_HS200: u32 = 21;
pub const MMC_CMD_SET_BLOCK_COUNT: u32 = 23;
pub const MMC_CMD_WRITE_SINGLE_BLOCK: u32 = 24;
pub const MMC_CMD_WRITE_MULTIPLE_BLOCK: u32 = 25;
pub const MMC_CMD_ERASE_GROUP_START: u32 = 35;
pub const MMC_CMD_ERASE_GROUP_END: u32 = 36;
pub const MMC_CMD_ERASE: u32 = 38;
pub const MMC_CMD_APP_CMD: u32 = 55;
pub const MMC_CMD_SPI_READ_OCR: u32 = 58;
pub const MMC_CMD_SPI_CRC_ON_OFF: u32 = 59;
pub const MMC_CMD_RES_MAN: u32 = 62;

// Define constants for MMC command 62 arguments
pub const MMC_CMD62_ARG1: u32 = 0xefac62ec;
pub const MMC_CMD62_ARG2: u32 = 0xcbaea7;

// Define constants for SD commands
pub const SD_CMD_SEND_RELATIVE_ADDR: u32 = 3;
pub const SD_CMD_SWITCH_FUNC: u32 = 6;
pub const SD_CMD_SEND_IF_COND: u32 = 8;
pub const SD_CMD_SWITCH_UHS18V: u32 = 11;

pub const SD_CMD_APP_SET_BUS_WIDTH: u32 = 6;
pub const SD_CMD_APP_SD_STATUS: u32 = 13;
pub const SD_CMD_ERASE_WR_BLK_START: u32 = 32;
pub const SD_CMD_ERASE_WR_BLK_END: u32 = 33;
pub const SD_CMD_APP_SEND_OP_COND: u32 = 41;
pub const SD_CMD_APP_SEND_SCR: u32 = 51;

pub const INIT_CLOCK_RATE: u64 = 400000;

pub const OCR_BUSY: u32 = 0x8000_0000;
pub const OCR_HCS: u32 = 0x4000_0000;
pub const OCR_S18R: u32 = 0x0100_0000;
pub const OCR_VOLTAGE_MASK: u32 = 0x007F_FF80;
pub const OCR_ACCESS_MODE: u32 = 0x6000_0000;

// VDD voltage levels for MMC/SD card
pub const MMC_VDD_165_195: u32 = 0x0000_0080; // VDD voltage 1.65 - 1.95V
pub const MMC_VDD_20_21: u32 = 0x0000_0100; // VDD voltage 2.0 - 2.1V
pub const MMC_VDD_21_22: u32 = 0x0000_0200; // VDD voltage 2.1 - 2.2V
pub const MMC_VDD_22_23: u32 = 0x0000_0400; // VDD voltage 2.2 - 2.3V
pub const MMC_VDD_23_24: u32 = 0x0000_0800; // VDD voltage 2.3 - 2.4V
pub const MMC_VDD_24_25: u32 = 0x0000_1000; // VDD voltage 2.4 - 2.5V
pub const MMC_VDD_25_26: u32 = 0x0000_2000; // VDD voltage 2.5 - 2.6V
pub const MMC_VDD_26_27: u32 = 0x0000_4000; // VDD voltage 2.6 - 2.7V
pub const MMC_VDD_27_28: u32 = 0x0000_8000; // VDD voltage 2.7 - 2.8V
pub const MMC_VDD_28_29: u32 = 0x0001_0000; // VDD voltage 2.8 - 2.9V
pub const MMC_VDD_29_30: u32 = 0x0002_0000; // VDD voltage 2.9 - 3.0V
pub const MMC_VDD_30_31: u32 = 0x0004_0000; // VDD voltage 3.0 - 3.1V
pub const MMC_VDD_31_32: u32 = 0x0008_0000; // VDD voltage 3.1 - 3.2V
pub const MMC_VDD_32_33: u32 = 0x0010_0000; // VDD voltage 3.2 - 3.3V
pub const MMC_VDD_33_34: u32 = 0x0020_0000; // VDD voltage 3.3 - 3.4V
pub const MMC_VDD_34_35: u32 = 0x0040_0000; // VDD voltage 3.4 - 3.5V
pub const MMC_VDD_35_36: u32 = 0x0080_0000; // VDD voltage 3.5 - 3.6V
