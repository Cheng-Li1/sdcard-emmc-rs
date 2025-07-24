//! SD Host Controller Register Block Definition
//!
//! Generated using the `tock_registers::register_structs!` macro for a
//! declarative, offset-based definition.

#![allow(dead_code)] // Allow unused fields, as a driver may not use all registers.

use core::ptr;
use sdmmc_protocol::sdmmc::mmc_struct::{MmcBusWidth, MmcTiming};
use sdmmc_protocol::sdmmc::sdmmc_capability::{MMC_VDD_31_32, MMC_VDD_32_33, MMC_VDD_33_34};
use sdmmc_protocol::sdmmc::{
    HostInfo, MMC_RSP_136, MMC_RSP_BUSY, MMC_RSP_CRC, MMC_RSP_NONE, MMC_RSP_OPCODE,
    MMC_RSP_PRESENT,
    MmcData, MmcDataFlag, MmcIos, MmcPowerMode, MmcSignalVoltage, SdmmcCmd, SdmmcError,
};
use sdmmc_protocol::sdmmc_os::process_wait_unreliable;
use sdmmc_protocol::sdmmc_traits::SdmmcHardware;
use sdmmc_protocol::{dev_log, info};
use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};
use tock_registers::{RegisterLongName, UIntLike, register_bitfields};

const IRQ_ENABLE_MASK: u16 = 0xFFFF;
const ERR_ENABLE_MASK: u16 = 0xFFFF;

const HC_SPEC_VER_MASK: u16 = 0xFF;

const PC_BUS_VSEL_1V8_MASK: u8 = 0x0000000A;
const PC_BUS_VSEL_3V0_MASK: u8 = 0x0000000C;
const PC_BUS_VSEL_3V3_MASK: u8 = 0x0000000E;
const PC_BUS_PWR_MASK: u8 = 0x00000001;
const PC_EMMC_HW_RST_MASK: u8 = 0x00000010;

const INPUT_CLOCK_HZ: u32 = 187481262;

const CC_EXT_MAX_DIV_CNT: u16 = 2046;
const CC_SDCLK_FREQ_SEL_MASK: u32 = 0x000000FF;
const CC_DIV_SHIFT: u32 = 8;
const CC_SDCLK_FREQ_SEL_EXT_MAS: u32 = 0x00000003;
const CC_EXT_DIV_SHIFT: u32 = 6;
const CC_INT_CLK_EN_MASK: u16 = 0x00000001;
const CC_SD_CLK_EN_MASK: u16 = 0x00000004;
const CC_INT_CLK_STABLE_MASK: u16 = 0x00000002;

const INIT_DELAY: u64 = 10000;

const CLK_400_KHZ: u32 = 400000;

const PSR_CARD_INSRT_MASK: u32 = 0x00010000;
const PSR_INIHIBIT_CMD_MASK: u32 = 0x00000001;
const PSR_INHIBIT_DAT_MASK: u32 = 0x00000002;

const NORM_INTR_ALL_MASK: u16 = 0x0000FFFF;
const ERROR_INTR_ALL_MASK: u16 = 0x0000F3FF;

const SWRST_ALL_MASK: u8 = 0x00000001;
const SWRST_CMD_LINE_MASK: u8 = 0x00000002;

const CAP_VOLT_3V3_MASK: u32 = 0x01000000;
const CAP_VOLT_3V0_MASK: u32 = 0x02000000;
const CAP_VOLT_1V8_MASK: u32 = 0x04000000;

const HC_DMA_ADMA2_64_MASK: u8 = 0x00000018;

const INTR_CARD_MASK: u16 = 0x00000100;

const BLK_SIZE_512_MASK: u16 = 0x200;

const TM_DMA_EN_MASK: u16 = 0x00000001;
const TM_BLK_CNT_EN_MASK: u16 = 0x00000002;
const TM_DAT_DIR_SEL_MASK: u16 = 0x00000010;

const INTR_CC_MASK: u16 = 0x00000001;
const INTR_TC_MASK: u16 = 0x00000002;
const INTR_BRR_MASK: u16 = 0x00000020;
const INTR_ERR_MASK: u16 = 0x00008000;

const INTR_ERR_CT_MASK: u16 = 0x00000001;

const BLK_SIZE_MASK: u16 = 0x00000FFF;

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

//--------------------------------------------------------------------------------------------------
// Bitfield Definitions (These are unchanged from the previous method)
//--------------------------------------------------------------------------------------------------

register_bitfields![u32,
    PRESENT_STATE [
        COMMAND_INHIBIT_CMD OFFSET(0) NUMBITS(1) [],
        COMMAND_INHIBIT_DAT OFFSET(1) NUMBITS(1) [],
        DATA_LINE_ACTIVE OFFSET(2) NUMBITS(1) [],
        WRITE_TRANSFER_ACTIVE OFFSET(8) NUMBITS(1) [],
        READ_TRANSFER_ACTIVE OFFSET(9) NUMBITS(1) [],
        BUFFER_WRITE_ENABLE OFFSET(10) NUMBITS(1) [],
        BUFFER_READ_ENABLE OFFSET(11) NUMBITS(1) [],
        CARD_INSERTED OFFSET(16) NUMBITS(1) [],
        CARD_STATE_STABLE OFFSET(17) NUMBITS(1) [],
        CARD_DETECT_PIN_LEVEL OFFSET(18) NUMBITS(1) [],
        WRITE_PROTECT_SWITCH_PIN_LEVEL OFFSET(19) NUMBITS(1) []
    ]
];
register_bitfields![u16,
    NORMAL_INTERRUPT [
        COMMAND_COMPLETE OFFSET(0) NUMBITS(1) [],
        TRANSFER_COMPLETE OFFSET(1) NUMBITS(1) [],
        BLOCK_GAP_EVENT OFFSET(2) NUMBITS(1) [],
        DMA_INTERRUPT OFFSET(3) NUMBITS(1) [],
        BUFFER_WRITE_READY OFFSET(4) NUMBITS(1) [],
        BUFFER_READ_READY OFFSET(5) NUMBITS(1) [],
        CARD_INSERTION OFFSET(6) NUMBITS(1) [],
        CARD_REMOVAL OFFSET(7) NUMBITS(1) [],
        CARD_INTERRUPT OFFSET(8) NUMBITS(1) [],
        INT_A OFFSET(9) NUMBITS(1) [],
        INT_B OFFSET(10) NUMBITS(1) [],
        INT_C OFFSET(11) NUMBITS(1) [],
        RE_TUNING OFFSET(12) NUMBITS(1) [],
        FX OFFSET(13) NUMBITS(1) [],
        RESEREVED OFFSET(14) NUMBITS(1) [],
        ERROR_INTERRUPT OFFSET(15) NUMBITS(1) []
    ],
    ERROR_INTERRUPT [
        COMMAND_TIMEOUT_ERROR OFFSET(0) NUMBITS(1) [],
        COMMAND_CRC_ERROR OFFSET(1) NUMBITS(1) [],
        COMMAND_END_BIT_ERROR OFFSET(2) NUMBITS(1) [],
        COMMAND_INDEX_ERROR OFFSET(3) NUMBITS(1) [],
        DATA_TIMEOUT_ERROR OFFSET(4) NUMBITS(1) [],
        DATA_CRC_ERROR OFFSET(5) NUMBITS(1) [],
        DATA_END_BIT_ERROR OFFSET(6) NUMBITS(1) [],
        CURRENT_LIMIT_ERROR OFFSET(7) NUMBITS(1) [],
        AUTO_CMD_ERROR OFFSET(8) NUMBITS(1) [],
        ADMA_ERROR OFFSET(9) NUMBITS(1) []
    ]
];

register_bitfields![u8,
    POWER_CONTROL [
        SD_BUS_POWER OFFSET(0) NUMBITS(1) [ Off = 0, On = 1 ],
        SD_BUS_VOLTAGE_SELECT OFFSET(1) NUMBITS(3) [ V3_3 = 0b111, V3_0 = 0b110, V1_8 = 0b101 ]
    ],
    SOFTWARE_RESET [
        RESET_ALL OFFSET(0) NUMBITS(1) [],
        RESET_CMD_LINE OFFSET(1) NUMBITS(1) [],
        RESET_DAT_LINE OFFSET(2) NUMBITS(1) []
    ]
];

//--------------------------------------------------------------------------------------------------
// Register Block Definition using register_structs!
//--------------------------------------------------------------------------------------------------

tock_registers::register_structs! {
    /// SD Host Controller Register Map
    pub SdhciRegister {
        (0x000 => sdma_system_address: ReadWrite<u32>),
        (0x004 => block_size: ReadWrite<u16>),
        (0x006 => block_count: ReadWrite<u16>),
        (0x008 => argument: ReadWrite<u32>),
        (0x00C => transfer_mode: ReadWrite<u16>),
        (0x00E => command: WriteOnly<u16>),
        (0x010 => response: [ReadOnly<u32>; 4]),
        (0x020 => buffer_data_port: ReadWrite<u32>),
        (0x024 => present_state: ReadOnly<u32, PRESENT_STATE::Register>),
        (0x028 => host_control_1: ReadWrite<u8>),
        (0x029 => power_control: ReadWrite<u8, POWER_CONTROL::Register>),
        (0x02A => block_gap_control: ReadWrite<u8>),
        (0x02B => wakeup_control: ReadWrite<u8>),
        (0x02C => clock_control: ReadWrite<u16>),
        (0x02E => timeout_control: ReadWrite<u8>),
        (0x02F => software_reset: ReadWrite<u8, SOFTWARE_RESET::Register>),
        (0x030 => normal_interrupt_status: ReadWrite<u16, NORMAL_INTERRUPT::Register>),
        (0x032 => error_interrupt_status: ReadWrite<u16, ERROR_INTERRUPT::Register>),
        (0x034 => normal_interrupt_status_enable: ReadWrite<u16, NORMAL_INTERRUPT::Register>),
        (0x036 => error_interrupt_status_enable: ReadWrite<u16, ERROR_INTERRUPT::Register>),
        (0x038 => normal_interrupt_signal_enable: ReadWrite<u16, NORMAL_INTERRUPT::Register>),
        (0x03A => error_interrupt_signal_enable: ReadWrite<u16, ERROR_INTERRUPT::Register>),
        (0x03C => auto_cmd_error_status: ReadOnly<u16>),
        (0x03E => host_control_2: ReadWrite<u16>),
        (0x040 => capabilities_0: ReadOnly<u32>),
        (0x044 => capabilities_1: ReadOnly<u32>),
        (0x048 => maximum_current_capabilities: ReadOnly<u32>),
        (0x04C => _reserved0),
        (0x050 => force_event_for_auto_cmd_error: WriteOnly<u16>),
        (0x052 => force_event_for_error_interrupt: WriteOnly<u16>),
        (0x054 => adma_error_status: ReadOnly<u8>),
        (0x055 => _reserved1),
        (0x058 => adma_system_address_64_lo: ReadWrite<u32>),
        (0x05C => adma_system_address_64_hi: ReadWrite<u32>),
        (0x060 => preset_value: [ReadOnly<u16>; 8]),
        (0x070 => _reserved2),
        (0x078 => adma3_id_address_64_hi: ReadWrite<u32>),
        (0x07C => adma3_id_address_64_lo: ReadWrite<u32>),
        (0x080 => uhs2_block_size: ReadOnly<u16>),
        (0x082 => uhs2_block_count: ReadOnly<u16>),
        (0x084 => _reserved3),
        (0x088 => uhs2_command_packet: [WriteOnly<u8>; 20]),
        (0x09C => uhs2_transfer_mode: WriteOnly<u16>),
        (0x09E => _reserved4),
        (0x0A0 => uhs2_response_packet: [ReadOnly<u8>; 20]),
        (0x0B4 => _reserved5),
        // Note: UHS-II MSG is non-contiguous, so we define its parts separately
        // and provide a helper method to combine them.
        (0x0B6 => uhs2_msg_hi: ReadOnly<u16>),
        (0x0B8 => _reserved6),
        (0x0BA => uhs2_msg_lo: ReadOnly<u16>),
        (0x0BC => uhs2_device_select: ReadWrite<u16>),
        (0x0BE => uhs2_dev_int_code: ReadOnly<u16>),
        (0x0C0 => uhs2_timer_control: ReadWrite<u16>),
        (0x0C2 => uhs2_software_reset: WriteOnly<u16>),
        (0x0C4 => uhs2_error_interrupt_status: ReadWrite<u32>),
        (0x0C8 => uhs2_error_interrupt_status_enable: ReadWrite<u32>),
        (0x0CC => uhs2_error_interrupt_signal_enable: ReadWrite<u32>),
        (0x0D0 => _reserved7),
        (0x0E0 => pointer_for_uhs2_host_capabilities: ReadOnly<u32>),
        (0x0E4 => pointer_for_uhs2_test: ReadWrite<u32>),
        (0x0E8 => pointer_for_vendor_specific_area: ReadWrite<u32>),
        (0x0EC => pointer_for_embedded_control: ReadWrite<u32>),
        (0x0F0 => _reserved8),
        (0x0FC => slot_interrupt_status: ReadOnly<u16>),
        (0x0FE => host_controller_version: ReadOnly<u16>),
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

    dev_log!("DAT: {}{}{}{}, ", ((present_state & (1 << 23)) != 0) as i32, ((present_state & (1 << 22)) != 0) as i32, ((present_state & (1 << 21)) != 0) as i32, ((present_state & (1 << 20)) != 0) as i32);

    dev_log!("CMD: {}, ", ((present_state & (1 << 24)) != 0) as i32);

    if (present_state & (1 << 25)) == 0 {
        dev_log!("host regulator voltage not stable, ");
    } else {
        dev_log!("host regulator voltage stable, ");
    }

    if (present_state & (1 << 27)) != 0 {
        dev_log!("command cannot be issued, ");
    }

    dev_log!("\n")
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

    if (desc.attribute & (1 << 5)) == 0 && (desc.attribute & (1 << 4)) == 0 && (desc.attribute & (1 << 3)) == 0 {
        dev_log!("no operationt, ");
    } else if (desc.attribute & (1 << 5)) == 0 && (desc.attribute & (1 << 4)) == 1 && (desc.attribute & (1 << 3)) == 0 {
        dev_log!("reserved, ")
    } else if (desc.attribute & (1 << 5)) == 1 && (desc.attribute & (1 << 4)) == 0 && (desc.attribute & (1 << 3)) == 0 {
        dev_log!("transfer data, ")
    } else if (desc.attribute & (1 << 5)) == 1 && (desc.attribute & (1 << 4)) == 1 && (desc.attribute & (1 << 3)) == 0 {
        dev_log!("link descriptor, ")
    }

    let length = desc.length;
    let address = desc.address;
    dev_log!("length: {}, address: {:#x}\n", if length == 0 {65536} else {length as u32}, address);
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

impl SdhciRegister {
    unsafe fn new(sdmmc_register_base: u64) -> &'static mut SdhciRegister {
        unsafe { &mut *(sdmmc_register_base as *mut SdhciRegister) }
    }
}

pub struct SdhciHost {
    register: &'static mut SdhciRegister,
    transfer_mode: u16,
    memory: *mut [u8; SDHCI_DESC_SIZE * SDHCI_DESC_NUMBER],
    cache_invalidate_function: fn(),
    physical_memory_addr: u32,
    i_tap_delay: u32,
    o_tap_delay: u32,
}

fn usleep(time: u64) {
    let ns_in_us: u64 = 1000;
    process_wait_unreliable(time * ns_in_us);
}

impl SdhciHost {
    pub unsafe fn new(
        sdmmc_register_base: u64,
        memory: *mut [u8; SDHCI_DESC_SIZE * SDHCI_DESC_NUMBER],
        cache_invalidate_function: fn(),
        physical_memory_addr: u32,
    ) -> Self {
        let register: &'static mut SdhciRegister =
            unsafe { SdhciRegister::new(sdmmc_register_base) };

        // TODO: Call reset function here
        SdhciHost {
            register,
            transfer_mode: 0,
            memory,
            cache_invalidate_function,
            physical_memory_addr,
            i_tap_delay: 0,
            o_tap_delay: 0,
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
        self.register.power_control.set(PC_EMMC_HW_RST_MASK);
        dev_log!("[set] power_control: {:#x}\n", PC_EMMC_HW_RST_MASK);
        usleep(1000);
    }

    fn reset(&self, value: u8) -> Result<(), SdmmcError> {
        self.register.software_reset.set(value);
        dev_log!("[set] software_reset: {:#x}\n", value);
        let ret = self.wait_for_event(&self.register.software_reset, Some(value), 0, 100000);
        dev_log!("[wait] software_reset, mask: {:#x}, value {:#x}\n", value, 0);
        ret
    }

    fn enable_bus_power(&self) {
        self.register
            .power_control
            .set((PC_BUS_VSEL_3V3_MASK | PC_BUS_PWR_MASK) & !PC_EMMC_HW_RST_MASK);
        dev_log!("[set] power_control: {:#x}\n", (PC_BUS_VSEL_3V3_MASK | PC_BUS_PWR_MASK) & !PC_EMMC_HW_RST_MASK);
        usleep(200)
    }

    fn reset_config(&self) -> Result<(), SdmmcError> {
        self.disable_bus_power();
        self.reset(SWRST_ALL_MASK)?;
        self.enable_bus_power();
        Ok(())
    }

    fn init_power(&self, host_caps: u32) {
        let power_level: u8;
        if (host_caps & CAP_VOLT_3V3_MASK) != 0 {
            power_level = PC_BUS_VSEL_3V3_MASK;
        } else if (host_caps & CAP_VOLT_3V0_MASK) != 0 {
            power_level = PC_BUS_VSEL_3V0_MASK;
        } else if (host_caps & CAP_VOLT_1V8_MASK) != 0 {
            power_level = PC_BUS_VSEL_1V8_MASK;
        } else {
            power_level = 0;
        }

        self.register
            .power_control
            .set(power_level | PC_BUS_PWR_MASK);
        dev_log!("[set] power_control: {:#x}\n", power_level | PC_BUS_PWR_MASK)
    }

    fn init_dma(&self) {
        self.register.host_control_1.set(HC_DMA_ADMA2_64_MASK);
        dev_log!("[set] host_control_1: {:#x}\n", HC_DMA_ADMA2_64_MASK)
    }

    fn init_interrupt(&mut self) {
        self.register
            .normal_interrupt_status_enable
            .set(NORM_INTR_ALL_MASK & !INTR_CARD_MASK);
        dev_log!("[set] normal_interrupt_status_enable: {:#x}\n", NORM_INTR_ALL_MASK & !INTR_CARD_MASK);
        self.register
            .error_interrupt_status_enable
            .set(ERROR_INTR_ALL_MASK);
        dev_log!("[set] error_interrupt_status_enable: {:#x}\n", ERROR_INTR_ALL_MASK);
        let _ = self.sdmmc_config_interrupt(false, false);
    }

    fn host_config(&mut self, host_caps: u32) {
        self.init_power(host_caps);
        self.init_dma();
        self.init_interrupt();

        self.transfer_mode = TM_DMA_EN_MASK | TM_BLK_CNT_EN_MASK | TM_DAT_DIR_SEL_MASK;

        self.register.block_size.set(BLK_SIZE_512_MASK);
        dev_log!("[set] block_size: {:#x}\n", BLK_SIZE_512_MASK)
    }

    fn cfg_initialize(&mut self) -> u32 {
        let hc_version = self.register.host_controller_version.get() & HC_SPEC_VER_MASK;
        info!("host controller version: {}", hc_version);
        assert_eq!(hc_version, 2);

        let host_caps = self.register.capabilities_0.get();
        info!("host controller capabilities: {:#x}", host_caps);

        if let Err(_) = self.reset_config() {
            panic!("reset failed")
        }

        self.host_config(host_caps);

        host_caps
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

    fn calc_clock(&self, clk_freq: u32) -> u32 {
        let mut divisor: u16 = 0;

        if INPUT_CLOCK_HZ > clk_freq {
            for div_cnt in 1..(CC_EXT_MAX_DIV_CNT / 2 + 1) {
                if INPUT_CLOCK_HZ / ((div_cnt as u32) * 2) <= clk_freq {
                    divisor = div_cnt;
                    break;
                }
            }
        }

        ((divisor as u32 & CC_SDCLK_FREQ_SEL_MASK) << CC_DIV_SHIFT)
            | (((divisor as u32 >> 8) & CC_SDCLK_FREQ_SEL_EXT_MAS) << CC_EXT_DIV_SHIFT)
    }

    fn enable_clock(&self, mut clock: u16) -> Result<(), SdmmcError> {
        clock |= CC_INT_CLK_EN_MASK;
        self.register.clock_control.set(clock);
        dev_log!("[set] clock_control: {:#x}\n", clock);

        self.wait_for_event(
            &self.register.clock_control,
            Some(CC_INT_CLK_STABLE_MASK),
            CC_INT_CLK_STABLE_MASK,
            150000,
        )?;
        dev_log!("[wait] register.clock_control, mask: {}, value: {}\n", CC_INT_CLK_STABLE_MASK, CC_INT_CLK_STABLE_MASK);

        clock |= CC_SD_CLK_EN_MASK;
        self.register.clock_control.set(clock);
        dev_log!("[set] clock_control: {:#x}\n", clock);

        Ok(())
    }

    fn set_clock(&self, clk_freq: u32) -> Result<(), SdmmcError> {
        /* Disable clock */
        self.register.clock_control.set(0);
        dev_log!("[set] clock_control: {:#x}\n", 0);

        if clk_freq == 0 {
            return Err(SdmmcError::EINVAL);
        }

        let clock = self.calc_clock(clk_freq);

        self.enable_clock(clock as u16)
    }

    fn change_clk_freq(&self, clk_freq: u32) -> Result<(), SdmmcError> {
        self.set_tap_delay();
        self.set_clock(clk_freq)
    }

    fn card_initialize(&self, _host_caps: u32) -> Result<(), SdmmcError> {
        self.change_clk_freq(CLK_400_KHZ)?;

        usleep(INIT_DELAY);

        self.register.normal_interrupt_status.set(NORM_INTR_ALL_MASK);
        dev_log!("[set] normal_interrupt_status: {:#x}\n", NORM_INTR_ALL_MASK);
        self.register.error_interrupt_status.set(ERROR_INTR_ALL_MASK);
        dev_log!("[set] error_interrupt_status: {:#x}\n", ERROR_INTR_ALL_MASK);

        self.reset(SWRST_CMD_LINE_MASK)
    }

    fn check_bus_idle(&self, value: u32) -> Result<(), SdmmcError> {
        if self.register.present_state.get() & PSR_CARD_INSRT_MASK != 0 {
            self.wait_for_event(&self.register.present_state, Some(value), 0, 10000000)?;
            dev_log!("[wait] present_state: mask: {:#x}, value: {:#x}\n", value, 0);
        }
        Ok(())
    }

    fn setup_cmd(&self, arg: u32, blk_cnt: u32) -> Result<(), SdmmcError> {
        self.check_bus_idle(PSR_INIHIBIT_CMD_MASK)?;

        self.register.block_count.set(blk_cnt as u16);
        dev_log!("[set] block_count: {:#x}\n", blk_cnt);
        self.register.timeout_control.set(0);
        dev_log!("[set] timeout_control: {:#x}\n", 0);
        self.register.argument.set(arg);
        dev_log!("[set] argument: {:#x}\n", arg);

        // acknowledge interrupt
        self.register
            .normal_interrupt_status
            .set(NORM_INTR_ALL_MASK);
        dev_log!("[set] normal_interrupt_status: {:#x}\n", NORM_INTR_ALL_MASK);
        self.register
            .error_interrupt_status
            .set(ERROR_INTR_ALL_MASK);
        dev_log!("[set] error_interrupt_status: {:#x}\n", ERROR_INTR_ALL_MASK);

        Ok(())
    }

    fn send_cmd(&self, cmdidx: u32, resp_type: u32) {
        if cmdidx != 21 && cmdidx != 19 {
            let _present_state = self.register.present_state.get();
            // dev_log!("[GET] present_state: {:#x}\n", _present_state);
            // todo: fix for data inhibit check
            // if present_state & PSR_INHIBIT_DAT_MASK != 0
        }

        let mut command: u16 = (cmdidx as u16) << 8;
        if (resp_type & MMC_RSP_PRESENT) != 0 {
            if (resp_type & MMC_RSP_136) != 0 {
                command |= 0b01;
            } else if (resp_type & MMC_RSP_BUSY) != 0 {
                command |= 0b11;
            } else {
                command |= 0b10;
            }
        }
        if (resp_type & MMC_RSP_CRC) != 0 {
            command |= 0b1000;
        }
        if (resp_type & MMC_RSP_OPCODE) != 0 {
            command |= 0b10000;
        }

        self.register.transfer_mode.set(self.transfer_mode);
        dev_log!("[set] transfer_mode: {:#x}\n", self.transfer_mode);
        self.register.command.set(command);
        dev_log!("[set] command: {:#x}\n", command);
    }

    fn setup_read_dma(
        &mut self,
        block_count: u32,
        mut block_size: u16,
        buffer_pointer: u64,
    ) -> Result<(), SdmmcError> {
        self.register.block_size.set(block_size & BLK_SIZE_MASK);
        dev_log!("[set] block_size: {:#x}\n", block_size & BLK_SIZE_MASK);
        block_size = self.register.block_size.get();

        if block_size != 512 {
            return Err(SdmmcError::EIO);
        }

        let total_desc_lines: u32;
        if block_count * (block_size as u32) < DESC_MAX_LENGTH {
            total_desc_lines = 1;
        } else {
            total_desc_lines = (block_count * block_size as u32) / DESC_MAX_LENGTH
                + ((block_count * block_size as u32) % DESC_MAX_LENGTH == 0) as u32;
        }

        let ptr = self.memory.cast::<ADMA2Descriptor64>();
        {
            let slice: &mut [ADMA2Descriptor64];
            unsafe {
                slice = &mut (*ptr::slice_from_raw_parts_mut(ptr, SDHCI_DESC_NUMBER));
            }

            for i in 0 .. (total_desc_lines as usize) - 1 {
                slice[i].address = buffer_pointer + (i * (DESC_MAX_LENGTH as usize)) as u64;
                slice[i].attribute = DESC_TRAN | DESC_VALID;
                slice[i].length = 0;
            }

            slice[total_desc_lines as usize - 1].address = buffer_pointer;
            slice[total_desc_lines as usize - 1].attribute = DESC_TRAN | DESC_END | DESC_VALID;
            slice[total_desc_lines as usize - 1].length = block_count as u16 * block_size;

            for i in 0 .. total_desc_lines as usize {
                dev_log!("ADMA desc {}: ", i);
                sdhci_print_adma2_descriptor_64(&slice[i]);
            }
        }

        self.register
            .adma_system_address_64_lo
            .set(self.physical_memory_addr as u32);
        dev_log!("[set] adma_system_address_64_lo: {:#x}\n", self.physical_memory_addr as u32);
        (self.cache_invalidate_function)();

        Ok(())
    }
}

/// Helper methods for registers with special handling.
impl SdmmcHardware for SdhciHost {
    fn sdmmc_init(&mut self) -> Result<(MmcIos, HostInfo, u128), SdmmcError> {
        sdhci_print_capabilities(self.register.capabilities_0.get(), self.register.capabilities_1.get());

        dev_log!("\n<init>\n");
        let host_caps = self.cfg_initialize();
        self.card_initialize(host_caps)?;

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

        dev_log!("============== initialised ===============\n");

        return Ok((ios, info, 0));
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
                if mmc_data.blockcnt == 1 {
                    self.transfer_mode = TM_BLK_CNT_EN_MASK | TM_DAT_DIR_SEL_MASK | TM_DMA_EN_MASK;
                } else {
                    todo!();
                }
                self.setup_read_dma(mmc_data.blockcnt, mmc_data.blocksize as u16, mmc_data.addr)?;
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

        self.send_cmd(cmd.cmdidx, cmd.resp_type);

        Ok(())
    }

    fn sdmmc_receive_response(
        &self,
        cmd: &SdmmcCmd,
        response: &mut [u32; 4],
    ) -> Result<(), SdmmcError> {
        let status = self.register.normal_interrupt_status.extract();

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
            (NORMAL_INTERRUPT::INT_A, "int_a (embedded)"),
            (NORMAL_INTERRUPT::INT_B, "int_b (embedded)"),
            (NORMAL_INTERRUPT::INT_C, "int_c (embedded)"),
            (NORMAL_INTERRUPT::RE_TUNING, "re-tuning"),
            (NORMAL_INTERRUPT::FX, "FX"),
            (NORMAL_INTERRUPT::RESEREVED, "reserved"),
            (NORMAL_INTERRUPT::ERROR_INTERRUPT, "error_interrupt"),
        ];

        dev_log!(
            "\n<RECV> [{} ({})], response: [{}], arg: {:#x}, status:",
            sd_get_cmd_name(cmd.cmdidx),
            cmd.cmdidx,
            sd_get_response_type(cmd.resp_type),
            cmd.cmdarg
        );
        for (field, name) in all_fields.iter() {
            if status.is_set(*field) {
                dev_log!(" {},", name);
            }
        }
        if all_fields.iter().all(|(field, _)| !status.is_set(*field)) {
            dev_log!(" none");
        }
        dev_log!("\n");

        sdhci_print_present_state_string(self.register.present_state.get());

        if (cmd.cmdidx == 19 || cmd.cmdidx == 21) && (status.get() & INTR_BRR_MASK) != 0 {
            self.register.normal_interrupt_status.set(INTR_BRR_MASK);
            dev_log!("[set] normal_interrupt_status: {:#x}\n", INTR_BRR_MASK);
        }

        if (status.get() & INTR_ERR_MASK) != 0 {
            let error: SdmmcError;
            if (self.register.error_interrupt_status.get() & !INTR_ERR_CT_MASK) == 0 {
                error = SdmmcError::ETIMEDOUT;
            } else {
                error = SdmmcError::EINVAL;
            }
            /* Write to clear error bits */
            self.register
                .error_interrupt_status
                .set(ERROR_INTR_ALL_MASK);
            dev_log!("[set] error_interrupt_status: {:#x}\n", ERROR_INTR_ALL_MASK);
            return Err(error);
        }

        if (status.get() & INTR_CC_MASK) != INTR_CC_MASK {
            return Err(SdmmcError::EBUSY);
        }

        // if cmd.cmdidx == 17 {
        //     if (status & INTR_TC_MASK) != INTR_TC_MASK {
        //         return Err(SdmmcError::EBUSY);
        //     }
        // }

        /* Write to clear bit */
        self.register.normal_interrupt_status.set(INTR_CC_MASK);
        dev_log!("[set] normal_interrupt_status: {:#x}\n", INTR_CC_MASK);

        let [rsp0, rsp1, rsp2, rsp3] = response;
        if cmd.resp_type & sdmmc_protocol::sdmmc::MMC_RSP_136 != 0 {
            *rsp0 = self.register.response[3].get();
            *rsp1 = self.register.response[2].get();
            *rsp2 = self.register.response[1].get();
            *rsp3 = self.register.response[0].get();
            dev_log!(
                "response: [{:#x}, {:#x}, {:#x}, {:#x}]\n",
                *rsp0,
                *rsp1,
                *rsp2,
                *rsp3
            );
        } else {
            *rsp0 = self.register.response[0].get();
            dev_log!("response: [{:#x}]\n", *rsp0,);
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
            self.register
                .normal_interrupt_signal_enable
                .set(IRQ_ENABLE_MASK);
            dev_log!("[set] normal_interrupt_signal_enable: {:#x}\n", IRQ_ENABLE_MASK);
            self.register
                .error_interrupt_signal_enable
                .set(ERR_ENABLE_MASK);
            dev_log!("[set] error_interrupt_signal_enable: {:#x}\n", ERR_ENABLE_MASK);
        } else {
            self.register.normal_interrupt_signal_enable.set(0);
            dev_log!("[set] normal_interrupt_signal_enable: {:#x}\n", 0);
            self.register.error_interrupt_signal_enable.set(0);
            dev_log!("[set] error_interrupt_signal_enable: {:#x}\n", 0);
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
