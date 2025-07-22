//! SD Host Controller Register Block Definition
//!
//! Generated using the `tock_registers::register_structs!` macro for a
//! declarative, offset-based definition.

#![allow(dead_code)] // Allow unused fields, as a driver may not use all registers.

use sdmmc_protocol::sdmmc::mmc_struct::{MmcBusWidth, MmcTiming};
use sdmmc_protocol::sdmmc::{HostInfo, MmcData, MmcIos, SdmmcCmd, SdmmcError};
use sdmmc_protocol::sdmmc_traits::SdmmcHardware;
use tock_registers::register_bitfields;
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};

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
    ],
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
    pub SdhciHost {
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
        (0x058 => adma_system_address_64: ReadWrite<u64>),
        (0x060 => preset_value: [ReadOnly<u16>; 8]),
        (0x070 => _reserved2),
        (0x078 => adma3_id_address_64: ReadWrite<u64>),
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

/// Helper methods for registers with special handling.
impl SdmmcHardware for SdhciHost {
    fn sdmmc_init(&mut self) -> Result<(MmcIos, HostInfo, u128), SdmmcError> {
        return Err(SdmmcError::ENOTIMPLEMENTED);
    }

    fn sdmmc_config_timing(&mut self, timing: MmcTiming) -> Result<u64, SdmmcError> {
        return Err(SdmmcError::ENOTIMPLEMENTED);
    }

    fn sdmmc_config_bus_width(&mut self, bus_width: MmcBusWidth) -> Result<(), SdmmcError> {
        return Err(SdmmcError::ENOTIMPLEMENTED);
    }

    fn sdmmc_read_datalanes(&self) -> Result<u8, SdmmcError> {
        return Err(SdmmcError::ENOTIMPLEMENTED);
    }

    fn sdmmc_send_command(
        &mut self,
        cmd: &SdmmcCmd,
        data: Option<&MmcData>,
    ) -> Result<(), SdmmcError> {
        return Err(SdmmcError::ENOTIMPLEMENTED);
    }

    fn sdmmc_receive_response(
        &self,
        cmd: &SdmmcCmd,
        response: &mut [u32; 4],
    ) -> Result<(), SdmmcError> {
        return Err(SdmmcError::ENOTIMPLEMENTED);
    }

    fn sdmmc_config_interrupt(
        &mut self,
        enable_irq: bool,
        enable_sdio_irq: bool,
    ) -> Result<(), SdmmcError> {
        return Err(SdmmcError::ENOTIMPLEMENTED);
    }

    fn sdmmc_ack_interrupt(&mut self) -> Result<(), SdmmcError> {
        return Err(SdmmcError::ENOTIMPLEMENTED);
    }
    
    fn sdmmc_execute_tuning(
        &mut self,
        memory: *mut [u8; 64],
        sleep: &mut dyn sdmmc_protocol::sdmmc_os::Sleep,
    ) -> Result<(), SdmmcError> {
        return Err(SdmmcError::ENOTIMPLEMENTED);
    }
    
    fn sdmmc_host_reset(&mut self) -> Result<sdmmc_protocol::sdmmc::MmcIos, SdmmcError> {
        return Err(SdmmcError::ENOTIMPLEMENTED);
    }
}