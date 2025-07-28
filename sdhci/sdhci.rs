//! SD Host Controller Register Block Definition
//!
//! Generated using the `tock_registers::register_structs!` macro for a
//! declarative, offset-based definition.

#![allow(dead_code)] // Allow unused fields, as a driver may not use all registers.

use core::ptr;
use sdmmc_protocol::dev_log;
use sdmmc_protocol::sdmmc::mmc_struct::{MmcBusWidth, MmcTiming};
use sdmmc_protocol::sdmmc::sdmmc_capability::{
    MMC_TIMING_LEGACY, MMC_TIMING_SD_HS, MMC_TIMING_UHS_DDR50, MMC_TIMING_UHS_SDR12,
    MMC_TIMING_UHS_SDR25, MMC_TIMING_UHS_SDR50, MMC_TIMING_UHS_SDR104, MMC_VDD_31_32,
    MMC_VDD_32_33, MMC_VDD_33_34,
};
use sdmmc_protocol::sdmmc::{
    HostInfo, MMC_RSP_136, MMC_RSP_BUSY, MMC_RSP_CRC, MMC_RSP_NONE, MMC_RSP_OPCODE,
    MMC_RSP_PRESENT, MmcData, MmcDataFlag, MmcIos, MmcPowerMode, MmcSignalVoltage, SdmmcCmd,
    SdmmcError,
};
use sdmmc_protocol::sdmmc_os::process_wait_unreliable;
use sdmmc_protocol::sdmmc_traits::SdmmcHardware;
use tock_registers::fields::FieldValue;
use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};
use tock_registers::{LocalRegisterCopy, RegisterLongName, UIntLike, register_bitfields};

use crate::sdhci_trait::SdhciHardware;

const INPUT_CLOCK_HZ: u32 = 187481262;

const INIT_DELAY: u64 = 10000;

const CLK_400_KHZ: u32 = 400000;

const CAP_VOLT_3V3_MASK: u32 = 0x01000000;
const CAP_VOLT_3V0_MASK: u32 = 0x02000000;
const CAP_VOLT_1V8_MASK: u32 = 0x04000000;

const DESC_MAX_LENGTH: u32 = 65536;
const DESC_TRAN: u16 = 2 << 4;
const DESC_VALID: u16 = 0x1;
const DESC_END: u16 = 1 << 1;

#[repr(C, packed)]
struct ADMA2Descriptor64 {
    attribute: u16,
    length: u16,
    address: u64,
}

#[repr(C, packed)]
struct ADMA2Descriptor32 {
    attribute: u16,
    length: u16,
    address: u32,
}

const SDHCI_DESC_SIZE: usize = [
    size_of::<ADMA2Descriptor32>(),
    size_of::<ADMA2Descriptor64>(),
][(size_of::<ADMA2Descriptor32>() < size_of::<ADMA2Descriptor64>()) as usize];
const SDHCI_DESC_NUMBER: usize = 32;

register_bitfields![u32,
    PRESENT_STATE [
        COMMAND_INHIBIT_CMD OFFSET(0) NUMBITS(1) [
            Off = 0,
            On = 1,
        ],
        COMMAND_INHIBIT_DAT OFFSET(1) NUMBITS(1) [
            Off = 0,
            On = 1,
        ],
        DATA_LINE_ACTIVE OFFSET(2) NUMBITS(1) [
            Off = 0,
            On = 1,
        ],
        WRITE_TRANSFER_ACTIVE OFFSET(8) NUMBITS(1) [
            Off = 0,
            On = 1,
        ],
        READ_TRANSFER_ACTIVE OFFSET(9) NUMBITS(1) [
            Off = 0,
            On = 1,
        ],
        BUFFER_WRITE_ENABLE OFFSET(10) NUMBITS(1) [
            Off = 0,
            On = 1,
        ],
        BUFFER_READ_ENABLE OFFSET(11) NUMBITS(1) [
            Off = 0,
            On = 1,
        ],
        CARD_INSERTED OFFSET(16) NUMBITS(1) [
            Off = 0,
            On = 1,
        ],
        CARD_STATE_STABLE OFFSET(17) NUMBITS(1) [
            Off = 0,
            On = 1,
        ],
        CARD_DETECT_PIN_LEVEL OFFSET(18) NUMBITS(1) [
            Off = 0,
            On = 1,
        ],
        DAT_LINE_SIGNAL_LEVEL_LO OFFSET(20) NUMBITS(4) [],
        CMD_LINE_SIGNAL_LEVEL OFFSET(24) NUMBITS(1) [],
    ],
    CAPABILITIES_LO [
        TIMEOUT_CLOCK_FREQUENCY OFFSET(0) NUMBITS(6) [
            Unknown = 0,
        ],
        TIMEOUT_CLOCK_UNIT OFFSET(7) NUMBITS(1) [
            KHz = 0,
            MHz = 1,
        ],
        BASE_CLOCK_FREQUENCY OFFSET(8) NUMBITS(8) [
            Unknown = 0,
        ],
        MAX_BLOCK_LENGTH OFFSET(16) NUMBITS(2) [
            L512 = 0,
            L1024 = 1,
            L2048 = 2,
            Reserved = 3,
        ],
        ADMA2_SUPPORT OFFSET(19) NUMBITS(1) [
            NotSupported = 0,
            Supported = 1,
        ],
        HIGH_SPEED_SUPPORT OFFSET(21) NUMBITS(1) [
            NotSupported = 0,
            Supported = 1,
        ],
        SDMA_SUPPORT OFFSET(22) NUMBITS(1) [
            NotSupported = 0,
            Supported = 1,
        ],
        SUSPEND_RESUME_SUPPORT OFFSET(23) NUMBITS(1) [
            NotSupported = 0,
            Supported = 1,
        ],
        VOLTAGE_SUPPORT_V3_3 OFFSET(24) NUMBITS(1) [
            NotSupported = 0,
            Supported = 1,
        ],
        VOLTAGE_SUPPORT_V3_0 OFFSET(25) NUMBITS(1) [
            NotSupported = 0,
            Supported = 1,
        ],
        VOLTAGE_SUPPORT_V1_8 OFFSET(26) NUMBITS(1) [
            NotSupported = 0,
            Supported = 1,
        ],
        SYS_ADDR_64_SUPPORT_V4 OFFSET(27) NUMBITS(1) [
            NotSupported = 0,
            Supported = 1,
        ],
        SYS_ADDR_64_SUPPORT_V3 OFFSET(28) NUMBITS(1) [
            NotSupported = 0,
            Supported = 1,
        ],
        ASYNC_INTERRUPT_SUPPORT OFFSET(29) NUMBITS(1) [
            NotSupported = 0,
            Supported = 1,
        ],
        SLOT_TYPE OFFSET(30) NUMBITS(2) [
            Removable = 0,
            Embedded = 1,
            SharedBus = 2,
        ],
    ],
    CAPABILITIES_HI [
        SDR50_SUPPORT OFFSET(0) NUMBITS(1) [
            NotSupported = 0,
            Supported = 1,
        ],
        SDR104_SUPPORT OFFSET(1) NUMBITS(1) [
            NotSupported = 0,
            Supported = 1,
        ],
        DDR50_SUPPORT OFFSET(2) NUMBITS(1) [
            NotSupported = 0,
            Supported = 1,
        ],
        UHSII_SUPPORT OFFSET(3) NUMBITS(1) [
            NotSupported = 0,
            Supported = 1,
        ],
        Type_A_SUPPORT OFFSET(4) NUMBITS(1) [
            NotSupported = 0,
            Supported = 1,
        ],
        Type_C_SUPPORT OFFSET(5) NUMBITS(1) [
            NotSupported = 0,
            Supported = 1,
        ],
        Type_D_SUPPORT OFFSET(6) NUMBITS(1) [
            NotSupported = 0,
            Supported = 1,
        ],
        TIMER_COUNT_FOR_RE_TUNING OFFSET(8) NUMBITS(4) [
        ],
        USE_TUNING_FOR_SDR50 OFFSET(13) NUMBITS(1) [
            NotRequired = 0,
            Required = 1,
        ],
        RE_TUNING_MODES OFFSET(14) NUMBITS(2) [
            /// Timer
            Mode1 = 0,
            /// Timer and Re-Tuning Request
            Mode2 = 1,
            /// Auto Re-Tuning (for transfer) Timer and Re-Tuning Request
            Mode3 = 2,
        ],
        CLOCK_MULTIPLIER OFFSET(16) NUMBITS(8) [
            NotSupported = 0,
        ],
        ADMA3_SUPPORT OFFSET(27) NUMBITS(1) [
            NotSupported = 0,
            Supported = 1,
        ],
    ],
];

register_bitfields![u16,
    BLOCK_SIZE [
        TRANSFER_BLOCK_SIZE OFFSET(0) NUMBITS(12) [
            Size512 = 512
        ]
    ],
    TRANSFER_MODE [
        DMA_ENABLE OFFSET(0) NUMBITS(1) [
            Off = 0,
            On = 1,
        ],
        BLOCK_COUNT_ENABLE OFFSET(1) NUMBITS(1) [
            Off = 0,
            On = 1,
        ],
        DATA_TRANSFER_DIRECTION OFFSET(4) NUMBITS(1) [
            Write = 0,
            Read = 1,
        ],
        MULTI_BLOCK_SELECT OFFSET(5) NUMBITS(1) [
            Single = 0,
            Multi = 1,
        ]
    ],
    COMMAND [
        RESPONSE_TYPE OFFSET(0) NUMBITS(2) [
            NoResponse = 0,
            Response136 = 1,
            Response48 = 2,
            Response48Busy = 3,
        ],
        CRC_CHECK OFFSET(3) NUMBITS(1) [
            Off = 0,
            On = 1,
        ],
        INDEX_CHECK OFFSET(4) NUMBITS(1) [
            Off = 0,
            On = 1,
        ],
        DATA_PRESENT OFFSET(5) NUMBITS(1) [
            Off = 0,
            On = 1,
        ],
        TYPE OFFSET(6) NUMBITS(2) [
            Normal = 0,
            Suspend = 1,
            Resume = 2,
            Abort = 3,
        ],
        INDEX OFFSET(8) NUMBITS(6) []
    ],
    CLOCK_CONTROL [
        INTERNAL_CLOCK_ENABLE OFFSET(0) NUMBITS(1) [
            Stop = 0,
            Oscillate = 1,
        ],
        INTERNAL_CLOCK_STABLE OFFSET(1) NUMBITS(1) [
            NotReady = 0,
            Ready = 1,
        ],
        SD_CLOCK_ENABLE OFFSET(2) NUMBITS(1) [
            Off = 0,
            On = 1,
        ],
        PLL_ENABLE OFFSET(3) NUMBITS(1) [
            Off = 0,
            On = 1,
        ],
        CLOCK_GENERATOR_SELECT OFFSET(5) NUMBITS(1) [
            Divided = 0,
            Programmable = 1,
        ],
        SDCLK_FREQUENCY_SELECT_HI OFFSET(6) NUMBITS(2) [],
        SDCLK_FREQUENCY_SELECT_LO OFFSET(8) NUMBITS(8) [],
    ],
    NORMAL_INTERRUPT [
        ENABLE OFFSET(0) NUMBITS(16) [
            None = 0,
            All = 0xFF,
        ],
        COMMAND_COMPLETE OFFSET(0) NUMBITS(1) [
        ],
        TRANSFER_COMPLETE OFFSET(1) NUMBITS(1) [
        ],
        BLOCK_GAP_EVENT OFFSET(2) NUMBITS(1) [
        ],
        DMA_INTERRUPT OFFSET(3) NUMBITS(1) [
        ],
        BUFFER_WRITE_READY OFFSET(4) NUMBITS(1) [
        ],
        BUFFER_READ_READY OFFSET(5) NUMBITS(1) [
        ],
        CARD_INSERTION OFFSET(6) NUMBITS(1) [
        ],
        CARD_REMOVAL OFFSET(7) NUMBITS(1) [
        ],
        CARD_INTERRUPT OFFSET(8) NUMBITS(1) [
        ],
        RE_TUNING OFFSET(12) NUMBITS(1) [
        ],
        FX OFFSET(13) NUMBITS(1) [],
        ERROR_INTERRUPT OFFSET(15) NUMBITS(1) [
        ],
    ],
    ERROR_INTERRUPT [
        ENABLE OFFSET(0) NUMBITS(16) [
            None = 0,
            All = 0xF7FF,
        ],
        COMMAND_TIMEOUT_ERROR OFFSET(0) NUMBITS(1) [],
        COMMAND_CRC_ERROR OFFSET(1) NUMBITS(1) [],
        COMMAND_END_BIT_ERROR OFFSET(2) NUMBITS(1) [],
        COMMAND_INDEX_ERROR OFFSET(3) NUMBITS(1) [],
        DATA_TIMEOUT_ERROR OFFSET(4) NUMBITS(1) [],
        DATA_CRC_ERROR OFFSET(5) NUMBITS(1) [],
        DATA_END_BIT_ERROR OFFSET(6) NUMBITS(1) [],
        CURRENT_LIMIT_ERROR OFFSET(7) NUMBITS(1) [],
        AUTO_CMD_ERROR OFFSET(8) NUMBITS(1) [],
        ADMA_ERROR OFFSET(9) NUMBITS(1) [],
        TUNING_ERROR OFFSET(10) NUMBITS(1) [],
        RESPONSE_ERROR OFFSET(11) NUMBITS(1) [],
    ],
    HOST_CONTROL_2 [
        /// requires "1.8V Signaling Enable"
        UHS_MODE_SELECT OFFSET(0) NUMBITS(3) [
            SDR12 = 0,
            SDR25 = 1,
            SDR50 = 2,
            SDR104 = 3,
            DDR50 = 4,
            // UHSII = 7,
        ],
        V1_8_SIGNALIING_ENABLE OFFSET(3) NUMBITS(1) [
            /// 3.3V signaling
            Off = 0,
            On = 1,
        ],
        DRIVER_STRENGTH_SELECT OFFSET(4) NUMBITS(2) [
            Type_B = 0,
            Type_A = 1,
            Type_C = 2,
            Type_D = 3,
        ],
        EXECUTE_TUNING OFFSET(6) NUMBITS(1) [
            Off = 0,
            On = 1,
        ],
        SAMPLING_CLOCK_SELECT OFFSET(7) NUMBITS(1) [
            FixedClock = 0,
            TunedClock = 1,
        ],
        HOST_VERSION_4_ENABLE OFFSET(12) NUMBITS(1) [
            /// version 3.00 compatible mode
            Off = 0,
            On = 1,
        ],
        ASYNC_INTERRUPT_ENABLE OFFSET(14) NUMBITS(1) [
            Off = 0,
            On = 1,
        ],
        PRESET_VALUE_ENABLE OFFSET(15) NUMBITS(1) [
            Off = 0,
            On = 1,
        ],
    ],
    PRESET_VALUE [
        SDCLK_FREQUENCY_SELECT_VALUE OFFSET(0) NUMBITS(10) [],
        CLOCK_GENERATOR_SELECT_VALUE OFFSET(10) NUMBITS(1) [
            HC_V2_COMPATIBLE = 0,
            PROGRAMMABLE = 1,
        ],
        DRIVER_STRENGTH_SELECT_VALUE OFFSET(14) NUMBITS(2) [
            Type_B = 0,
            Type_A = 1,
            Type_C = 2,
            Type_D = 3,
        ],
    ],
    HOST_CONTROLLER_VERSION [
        SPECIFICATION_VERSION OFFSET(0) NUMBITS(8) [
            V1_00 = 0,
            V2_00 = 1,
            V3_00 = 2,
            V4_00 = 3,
            V4_10 = 4,
            V4_20 = 5,
        ],
        VENDOR_VERSION_NUMBER OFFSET(8) NUMBITS(8) [],
    ],
];

register_bitfields![u8,
    HOST_CONTROL_1 [
        LED_CONTROL OFFSET(0) NUMBITS(1) [
            Off = 0,
            On = 1,
        ],
        DATA_TRANSFER_WIDTH OFFSET(1) NUMBITS(1) [
            Bits1 = 0,
            Bits4 = 1,
        ],
        HIGH_SPEED_ENABLE OFFSET(2) NUMBITS(1) [
            /// normal speed mode
            Off = 0,
            On = 1,
        ],
        DMA_SELECT OFFSET(3) NUMBITS(2) [
            SDMA = 0,
            ADMA2_32bit = 2,
            /// requires "64-bit System Address Support for V3" in the Capabilities register
            ADMA2_64bit = 3,
        ],
    ],
    TIMEOUT_CONTROL [
        /// TMCLK * (2 ^ Counter)
        DATA_TIMEOUT_COUNTER OFFSET(0) NUMBITS(4) [
            Min = 0b0000,
            Max = 0b1110,
        ]
    ],
    POWER_CONTROL [
        SD_BUS_POWER_FOR_VDD1 OFFSET(0) NUMBITS(1) [ Off = 0, On = 1 ],
        SD_BUS_VOLTAGE_SELECT_FOR_VDD1 OFFSET(1) NUMBITS(3) [ V1_8 = 0b101, V3_0 = 0b110, V3_3 = 0b111 ]
    ],
    SOFTWARE_RESET [
        ALL OFFSET(0) NUMBITS(1) [Off = 0, On = 1],
        CMD_LINE OFFSET(1) NUMBITS(1) [Off = 0, On = 1],
        DAT_LINE OFFSET(2) NUMBITS(1) [Off = 0, On = 1],
    ]
];

tock_registers::register_structs! {
    /// SD Host Controller Register Map, assumes host version 3.00 compatible mode without UHS-II
    pub SdhciRegister {
        (0x000 => sdma_system_address: ReadWrite<u32>),
        /// can be accessed only if no transaction is executing
        (0x004 => block_size: ReadWrite<u16, BLOCK_SIZE::Register>),
        (0x006 => block_count_16: ReadWrite<u16>),
        (0x008 => argument: ReadWrite<u32>),
        (0x00C => transfer_mode: ReadWrite<u16, TRANSFER_MODE::Register>),
        (0x00E => command: WriteOnly<u16, COMMAND::Register>),
        (0x010 => response: [ReadOnly<u32>; 4]),
        (0x020 => buffer_data_port: ReadWrite<u32>),
        (0x024 => present_state: ReadOnly<u32, PRESENT_STATE::Register>),
        (0x028 => host_control_1: ReadWrite<u8, HOST_CONTROL_1::Register>),
        (0x029 => power_control: ReadWrite<u8, POWER_CONTROL::Register>),
        (0x02A => block_gap_control: ReadWrite<u8>),
        (0x02B => wakeup_control: ReadWrite<u8>),
        (0x02C => clock_control: ReadWrite<u16, CLOCK_CONTROL::Register>),
        (0x02E => timeout_control: ReadWrite<u8, TIMEOUT_CONTROL::Register>),
        (0x02F => software_reset: ReadWrite<u8, SOFTWARE_RESET::Register>),
        (0x030 => normal_interrupt_status: ReadWrite<u16, NORMAL_INTERRUPT::Register>),
        (0x032 => error_interrupt_status: ReadWrite<u16, ERROR_INTERRUPT::Register>),
        (0x034 => normal_interrupt_status_enable: ReadWrite<u16, NORMAL_INTERRUPT::Register>),
        (0x036 => error_interrupt_status_enable: ReadWrite<u16, ERROR_INTERRUPT::Register>),
        (0x038 => normal_interrupt_signal_enable: ReadWrite<u16, NORMAL_INTERRUPT::Register>),
        (0x03A => error_interrupt_signal_enable: ReadWrite<u16, ERROR_INTERRUPT::Register>),
        (0x03C => auto_cmd_error_status: ReadOnly<u16>),
        (0x03E => host_control_2: ReadWrite<u16, HOST_CONTROL_2::Register>),
        (0x040 => capabilities_lo: ReadOnly<u32, CAPABILITIES_LO::Register>),
        (0x044 => capabilities_hi: ReadOnly<u32, CAPABILITIES_HI::Register>),
        (0x048 => maximum_current_capabilities_lo: ReadOnly<u32>),
        (0x04C => maximum_current_capabilities_hi: ReadOnly<u32>),
        (0x050 => force_event_for_auto_cmd_error: WriteOnly<u16>),
        (0x052 => force_event_for_error_interrupt: WriteOnly<u16, ERROR_INTERRUPT::Register>),
        (0x054 => adma_error_status: ReadOnly<u8>),
        (0x055 => _reserved0),
        (0x058 => adma_system_address_64_lo: ReadWrite<u32>),
        (0x05C => adma_system_address_64_hi: ReadWrite<u32>),
        (0x060 => preset_value_init: ReadOnly<u16, PRESET_VALUE::Register>),
        (0x062 => preset_value_default: ReadOnly<u16, PRESET_VALUE::Register>),
        (0x064 => preset_value_high_speed: ReadOnly<u16, PRESET_VALUE::Register>),
        (0x066 => preset_value_sdr12: ReadOnly<u16, PRESET_VALUE::Register>),
        (0x068 => preset_value_sdr25: ReadOnly<u16, PRESET_VALUE::Register>),
        (0x06A => preset_value_sdr50: ReadOnly<u16, PRESET_VALUE::Register>),
        (0x06C => preset_value_sdr104: ReadOnly<u16, PRESET_VALUE::Register>),
        (0x06E => preset_value_ddr50: ReadOnly<u16, PRESET_VALUE::Register>),
        (0x070 => _preset_value_reserved),
        (0x078 => adma3_id_address_64_hi: ReadWrite<u32>),
        (0x07C => adma3_id_address_64_lo: ReadWrite<u32>),
        (0x080 => _uhs2_registers),
        (0x0FC => slot_interrupt_status: ReadOnly<u16>),
        (0x0FE => host_controller_version: ReadOnly<u16, HOST_CONTROLLER_VERSION::Register>),
        (0x100 => @END),
    }
}

fn sd_get_cmd_name(cmdidx: u32) -> &'static str {
    match cmdidx {
        0 => "go_idle_state",
        1 => "reserved",
        2 => "all_send_cid",
        3 => "send_relative_addr",
        4 => "set_dst",
        5 => "reserved_for_sdio_cards",
        7 => "select/deselect_card",
        8 => "send_if_cond",
        9 => "send_csd",
        10 => "send_cid",
        11 => "voltage_switch",
        12 => "stop_transmission",
        13 => "send_status/send_task_status | sd_status",
        14 => "reserved",
        15 => "go_inactive_state",
        16 => "set_blocklen",
        17 => "read_single_block | reserved",
        18 => "read_multiple_block",
        19 => "send_tuning_block | reserved",
        20 => "speed_class_control | reserved",
        21 => " | reserved",
        22 => "address_extension | send_num_wr_blocks",
        23 => "set_block_count | set_wr_blk_erase_count",
        24 => "write_block | reserved",
        25 => "write_multiple_block",
        26 => "reserved_by_manufacturer",
        27 => "program_csd",
        28 => "set_write_prot",
        29 => "clr_write_prot",
        30 => "send_write_prot",
        31 => "reserved",
        32 => "erase_wr_blk_start",
        33 => "erase_wr_blk_end",
        38 => "erase",
        39 => " | reserved",
        40 => "defined_by_dps_spec | reserved",
        41 => "reserved | sd_send_op_cond",
        42 => "lock_unlock | set_clr_card_detect",
        51 => "reserved",
        52 => "cmd_for_sdio | send_scr",
        53 => "cmd_for_sdio",
        54 => "cmd_for_sdio",
        55 => "app_cmd",
        56 => "gen_cmd",
        60 => "reserved_by_manufacturer",
        61 => "reserved_by_manufacturer",
        62 => "reserved_by_manufacturer",
        63 => "reserved_by_manufacturer",
        _ => "unknown",
    }
}

fn sd_get_response_type(resp_type: u32) -> &'static str {
    if resp_type == MMC_RSP_PRESENT | MMC_RSP_CRC | MMC_RSP_OPCODE {
        return "present, crc, opcode";
    } else if resp_type == MMC_RSP_PRESENT | MMC_RSP_CRC | MMC_RSP_OPCODE | MMC_RSP_BUSY {
        return "present, crc, opcode, busy";
    } else if resp_type == MMC_RSP_PRESENT | MMC_RSP_136 | MMC_RSP_CRC {
        return "present, 136 bit, crc";
    } else if resp_type == MMC_RSP_PRESENT {
        return "present";
    } else if resp_type == MMC_RSP_NONE {
        return "none";
    } else {
        return "unknown";
    }
}

fn sdhci_print_present_state_string(present_state: u32) {
    dev_log!("present state: ");

    if (present_state & (1 << 0)) != 0 {
        dev_log!("CMD unusable, ");
    }

    if (present_state & (1 << 1)) != 0 {
        dev_log!("DAT unusable, ");
    }

    if (present_state & (1 << 2)) == 0 {
        dev_log!("DAT inactive, ");
    } else {
        dev_log!("DAT active, ");
    }

    if (present_state & (1 << 3)) != 0 {
        dev_log!("needs re-tuning, ");
    }

    if (present_state & (1 << 8)) != 0 {
        dev_log!("write transfer active, ");
    }

    if (present_state & (1 << 9)) != 0 {
        dev_log!("read transfer active, ");
    }

    if (present_state & (1 << 10)) != 0 {
        dev_log!("buffer write enable, ");
    }

    if (present_state & (1 << 11)) != 0 {
        dev_log!("buffer read enable, ");
    }

    if (present_state & (1 << 16)) == 0 {
        dev_log!("card reset or debouncing or no card, ");
    } else {
        dev_log!("card inserted, ");
    }

    if (present_state & (1 << 17)) == 0 {
        dev_log!("card unstable, ");
    }

    if (present_state & (1 << 18)) == 0 {
        dev_log!("no card present, ");
    } else {
        dev_log!("card present, ");
    }

    if (present_state & (1 << 19)) == 0 {
        dev_log!("write protected, ");
    } else {
        dev_log!("write enabled, ");
    }

    dev_log!(
        "DAT: {}{}{}{}, ",
        ((present_state & (1 << 23)) != 0) as i32,
        ((present_state & (1 << 22)) != 0) as i32,
        ((present_state & (1 << 21)) != 0) as i32,
        ((present_state & (1 << 20)) != 0) as i32
    );

    dev_log!("CMD: {}\n", ((present_state & (1 << 24)) != 0) as i32);
}

fn sdhci_print_adma2_descriptor_64(desc: &ADMA2Descriptor64) {
    if (desc.attribute & (1 << 0)) == 0 {
        dev_log!("invalid, ");
    } else {
        dev_log!("valid, ");
    }

    if (desc.attribute & (1 << 1)) != 0 {
        dev_log!("end of descriptor, ");
    }

    if (desc.attribute & (1 << 2)) != 0 {
        dev_log!("force to generate ADMA interrupt, ");
    }

    if (desc.attribute & (1 << 5)) == 0
        && (desc.attribute & (1 << 4)) == 0
        && (desc.attribute & (1 << 3)) == 0
    {
        dev_log!("no operationt, ");
    } else if (desc.attribute & (1 << 5)) == 0
        && (desc.attribute & (1 << 4)) == 1
        && (desc.attribute & (1 << 3)) == 0
    {
        dev_log!("reserved, ")
    } else if (desc.attribute & (1 << 5)) == 1
        && (desc.attribute & (1 << 4)) == 0
        && (desc.attribute & (1 << 3)) == 0
    {
        dev_log!("transfer data, ")
    } else if (desc.attribute & (1 << 5)) == 1
        && (desc.attribute & (1 << 4)) == 1
        && (desc.attribute & (1 << 3)) == 0
    {
        dev_log!("link descriptor, ")
    }

    let length = desc.length;
    let address = desc.address;
    dev_log!(
        "length: {}, address: {:#x}\n",
        if length == 0 { 65536 } else { length as u32 },
        address
    );
}

fn sdhci_print_capabilities(capabilities_0: u32, capabilities_1: u32) {
    if (capabilities_0 & 0b111111) == 0 {
        dev_log!("unknown timeout clock frequency, ");
    } else if (capabilities_0 & (1 << 7)) == 0 {
        dev_log!("timeout clock: {} kHz, ", capabilities_0 & (1 << 7));
    } else {
        dev_log!("timeout clock: {} MHz, ", capabilities_0 & (1 << 7));
    }

    if (capabilities_0 & 0xFF00) == 0 {
        dev_log!("unknown base clock frequency, ");
    } else {
        dev_log!("base clock: {} MHz, ", (capabilities_0 & 0xFF00) >> 8);
    }

    let block_len = (capabilities_0 & 0x30000) >> 16;
    if block_len == 3 {
        dev_log!("unknown max block length, ");
    } else {
        dev_log!("max block length: {}, ", 512 << block_len);
    }

    dev_log!("\n");

    if (capabilities_0 & (1 << 19)) == 0 {
        dev_log!("adma2 not supported, ");
    } else {
        dev_log!("adma2 supported, ");
    }

    if (capabilities_0 & (1 << 21)) == 0 {
        dev_log!("high speed not supported, ");
    } else {
        dev_log!("high speed supported, ");
    }

    if (capabilities_0 & (1 << 22)) == 0 {
        dev_log!("sdma not supported, ");
    } else {
        dev_log!("sdma supported, ");
    }

    if (capabilities_0 & (1 << 23)) == 0 {
        dev_log!("suspend/resume not supported, ");
    } else {
        dev_log!("suspend/resume supported, ");
    }

    dev_log!("\n");

    if (capabilities_0 & (1 << 24)) == 0 {
        dev_log!("3.3V not supported, ");
    } else {
        dev_log!("3.3V supported, ");
    }

    if (capabilities_0 & (1 << 25)) == 0 {
        dev_log!("3.0V not supported, ");
    } else {
        dev_log!("3.0V supported, ");
    }

    if (capabilities_0 & (1 << 26)) == 0 {
        dev_log!("1.8V not supported, ");
    } else {
        dev_log!("1.8V supported, ");
    }

    dev_log!("\n");

    if (capabilities_0 & (1 << 27)) == 0 {
        dev_log!("64-bit system address V4 not supported, ");
    } else {
        dev_log!("64-bit system address V4 supported, ");
    }

    if (capabilities_0 & (1 << 28)) == 0 {
        dev_log!("64-bit system address V3 not supported, ");
    } else {
        dev_log!("64-bit system address V3 supported, ");
    }

    dev_log!("\n");

    if (capabilities_0 & (1 << 29)) == 0 {
        dev_log!("async interrupt not supported, ");
    } else {
        dev_log!("async interrupt supported, ");
    }

    if (capabilities_0 & (1 << 30)) == 0 && (capabilities_0 & (1 << 31)) == 0 {
        dev_log!("removable card slot, ");
    } else {
        dev_log!("unknown slot type, ");
    }

    dev_log!("\n");

    if (capabilities_1 & (1 << 0)) == 0 {
        dev_log!("SDR50 not supported, ");
    } else {
        dev_log!("SDR50 supported, ");
    }

    if (capabilities_1 & (1 << 1)) == 0 {
        dev_log!("SDR104 not supported, ");
    } else {
        dev_log!("SDR104 supported, ");
    }

    if (capabilities_1 & (1 << 2)) == 0 {
        dev_log!("DDR50 not supported, ");
    } else {
        dev_log!("DDR50 supported, ");
    }

    let clock_multiplier = (capabilities_1 & 0xFF0000) >> 16;
    if clock_multiplier == 0 {
        dev_log!("clock multiplier not supported, ");
    } else {
        dev_log!("clock multiplier: {}, ", clock_multiplier + 1);
    }

    dev_log!("\n");
}

fn sdhci_print_command(command: u16) {
    let index = (command & 0x3F00) >> 8;
    dev_log!("{} ({}), ", sd_get_cmd_name(index as u32), index);

    let cmd_type = (command & 0xC0) >> 6;
    if cmd_type == 0 {
        dev_log!("normal, ");
    } else if cmd_type == 1 {
        dev_log!("suspend, ");
    } else if cmd_type == 2 {
        dev_log!("resume, ");
    } else {
        dev_log!("abort, ");
    }

    if (command & (1 << 5)) == 0 {
        dev_log!("no data present, ");
    } else {
        dev_log!("data present, ");
    }

    if (command & (1 << 4)) == 0 {
        dev_log!("no index check, ");
    } else {
        dev_log!("index check, ");
    }

    if (command & (1 << 3)) == 0 {
        dev_log!("no CRC check, ");
    } else {
        dev_log!("CRC check, ");
    }

    let response_type = command & 0x3;
    if response_type == 0 {
        dev_log!("no response");
    } else if response_type == 1 {
        dev_log!("response length 136");
    } else if response_type == 1 {
        dev_log!("response length 48");
    } else if response_type == 1 {
        dev_log!("response length 48 with Busy check");
    }

    dev_log!("\n");
}

fn sdhci_print_transfer_mode(mode: u16) {
    if (mode & (1 << 0)) == 0 {
        dev_log!("no data transfer or non DMA data transfer, ");
    } else {
        dev_log!("DMA transfer, ");
    }

    if (mode & (1 << 1)) != 0 {
        dev_log!("block count enable, ");
    }

    if (mode & (1 << 4)) == 0 {
        dev_log!("write, ");
    } else {
        dev_log!("read, ");
    }

    if (mode & (1 << 5)) == 0 {
        dev_log!("single block, ");
    } else {
        dev_log!("multiple block, ");
    }

    if (mode & (1 << 6)) == 0 {
        dev_log!("MMIO response, ");
    } else {
        dev_log!("SDIO response, ");
    }

    if (mode & (1 << 7)) != 0 {
        dev_log!("response error check, ");
    }

    if (mode & (1 << 8)) == 0 {
        dev_log!("response interrupt, ");
    }

    dev_log!("\n");
}

fn sdhci_print_host_control(host_control_1: u8) {
    dev_log!("host controller: ");
    if (host_control_1 & (1 << 5)) == 0 {
        if (host_control_1 & (1 << 1)) == 0 {
            dev_log!("1-bit, ");
        } else {
            dev_log!("4-bit, ");
        }
    } else {
        dev_log!("8-bit, ");
    }

    let dma_select = (host_control_1 & 0x18) >> 3;
    if dma_select == 0 {
        dev_log!("SDMA, ");
    } else if dma_select == 1 {
        dev_log!("reserved DMA select, ");
    } else if dma_select == 2 {
        dev_log!("32-bit ADMA2, ");
    } else {
        dev_log!("64-bit ADMA2, ");
    }

    if (host_control_1 & (1 << 2)) == 0 {
        dev_log!("normal speed, ");
    } else {
        dev_log!("high speed, ");
    }

    dev_log!("\n");
}

fn sdhci_print_normal_interrupt_status(status: u16) {
    let all_fields = &[
        (NORMAL_INTERRUPT::COMMAND_COMPLETE, "command_complete"),
        (NORMAL_INTERRUPT::TRANSFER_COMPLETE, "transfer_complete"),
        (NORMAL_INTERRUPT::BLOCK_GAP_EVENT, "block_gap_event"),
        (NORMAL_INTERRUPT::DMA_INTERRUPT, "dma_interrupt"),
        (NORMAL_INTERRUPT::BUFFER_WRITE_READY, "buffer_write_ready"),
        (NORMAL_INTERRUPT::BUFFER_READ_READY, "buffer_read_ready"),
        (NORMAL_INTERRUPT::CARD_INSERTION, "card_insertion"),
        (NORMAL_INTERRUPT::CARD_REMOVAL, "card_removal"),
        (NORMAL_INTERRUPT::CARD_INTERRUPT, "card_interrupt"),
        (NORMAL_INTERRUPT::RE_TUNING, "re-tuning"),
        (NORMAL_INTERRUPT::FX, "FX"),
        (NORMAL_INTERRUPT::ERROR_INTERRUPT, "error_interrupt"),
    ];

    dev_log!("status: ");
    if status == u16::MAX {
        return dev_log!("all\n");
    } else if status == 0 {
        return dev_log!("none\n");
    }
    for (field, name) in all_fields.iter() {
        if (status & field.val(1).value) != 0 {
            dev_log!("{}, ", name);
        }
    }
    dev_log!("\n");
}

fn sdhci_print_error_interrupt_status(status: &LocalRegisterCopy<u16, ERROR_INTERRUPT::Register>) {
    let all_fields = &[
        (
            ERROR_INTERRUPT::COMMAND_TIMEOUT_ERROR,
            "command_timeout_error",
        ),
        (ERROR_INTERRUPT::COMMAND_CRC_ERROR, "command_crc_error"),
        (
            ERROR_INTERRUPT::COMMAND_END_BIT_ERROR,
            "command_end_bit_error",
        ),
        (ERROR_INTERRUPT::COMMAND_INDEX_ERROR, "command_index_error"),
        (ERROR_INTERRUPT::DATA_TIMEOUT_ERROR, "data_timeout_error"),
        (ERROR_INTERRUPT::DATA_CRC_ERROR, "data_crc_error"),
        (ERROR_INTERRUPT::DATA_END_BIT_ERROR, "data_end_bit_error"),
        (ERROR_INTERRUPT::CURRENT_LIMIT_ERROR, "current_limit_error"),
        (ERROR_INTERRUPT::AUTO_CMD_ERROR, "auto_cmd_error"),
        (ERROR_INTERRUPT::ADMA_ERROR, "adma_error"),
        (ERROR_INTERRUPT::TUNING_ERROR, "tuning_error"),
        (ERROR_INTERRUPT::RESPONSE_ERROR, "response_error"),
    ];

    dev_log!("error interrupt status: ");
    if status.get() == u16::MAX {
        return dev_log!("all\n");
    } else if status.get() == 0 {
        return dev_log!("none\n");
    }
    for (field, name) in all_fields.iter() {
        if status.is_set(*field) {
            dev_log!("{}, ", name);
        }
    }
    dev_log!("\n");
}

impl SdhciRegister {
    unsafe fn new(sdmmc_register_base: u64) -> &'static mut SdhciRegister {
        unsafe { &mut *(sdmmc_register_base as *mut SdhciRegister) }
    }
}

pub struct SdhciHost<H: SdhciHardware> {
    register: &'static mut SdhciRegister,
    memory: *mut [u8; SDHCI_DESC_SIZE * SDHCI_DESC_NUMBER],
    cache_invalidate_function: fn(),
    physical_memory_addr: u32,
    i_tap_delay: u32,
    o_tap_delay: u32,
    sdhci_hal: H,
}

fn usleep(time: u64) {
    let ns_in_us: u64 = 1000;
    process_wait_unreliable(time * ns_in_us);
}

impl<H: SdhciHardware> SdhciHost<H> {
    pub unsafe fn new(
        sdmmc_register_base: u64,
        memory: *mut [u8; SDHCI_DESC_SIZE * SDHCI_DESC_NUMBER],
        cache_invalidate_function: fn(),
        physical_memory_addr: u32,
        sdhci_hal: H,
    ) -> Self {
        let register: &'static mut SdhciRegister =
            unsafe { SdhciRegister::new(sdmmc_register_base) };

        // TODO: Call reset function here
        SdhciHost {
            register,
            memory,
            cache_invalidate_function,
            physical_memory_addr,
            i_tap_delay: 0,
            o_tap_delay: 0,
            sdhci_hal,
        }
    }

    fn wait_for_event<T: UIntLike, R: RegisterLongName, S: Readable<T = T, R = R>>(
        &self,
        reg: &S,
        event_mask: Option<T>,
        event: T,
        mut timeout: u32,
    ) -> Result<(), SdmmcError> {
        while timeout > 0 {
            let mut value = reg.get();
            if let Some(mask) = event_mask {
                value = value & mask;
            }
            if value == event {
                return Ok(());
            }
            timeout -= 1;
            usleep(1);
        }
        Err(SdmmcError::ETIMEDOUT)
    }

    fn disable_bus_power(&self) {
        let power_control = POWER_CONTROL::SD_BUS_POWER_FOR_VDD1::Off;
        self.register.power_control.write(power_control);
        dev_log!("[set] power_control: {:#x}\n", power_control.value);
        usleep(1000);
    }

    fn reset(&self, value: FieldValue<u8, SOFTWARE_RESET::Register>) -> Result<(), SdmmcError> {
        self.register.software_reset.write(value);
        dev_log!("[set] software_reset: {:#x}\n", value.value);
        let ret = self.wait_for_event(&self.register.software_reset, Some(value.value), 0, 100000);
        dev_log!(
            "[wait] software_reset, mask: {:#x}, value {:#x}\n",
            value.value,
            0
        );
        ret
    }

    fn enable_bus_power(&self) {
        let power_control = POWER_CONTROL::SD_BUS_POWER_FOR_VDD1::On
            + POWER_CONTROL::SD_BUS_VOLTAGE_SELECT_FOR_VDD1::V3_3;
        self.register.power_control.write(power_control);
        dev_log!("[set] power_control: {:#x}\n", power_control.value);
        usleep(200)
    }

    fn reset_config(&self) -> Result<(), SdmmcError> {
        self.disable_bus_power();
        self.reset(SOFTWARE_RESET::ALL::On)?;
        self.enable_bus_power();
        Ok(())
    }

    fn init_power(&self, host_caps: LocalRegisterCopy<u32, CAPABILITIES_LO::Register>) {
        let mut power_control = POWER_CONTROL::SD_BUS_POWER_FOR_VDD1::On;
        if host_caps.is_set(CAPABILITIES_LO::VOLTAGE_SUPPORT_V3_3) {
            power_control += POWER_CONTROL::SD_BUS_VOLTAGE_SELECT_FOR_VDD1::V3_3;
        } else if host_caps.is_set(CAPABILITIES_LO::VOLTAGE_SUPPORT_V3_0) {
            power_control += POWER_CONTROL::SD_BUS_VOLTAGE_SELECT_FOR_VDD1::V3_0;
        } else if host_caps.is_set(CAPABILITIES_LO::VOLTAGE_SUPPORT_V1_8) {
            power_control += POWER_CONTROL::SD_BUS_VOLTAGE_SELECT_FOR_VDD1::V1_8;
        } else {
            panic!();
        }

        self.register.power_control.write(power_control);
        dev_log!("[set] power_control: {:#x}\n", power_control.value)
    }

    fn init_dma(&self) {
        let host_control_1 =
            HOST_CONTROL_1::DATA_TRANSFER_WIDTH::Bits1 + HOST_CONTROL_1::DMA_SELECT::ADMA2_64bit;
        self.register.host_control_1.write(host_control_1);
        dev_log!("[set] host_control_1: {:#x}\n", host_control_1.value);
    }

    fn init_interrupt(&mut self) {
        let normal_interrupt =
            NORMAL_INTERRUPT::ENABLE::All.value & !NORMAL_INTERRUPT::CARD_INSERTION::SET.value;
        self.register
            .normal_interrupt_status_enable
            .set(normal_interrupt);
        dev_log!(
            "[set] normal_interrupt_status_enable: {:#x}\n",
            normal_interrupt
        );

        let error_interrupt = ERROR_INTERRUPT::ENABLE::All;
        self.register
            .error_interrupt_status_enable
            .write(error_interrupt);
        dev_log!(
            "[set] error_interrupt_status_enable: {:#x}\n",
            error_interrupt.value
        );

        let _ = self.sdmmc_config_interrupt(false, false);
    }

    fn host_config(&mut self, host_caps: LocalRegisterCopy<u32, CAPABILITIES_LO::Register>) {
        self.init_power(host_caps);
        self.init_dma();
        self.init_interrupt();

        let block_size = BLOCK_SIZE::TRANSFER_BLOCK_SIZE::Size512;
        self.register.block_size.write(block_size);
        dev_log!("[set] block_size: {:#x}\n", block_size.value)
    }

    fn get_cap(
        &self,
        caps_lo: LocalRegisterCopy<u32, CAPABILITIES_LO::Register>,
        caps_hi: LocalRegisterCopy<u32, CAPABILITIES_HI::Register>,
    ) -> u128 {
        let mut ret: u128 = MMC_TIMING_LEGACY;
        if caps_lo.is_set(CAPABILITIES_LO::HIGH_SPEED_SUPPORT) {
            ret |= MMC_TIMING_SD_HS;
        }
        // we assume host controller version 3.0, which implies UHS-I support
        ret |= MMC_TIMING_UHS_SDR12 | MMC_TIMING_UHS_SDR25;
        if caps_hi.is_set(CAPABILITIES_HI::SDR50_SUPPORT) {
            ret |= MMC_TIMING_UHS_SDR50;
        }
        if caps_hi.is_set(CAPABILITIES_HI::DDR50_SUPPORT) {
            ret |= MMC_TIMING_UHS_DDR50;
        }
        if caps_hi.is_set(CAPABILITIES_HI::SDR104_SUPPORT) {
            ret |= MMC_TIMING_UHS_SDR104;
        }
        ret
    }

    fn cfg_initialize(&mut self) -> u128 {
        let hc_version = self
            .register
            .host_controller_version
            .read(HOST_CONTROLLER_VERSION::SPECIFICATION_VERSION);
        dev_log!("host controller version: {}\n", hc_version);
        assert!(hc_version == HOST_CONTROLLER_VERSION::SPECIFICATION_VERSION::V3_00.value);

        let host_caps_lo = self.register.capabilities_lo.extract();
        let host_caps_hi = self.register.capabilities_hi.extract();
        sdhci_print_capabilities(host_caps_lo.get(), host_caps_hi.get());

        self.reset_config().expect("reset failed");

        self.host_config(host_caps_lo);

        self.get_cap(host_caps_lo, host_caps_hi)
    }

    fn dll_rst_ctrl(&self, en_rst: u8) {
        let slcr_base_addr: u64 = 0xFF180000;
        let sd_dll_ctrl = 0x00000358;
        let sd1_dll_rst = 0x00040000;
        unsafe {
            let mut dll_ctrl = ptr::read_volatile((slcr_base_addr + sd_dll_ctrl) as *const u32);
            if en_rst == 1 {
                dll_ctrl |= sd1_dll_rst;
            } else {
                dll_ctrl &= !sd1_dll_rst;
            }
            ptr::write_volatile((slcr_base_addr + sd_dll_ctrl) as *mut u32, dll_ctrl);
        }
    }

    fn config_tap_delay(&self) {
        let slcr_base_addr: u64 = 0xFF180000;
        let sd_itapdly = 0x00000314;
        let sd_otapdly = 0x00000318;
        let i_tap_delay = self.i_tap_delay << 16;
        let o_tap_delay = self.o_tap_delay << 16;
        if i_tap_delay != 0 {
            todo!();
        } else {
            unsafe {
                let mut tap_delay = ptr::read_volatile((slcr_base_addr + sd_itapdly) as *const u32);
                let sd1_itapdly_sel_mask = 0x00FF0000;
                let sd1_itapchgwin = 0x02000000;
                let sd1_itapdlyena = 0x01000000;
                tap_delay &= !(sd1_itapdly_sel_mask | sd1_itapchgwin | sd1_itapdlyena);
                ptr::write_volatile((slcr_base_addr + sd_itapdly) as *mut u32, tap_delay);
            }
        }
        if o_tap_delay != 0 {
            todo!();
        } else {
            unsafe {
                let mut tap_delay = ptr::read_volatile((slcr_base_addr + sd_otapdly) as *const u32);
                let sd1_otapdly_sel_mask = 0x003F0000;
                tap_delay &= !sd1_otapdly_sel_mask;
                ptr::write_volatile((slcr_base_addr + sd_otapdly) as *mut u32, tap_delay);
            }
        }
    }

    fn set_tap_delay(&self) {
        self.dll_rst_ctrl(1);
        self.config_tap_delay();
        self.dll_rst_ctrl(0);
    }

    fn calc_clock(&self, clk_freq: u32) -> FieldValue<u16, CLOCK_CONTROL::Register> {
        let mut divisor: u16 = 0;

        if INPUT_CLOCK_HZ > clk_freq {
            for div_cnt in 1..1024 {
                if INPUT_CLOCK_HZ / (div_cnt * 2) <= clk_freq {
                    divisor = div_cnt as u16;
                    break;
                }
            }
        }

        dev_log!("divisor: {}\n", divisor);

        CLOCK_CONTROL::SDCLK_FREQUENCY_SELECT_HI
            .val(divisor >> (size_of::<u16>() * 8 - CLOCK_CONTROL::SDCLK_FREQUENCY_SELECT_LO.shift))
            + CLOCK_CONTROL::SDCLK_FREQUENCY_SELECT_LO.val(divisor)
    }

    fn enable_clock(
        &self,
        mut clock: FieldValue<u16, CLOCK_CONTROL::Register>,
    ) -> Result<(), SdmmcError> {
        clock += CLOCK_CONTROL::INTERNAL_CLOCK_ENABLE::Oscillate;
        self.register.clock_control.write(clock);
        dev_log!("[set] clock_control: {:#x}\n", clock.value);

        self.wait_for_event(
            &self.register.clock_control,
            Some(CLOCK_CONTROL::INTERNAL_CLOCK_STABLE::Ready.value),
            CLOCK_CONTROL::INTERNAL_CLOCK_STABLE::Ready.value,
            150000,
        )?;
        dev_log!(
            "[wait] clock_control, mask: {}, value: {}\n",
            CLOCK_CONTROL::INTERNAL_CLOCK_STABLE::Ready.value,
            CLOCK_CONTROL::INTERNAL_CLOCK_STABLE::Ready.value
        );

        clock += CLOCK_CONTROL::SD_CLOCK_ENABLE::On;
        self.register.clock_control.write(clock);
        dev_log!("[set] clock_control: {:#x}\n", clock.value);

        Ok(())
    }

    fn set_clock(&self, clk_freq: u32) -> Result<(), SdmmcError> {
        /* Disable clock */
        let clock_control = CLOCK_CONTROL::SD_CLOCK_ENABLE::Off;
        self.register.clock_control.write(clock_control);
        dev_log!("[set] clock_control: {:#x}\n", clock_control.value);

        if clk_freq == 0 {
            return Err(SdmmcError::EINVAL);
        }

        let clock = self.calc_clock(clk_freq);

        self.enable_clock(clock)
    }

    fn change_clk_freq(&self, clk_freq: u32) -> Result<(), SdmmcError> {
        self.set_tap_delay();
        self.set_clock(clk_freq)
    }

    fn card_initialize(&self) -> Result<(), SdmmcError> {
        self.change_clk_freq(CLK_400_KHZ)?;

        usleep(INIT_DELAY);

        let normal_interrupt = NORMAL_INTERRUPT::ENABLE::All;
        self.register
            .normal_interrupt_status
            .write(normal_interrupt);
        dev_log!("[set] normal_interrupt_status: ");
        sdhci_print_normal_interrupt_status(normal_interrupt.value);

        let error_interrupt = ERROR_INTERRUPT::ENABLE::All;
        self.register.error_interrupt_status.write(error_interrupt);
        dev_log!(
            "[set] error_interrupt_status: {:#x}\n",
            error_interrupt.value
        );

        self.reset(SOFTWARE_RESET::CMD_LINE::On)
    }

    fn check_bus_idle(
        &self,
        value: FieldValue<u32, PRESENT_STATE::Register>,
    ) -> Result<(), SdmmcError> {
        if self
            .register
            .present_state
            .is_set(PRESENT_STATE::CARD_INSERTED)
        {
            self.wait_for_event(&self.register.present_state, Some(value.value), 0, 10000000)?;
            dev_log!(
                "[wait] present_state: mask: {:#x}, value: {:#x}\n",
                value.value,
                0
            );
        }
        Ok(())
    }

    fn setup_cmd(&self, arg: u32, blk_cnt: u32) -> Result<(), SdmmcError> {
        self.check_bus_idle(PRESENT_STATE::COMMAND_INHIBIT_CMD::On)?;

        self.register.block_count_16.set(blk_cnt as u16);
        dev_log!("[set] block_count: {:#x}\n", blk_cnt);
        let timeout_control = TIMEOUT_CONTROL::DATA_TIMEOUT_COUNTER::Max;
        self.register.timeout_control.set(timeout_control.value);
        dev_log!("[set] timeout_control: {:#x}\n", timeout_control.value);
        self.register.argument.set(arg);
        dev_log!("[set] argument: {:#x}\n", arg);

        // acknowledge interrupt
        let normal_interrupt = NORMAL_INTERRUPT::ENABLE::All;
        self.register
            .normal_interrupt_status
            .write(normal_interrupt);
        dev_log!(
            "[set] normal_interrupt_status: {:#x}\n",
            normal_interrupt.value
        );

        let error_interrupt = ERROR_INTERRUPT::ENABLE::All;
        self.register.error_interrupt_status.write(error_interrupt);
        dev_log!(
            "[set] error_interrupt_status: {:#x}\n",
            error_interrupt.value
        );

        Ok(())
    }

    fn send_cmd(&self, cmdidx: u32, resp_type: u32, present: bool) {
        if cmdidx != 21 && cmdidx != 19 {
            let _present_state = self.register.present_state.get();
            // dev_log!("[GET] present_state: {:#x}\n", _present_state);
            // todo: fix for data inhibit check
            // if present_state & PSR_INHIBIT_DAT_MASK != 0
        }

        let mut command = COMMAND::INDEX.val(cmdidx as u16);
        if present {
            command += COMMAND::DATA_PRESENT::On;
        }
        if (resp_type & MMC_RSP_PRESENT) != 0 {
            if (resp_type & MMC_RSP_136) != 0 {
                command += COMMAND::RESPONSE_TYPE::Response136;
            } else if (resp_type & MMC_RSP_BUSY) != 0 {
                command += COMMAND::RESPONSE_TYPE::Response48Busy;
            } else {
                command += COMMAND::RESPONSE_TYPE::Response48;
            }
        }
        if (resp_type & MMC_RSP_CRC) != 0 {
            command += COMMAND::CRC_CHECK::On;
        }
        if (resp_type & MMC_RSP_OPCODE) != 0 {
            command += COMMAND::INDEX_CHECK::On;
        }

        self.register.command.write(command);
        dev_log!("[set] command: ");
        sdhci_print_command(command.value);
    }

    fn setup_read_dma(
        &mut self,
        block_count: u32,
        block_size: FieldValue<u16, BLOCK_SIZE::Register>,
        buffer_pointer: u64,
        transfer_mode: FieldValue<u16, TRANSFER_MODE::Register>,
    ) -> Result<(), SdmmcError> {
        self.register.block_size.write(block_size);
        dev_log!("[set] block_size: {:#x}\n", block_size.value);

        self.register.transfer_mode.write(transfer_mode);
        dev_log!("[set] transfer_mode: ");
        sdhci_print_transfer_mode(transfer_mode.value);

        let total_desc_lines: u32;
        if block_count * (block_size.value as u32) < DESC_MAX_LENGTH {
            total_desc_lines = 1;
        } else {
            total_desc_lines = (block_count * block_size.value as u32) / DESC_MAX_LENGTH
                + ((block_count * block_size.value as u32) % DESC_MAX_LENGTH == 0) as u32;
        }

        let ptr = self.memory.cast::<ADMA2Descriptor64>();
        {
            let slice: &mut [ADMA2Descriptor64];
            unsafe {
                slice = &mut (*ptr::slice_from_raw_parts_mut(ptr, SDHCI_DESC_NUMBER));
            }

            for i in 0..(total_desc_lines as usize) - 1 {
                slice[i].address = buffer_pointer + (i * (DESC_MAX_LENGTH as usize)) as u64;
                slice[i].attribute = DESC_TRAN | DESC_VALID;
                slice[i].length = 0;
            }

            slice[total_desc_lines as usize - 1].address = buffer_pointer;
            slice[total_desc_lines as usize - 1].attribute = DESC_TRAN | DESC_END | DESC_VALID;
            slice[total_desc_lines as usize - 1].length = block_count as u16 * block_size.value;

            for i in 0..total_desc_lines as usize {
                dev_log!("ADMA desc {}: ", i);
                sdhci_print_adma2_descriptor_64(&slice[i]);
            }
        }

        self.register
            .adma_system_address_64_lo
            .set(self.physical_memory_addr as u32);
        dev_log!(
            "[set] adma_system_address_64_lo: {:#x}\n",
            self.physical_memory_addr as u32
        );
        (self.cache_invalidate_function)();

        Ok(())
    }

    // this function roughly follows section 3.7.2.1 "Not using DMA" of the SDHC specification 4.20
    pub fn read_one_block_no_dma(
        &mut self,
        start_idx: u32,
        destination: u64,
    ) -> Result<(), SdmmcError> {
        dev_log!("\n<read_one_block_no_dma>\n");

        self.register.block_count_16.set(1);

        self.register.argument.set(start_idx);

        let transfer_mode = TRANSFER_MODE::DMA_ENABLE::Off
            + TRANSFER_MODE::BLOCK_COUNT_ENABLE::On
            + TRANSFER_MODE::DATA_TRANSFER_DIRECTION::Read;
        self.register.transfer_mode.write(transfer_mode);
        sdhci_print_transfer_mode(transfer_mode.value);

        let command = COMMAND::INDEX.val(17)
            + COMMAND::DATA_PRESENT::On
            + COMMAND::INDEX_CHECK::On
            + COMMAND::CRC_CHECK::On;
        self.register.command.write(command);
        sdhci_print_command(command.value);

        let command_complete = NORMAL_INTERRUPT::COMMAND_COMPLETE::SET;
        self.wait_for_event(
            &self.register.normal_interrupt_status,
            Some(command_complete.value),
            command_complete.value,
            1000,
        )?;

        let buffer_ready = NORMAL_INTERRUPT::BUFFER_READ_READY::SET;
        self.wait_for_event(
            &self.register.normal_interrupt_status,
            Some(buffer_ready.value),
            buffer_ready.value,
            1000,
        )?;

        for i in (0..512).step_by(4) {
            let data = self.register.buffer_data_port.get();
            // assumes no strict aliasing
            unsafe {
                *((destination + i) as *mut u32) = data;
            }
        }

        let transfer_complete = NORMAL_INTERRUPT::TRANSFER_COMPLETE::SET;
        self.wait_for_event(
            &self.register.normal_interrupt_status,
            Some(transfer_complete.value),
            transfer_complete.value,
            1000,
        )?;

        Ok(())
    }
}

/// Helper methods for registers with special handling.
impl<H: SdhciHardware> SdmmcHardware for SdhciHost<H> {
    fn sdmmc_init(&mut self) -> Result<(MmcIos, HostInfo, u128), SdmmcError> {
        dev_log!("\n<init>\n");
        let caps = self.cfg_initialize();
        self.card_initialize()?;

        let ios: MmcIos = MmcIos {
            clock: CLK_400_KHZ as u64,
            power_mode: MmcPowerMode::On,
            bus_width: MmcBusWidth::Width1,
            signal_voltage: MmcSignalVoltage::Voltage330,
            enabled_irq: false,
            emmc: None,
            spi: None,
        };

        let info: HostInfo = HostInfo {
            max_frequency: CLK_400_KHZ as u64, // ???
            min_frequency: CLK_400_KHZ as u64, // ???
            max_block_per_req: 1,              // ???
            // On odroid c4, the operating voltage is default to 3.3V
            vdd: (MMC_VDD_33_34 | MMC_VDD_32_33 | MMC_VDD_31_32), // ???
            // TODO, figure out the correct value when we can power the card on and off
            power_delay_ms: 5,
        };

        sdhci_print_present_state_string(self.register.present_state.get());
        sdhci_print_host_control(self.register.host_control_1.get());

        dev_log!("============== initialised ===============\n");

        return Ok((ios, info, caps));
    }

    fn sdmmc_config_timing(&mut self, timing: MmcTiming) -> Result<u64, SdmmcError> {
        dev_log!("\n<config_timing> {:?}\n", timing);
        Ok(CLK_400_KHZ as u64)
    }

    fn sdmmc_config_bus_width(&mut self, bus_width: MmcBusWidth) -> Result<(), SdmmcError> {
        dev_log!("\n<config_bus_width> {:?}\n", bus_width);
        Ok(())
    }

    fn sdmmc_read_datalanes(&self) -> Result<u8, SdmmcError> {
        dev_log!("\n<read_datalanes> \n");
        return Err(SdmmcError::ENOTIMPLEMENTED);
    }

    fn sdmmc_send_command(
        &mut self,
        cmd: &SdmmcCmd,
        data: Option<&MmcData>,
    ) -> Result<(), SdmmcError> {
        {
            if let Some(mmc_data) = data {
                dev_log!(
                    "\n<SEND> {} - [{} ({})], response: [{}], arg: {:#x}, block size: {}, block count: {}, addr: {:#x}\n",
                    if let sdmmc_protocol::sdmmc::MmcDataFlag::SdmmcDataRead = &mmc_data.flags {
                        "read"
                    } else {
                        "write"
                    },
                    sd_get_cmd_name(cmd.cmdidx),
                    cmd.cmdidx,
                    sd_get_response_type(cmd.resp_type),
                    cmd.cmdarg,
                    mmc_data.blocksize,
                    mmc_data.blockcnt,
                    mmc_data.addr
                );
            } else {
                dev_log!(
                    "\n<SEND> [{} ({})], response: [{}], arg: {:#x}\n",
                    sd_get_cmd_name(cmd.cmdidx),
                    cmd.cmdidx,
                    sd_get_response_type(cmd.resp_type),
                    cmd.cmdarg,
                );
            }
        }

        if let Some(mmc_data) = data {
            self.setup_cmd(cmd.cmdarg, mmc_data.blockcnt)?;
            if let MmcDataFlag::SdmmcDataRead = mmc_data.flags {
                let transfer_mode = TRANSFER_MODE::DMA_ENABLE::On
                    + TRANSFER_MODE::DATA_TRANSFER_DIRECTION::Read
                    + TRANSFER_MODE::BLOCK_COUNT_ENABLE::On;
                if mmc_data.blockcnt > 1 {
                    todo!();
                }
                self.setup_read_dma(
                    mmc_data.blockcnt,
                    BLOCK_SIZE::TRANSFER_BLOCK_SIZE.val(mmc_data.blocksize as u16),
                    mmc_data.addr,
                    transfer_mode,
                )?;
            } else {
                todo!()
            }
        } else {
            self.setup_cmd(cmd.cmdarg, 0)?;
        }

        // When cmd is send status or stop transmission, the sdcard can execute those when trasferring data
        // if cmd.cmdidx != 13 && self.register.present_state.get() {
        //     todo!("wait for data transform")
        // }

        self.send_cmd(cmd.cmdidx, cmd.resp_type, data.is_some());

        Ok(())
    }

    fn sdmmc_receive_response(
        &self,
        cmd: &SdmmcCmd,
        response: &mut [u32; 4],
    ) -> Result<(), SdmmcError> {
        let status = self.register.normal_interrupt_status.extract();

        dev_log!(
            "\n<RECV> [{} ({})], response: [{}], arg: {:#x}, ",
            sd_get_cmd_name(cmd.cmdidx),
            cmd.cmdidx,
            sd_get_response_type(cmd.resp_type),
            cmd.cmdarg
        );
        sdhci_print_normal_interrupt_status(status.get());
        sdhci_print_error_interrupt_status(&self.register.error_interrupt_status.extract());

        sdhci_print_present_state_string(self.register.present_state.get());

        if (cmd.cmdidx == 19 || cmd.cmdidx == 21)
            && status.is_set(NORMAL_INTERRUPT::BUFFER_READ_READY)
        {
            let normal_interrupt = NORMAL_INTERRUPT::BUFFER_READ_READY::SET;
            self.register
                .normal_interrupt_status
                .write(normal_interrupt);
            dev_log!("[set] normal_interrupt_status: ");
            sdhci_print_normal_interrupt_status(normal_interrupt.value);
        }

        if status.is_set(NORMAL_INTERRUPT::ERROR_INTERRUPT) {
            let error: SdmmcError;
            if self
                .register
                .error_interrupt_status
                .is_set(ERROR_INTERRUPT::COMMAND_TIMEOUT_ERROR)
            {
                error = SdmmcError::ETIMEDOUT;
            } else {
                error = SdmmcError::EUNKNOWN;
            }

            /* Write to clear error bits */
            let error_interrupt = ERROR_INTERRUPT::ENABLE::All;
            self.register.error_interrupt_status.write(error_interrupt);
            dev_log!(
                "[set] error_interrupt_status: {:#x}\n",
                error_interrupt.value
            );
            return Err(error);
        }

        let normal_interrupt: FieldValue<u16, NORMAL_INTERRUPT::Register>;

        // TODO
        if cmd.cmdidx == 17 {
            if !status.is_set(NORMAL_INTERRUPT::TRANSFER_COMPLETE) {
                return Err(SdmmcError::EBUSY);
            }
            normal_interrupt =
                NORMAL_INTERRUPT::COMMAND_COMPLETE::SET + NORMAL_INTERRUPT::TRANSFER_COMPLETE::SET;
        } else {
            if !status.is_set(NORMAL_INTERRUPT::COMMAND_COMPLETE) {
                return Err(SdmmcError::EBUSY);
            }
            normal_interrupt = NORMAL_INTERRUPT::COMMAND_COMPLETE::SET;
        }
        self.register
            .normal_interrupt_status
            .write(normal_interrupt);
        dev_log!("[set] normal_interrupt_status: ");
        sdhci_print_normal_interrupt_status(normal_interrupt.value);

        if cmd.resp_type & sdmmc_protocol::sdmmc::MMC_RSP_136 != 0 {
            // SDHCI_QUIRK2_RSP_136_HAS_CRC
            for i in 0..4 {
                response[i] = self.register.response[3 - i].get();
            }
            for i in 0..4 {
                response[i] <<= 8;
                if i != 3 {
                    response[i] |= response[i + 1] >> 24;
                }
            }
            dev_log!(
                "response: [{:#x}, {:#x}, {:#x}, {:#x}]\n",
                response[0],
                response[1],
                response[2],
                response[3]
            );
        } else {
            response[0] = self.register.response[0].get();
            dev_log!("response: [{:#x}]\n", response[0]);
        }

        Ok(())
    }

    fn sdmmc_config_interrupt(
        &mut self,
        enable_irq: bool,
        enable_sdio_irq: bool,
    ) -> Result<(), SdmmcError> {
        dev_log!(
            "\n<config_interrupt> irq: {}, sdio_irq: {}\n",
            enable_irq,
            enable_sdio_irq
        );
        if enable_irq {
            let normal_interrupt = NORMAL_INTERRUPT::ENABLE::All;
            self.register
                .normal_interrupt_signal_enable
                .write(normal_interrupt);
            dev_log!(
                "[set] normal_interrupt_signal_enable: {:#x}\n",
                normal_interrupt.value
            );

            let error_interrupt = ERROR_INTERRUPT::ENABLE::All;
            self.register
                .error_interrupt_signal_enable
                .write(error_interrupt);
            dev_log!(
                "[set] error_interrupt_signal_enable: {:#x}\n",
                error_interrupt.value
            );
        } else {
            let normal_interrupt = NORMAL_INTERRUPT::ENABLE::None;
            self.register
                .normal_interrupt_signal_enable
                .write(normal_interrupt);
            dev_log!(
                "[set] normal_interrupt_signal_enable: {:#x}\n",
                normal_interrupt.value
            );

            let error_interrupt = ERROR_INTERRUPT::ENABLE::None;
            self.register
                .error_interrupt_signal_enable
                .write(error_interrupt);
            dev_log!(
                "[set] error_interrupt_signal_enable: {:#x}\n",
                error_interrupt.value
            );
        }

        return Ok(());
    }

    // Should I remove this method and auto ack the irq in the receive response fcuntion?
    fn sdmmc_ack_interrupt(&mut self) -> Result<(), SdmmcError> {
        dev_log!("\n<ack_interrupt>\n");
        Ok(())
    }

    fn sdmmc_execute_tuning(
        &mut self,
        _memory: *mut [u8; 64],
        _sleep: &mut dyn sdmmc_protocol::sdmmc_os::Sleep,
    ) -> Result<(), SdmmcError> {
        dev_log!("\n<execute_tuning>\n");
        return Err(SdmmcError::ENOTIMPLEMENTED);
    }

    fn sdmmc_host_reset(&mut self) -> Result<sdmmc_protocol::sdmmc::MmcIos, SdmmcError> {
        dev_log!("\n<host_reset>\n");
        return Err(SdmmcError::ENOTIMPLEMENTED);
    }
}
