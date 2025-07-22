//! SD Host Controller Register Block Definition
//!
//! Generated using the `tock_registers::register_structs!` macro for a
//! declarative, offset-based definition.

#![allow(dead_code)] // Allow unused fields, as a driver may not use all registers.

use sdmmc_protocol::{dev_log, info};
use sdmmc_protocol::sdmmc::mmc_struct::{MmcBusWidth, MmcTiming};
use sdmmc_protocol::sdmmc::sdmmc_capability::{MMC_VDD_31_32, MMC_VDD_32_33, MMC_VDD_33_34};
use sdmmc_protocol::sdmmc::{HostInfo, MmcData, MmcIos, MmcPowerMode, MmcSignalVoltage, SdmmcCmd, SdmmcError};
use sdmmc_protocol::sdmmc_os::process_wait_unreliable;
use sdmmc_protocol::sdmmc_traits::SdmmcHardware;
use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::register_bitfields;
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};

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
const CC_INT_CLK_EN_MASK: u32 = 0x00000001;
const CC_SD_CLK_EN_MASK: u32 = 0x00000004;
const CC_INT_CLK_STABLE_MASK: u16 = 0x00000002;

const INIT_DELAY: u64 = 100000;

const CLK_400_KHZ: u32 = 400000;

const PSR_CARD_INSRT_MASK: u32 = 0x00010000;
const PSR_INIHIBIT_CMD_MASK: u32 = 0x00000001;
const PSR_INHIBIT_DAT_MASK: u32 = 0x00000002;

const NORM_INTR_ALL_MASK: u16 = 0x0000FFFF;
const ERROR_INTR_ALL_MASK: u16 = 0x0000F3FF;

const XWRST_ALL_MASK: u8 = 0x00000001;

const CAP_VOLT_3V3_MASK: u32 = 0x01000000;
const CAP_VOLT_3V0_MASK: u32 = 0x02000000;
const CAP_VOLT_1V8_MASK: u32 = 0x04000000;

const HC_DMA_ADMA2_64_MASK: u8 = 0x00000018;

const INTR_CARD_MASK: u16 = 0x00000100;

const BLK_SIZE_512_MASK: u16 = 0x200;

const TM_DMA_EN_MASK: u16 = 0x00000001;
const TM_BLK_CNT_EN_MASK: u16 = 0x00000002;
const TM_DAT_DIR_SEL_MASK: u16 = 0x00000010;

const INTR_ERR_MASK: u16 = 0x00008000;
const INTR_CC_MASK: u16 = 0x00000001;

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

impl SdhciRegister {
    unsafe fn new(sdmmc_register_base: u64) -> &'static mut SdhciRegister {
        unsafe { &mut *(sdmmc_register_base as *mut SdhciRegister) }
    }
}

pub struct SdhciHost {
    register: &'static mut SdhciRegister,
}

const NS_IN_US: u64 = 1000;
fn usleep(time : u64) {
    process_wait_unreliable(time * NS_IN_US);
}

impl SdhciHost {
    pub unsafe fn new(sdmmc_register_base: u64) -> Self {
        let register: &'static mut SdhciRegister =
            unsafe { SdhciRegister::new(sdmmc_register_base) };

        // TODO: Call reset function here
        SdhciHost { register }
    }

    fn disable_bus_power(&self) {
        self.register.power_control.set(PC_EMMC_HW_RST_MASK);
        usleep(1000);
    }

    fn check_reset_done(&self) -> Result<(), SdmmcError> {
        let mut timeout: u32 = 100000;
        while timeout > 0 {
            if self.register.software_reset.get() == 0 {
                return Ok(());
            }
            timeout -= 1;
            usleep(1);
        }
        panic!("timeout")
    }

    fn reset(&self, value: u8) -> Result<(), SdmmcError> {
        self.register.software_reset.set(value);
        self.check_reset_done()
    }

    fn enable_bus_power(&self) {
        self.register.power_control.set((PC_BUS_VSEL_3V3_MASK | PC_BUS_PWR_MASK) & !PC_EMMC_HW_RST_MASK);
        usleep(200)
    }

    fn reset_config(&self) -> Result<(), SdmmcError> {
        self.disable_bus_power();
        if let Err(error) = self.reset(XWRST_ALL_MASK) {
            return Err(error);
        }
        self.enable_bus_power();
        Ok(())
    }

    fn config_power(&self, host_caps: u32) {
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

        self.register.power_control.set(power_level | PC_BUS_PWR_MASK)
    }

    fn config_dma(&self) {
        self.register.host_control_1.set(HC_DMA_ADMA2_64_MASK)
    }

    fn config_interrupt(&mut self) {
        self.register.normal_interrupt_status_enable.set(NORM_INTR_ALL_MASK & (!INTR_CARD_MASK));
        self.register.error_interrupt_status_enable.set(ERROR_INTR_ALL_MASK);
        let _ = self.sdmmc_config_interrupt(false, false);
    }

    fn host_config(&mut self, host_caps: u32) {
        self.config_power(host_caps);
        self.config_dma();
        self.config_interrupt();

        /*
        * Transfer mode register - default value
        * DMA enabled, block count enabled, data direction card to host(read)
        */
        // let tm_dma_en_mask: u32 = 0x00000001;
        // let tm_blk_cnt_en_mask: u32 = 0x00000002;
        // let tm_dat_dir_sel_mask: u32 = 0x00000010;
        // unsafe {
        //     TRANSFER_MODE = tm_dma_en_mask | tm_blk_cnt_en_mask | tm_dat_dir_sel_mask;
        // }

        /* Set block size to 512 by default */
        self.register.block_size.set(BLK_SIZE_512_MASK)
    }

    fn cfg_initialize(&mut self) -> u32 {
        let hc_version = self.register.host_controller_version.get() & HC_SPEC_VER_MASK;
        info!("host controller version: {}", hc_version);
        assert_eq!(hc_version, 2);

        let host_caps = self.register.capabilities_0.get();
        info!("host controller capabilities: {}", host_caps);

        if let Err(_) = self.reset_config() {
            panic!("reset failed")
        }

        self.host_config(host_caps);

        host_caps
    }

    fn set_tap_delay(&self) {
        // TODO
    }

    fn calc_clock(&self, clk_freq: u32) -> u32 {
        let mut divisor: u16 = 0;

        if INPUT_CLOCK_HZ > clk_freq {
            for div_cnt in (2 .. CC_EXT_MAX_DIV_CNT + 2).step_by(2) {
                if INPUT_CLOCK_HZ / (div_cnt as u32) <= clk_freq {
                    divisor = div_cnt >> 1;
                    break;
                }
            }
        }

        ((divisor as u32 & CC_SDCLK_FREQ_SEL_MASK) << CC_DIV_SHIFT) | (((divisor as u32 >> 8) & CC_SDCLK_FREQ_SEL_EXT_MAS) << CC_EXT_DIV_SHIFT)
    }

    fn enable_clock(&self, mut clock: u16) -> Result<(), SdmmcError> {
        clock |= CC_INT_CLK_EN_MASK as u16;
        self.register.clock_control.set(clock);

        let mut timeout: u32 = 150000;
        while timeout > 0 {
            if self.register.clock_control.get() & CC_INT_CLK_STABLE_MASK == CC_INT_CLK_STABLE_MASK {
                break;
            }
            timeout -= 1;
            usleep(1);
        }
        if timeout == 0 {
            panic!("timeout");
        }

        /* Enable SD clock */
        clock |= CC_SD_CLK_EN_MASK as u16;
        self.register.clock_control.set(clock);

        Ok(())
    }

    fn set_clock(&self, clk_freq: u32) -> Result<(), SdmmcError> {
        /* Disable clock */
        self.register.clock_control.set(0);

        if clk_freq == 0 {
            return Err(SdmmcError::EINVAL);
        }

        let clock = self.calc_clock(clk_freq);
        dev_log!("clock frequency: {}, clock divisor: {}, freq/divisor: {}\n", clk_freq, clock, clk_freq / clock);

        self.enable_clock(clock as u16)
    }

    fn change_clk_freq(&self, clk_freq: u32) -> Result<(), SdmmcError> {
        self.set_clock(clk_freq)
    }

    fn card_initialize(&self, _host_caps: u32) -> Result<(), SdmmcError> {
        if let Err(error) = self.change_clk_freq(CLK_400_KHZ) {
            return Err(error);
        }

        usleep(INIT_DELAY);

        Ok(())
    }

    fn check_bus_idle(&self, value: u32) -> Result<(), SdmmcError> {
        if self.register.present_state.get() & PSR_CARD_INSRT_MASK != 0 {
            let mut timeout = 10000000;
            while timeout > 0 {
                if self.register.present_state.get() & value == 0 {
                    break;
                }
                timeout -= 1;
                usleep(1);
            }
            if timeout == 0 {
                return Err(SdmmcError::EBUSY);
            }
        }
        Ok(())
    }

    fn setup_cmd(&self, arg: u32, blk_cnt: u32) -> Result<(), SdmmcError> {
        self.check_bus_idle(PSR_INIHIBIT_CMD_MASK)?;

        self.register.block_count.set(blk_cnt as u16);
        self.register.timeout_control.set(0xE);
        self.register.argument.set(arg);

        // acknowledge interrupt
        self.register.normal_interrupt_status.set(NORM_INTR_ALL_MASK);
        self.register.error_interrupt_status.set(ERROR_INTR_ALL_MASK);

        dev_log!("[SET] block_count: 0x{:x}, timeout_control: 0x{:x}, argument: 0x{:x}\n", self.register.block_count.get(), self.register.timeout_control.get(), self.register.argument.get());

        Ok(())
    }

    fn send_cmd(&self, cmdidx: u32, _resp_type: u32) -> Result<(), SdmmcError> {
        if cmdidx != 21 && cmdidx != 19 {
            let _present_state = self.register.present_state.get();
            dev_log!("[GET] present_state: 0x{:x}\n", _present_state);
            // todo: fix for data inhibit check
            // if present_state & PSR_INHIBIT_DAT_MASK != 0 
        }
        let command: u16;
        if cmdidx == 0 {
            command = ((cmdidx as u16) << 8);
        } else {
            command = ((cmdidx as u16) << 8) | 0b11010;
        }
        self.register.command.set(command);
        self.register.transfer_mode.set(TM_DMA_EN_MASK | TM_BLK_CNT_EN_MASK | TM_DAT_DIR_SEL_MASK);

        dev_log!("[SET] command: 0x{:x}, transfer_mode: 0x{:x}\n", command, self.register.transfer_mode.get());
        Ok(())
    }
}

/// Helper methods for registers with special handling.
impl SdmmcHardware for SdhciHost {
    fn sdmmc_init(&mut self) -> Result<(MmcIos, HostInfo, u128), SdmmcError> {
        let host_caps = self.cfg_initialize();

        if let Err(error) = self.card_initialize(host_caps) {
            return Err(error);
        }

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
            max_block_per_req: 1, // ???
            // On odroid c4, the operating voltage is default to 3.3V
            vdd: (MMC_VDD_33_34 | MMC_VDD_32_33 | MMC_VDD_31_32), // ???
            // TODO, figure out the correct value when we can power the card on and off
            power_delay_ms: 5,
        };

        return Ok((ios, info, 0));
    }

    fn sdmmc_config_timing(&mut self, timing: MmcTiming) -> Result<u64, SdmmcError> {
        Ok(CLK_400_KHZ as u64)
    }

    fn sdmmc_config_bus_width(&mut self, bus_width: MmcBusWidth) -> Result<(), SdmmcError> {
        Ok(())
    }

    fn sdmmc_read_datalanes(&self) -> Result<u8, SdmmcError> {
        return Err(SdmmcError::ENOTIMPLEMENTED);
    }

    fn sdmmc_send_command(
        &mut self,
        cmd: &SdmmcCmd,
        data: Option<&MmcData>,
    ) -> Result<(), SdmmcError> {
         if let Some(_) = data {
            todo!()
        }

        // When cmd is send status or stop transmission, the sdcard can execute those when trasferring data
        // if cmd.cmdidx != 13 && self.register.present_state.get() {
        //     todo!("wait for data transform")
        // }

        self.setup_cmd(cmd.cmdarg, 0)?;

        self.send_cmd(cmd.cmdidx, cmd.resp_type)
    }

    fn sdmmc_receive_response(
        &self,
        cmd: &SdmmcCmd,
        response: &mut [u32; 4],
    ) -> Result<(), SdmmcError> {
        // command complete/error
        // present_status data lane in use
        
        let status = self.register.normal_interrupt_status.get();
        if (status & INTR_ERR_MASK) == INTR_ERR_MASK {
            todo!();
            return Err(SdmmcError::EUNKNOWN);
        }

        dev_log!("status: {}\n", status);

        if (status & INTR_CC_MASK) != INTR_CC_MASK {
            return Err(SdmmcError::EBUSY);
        }

        if cmd.cmdidx == 19 || cmd.cmdidx == 21 {
            todo!()
        }

        /* Write to clear bit */
        self.register.normal_interrupt_status.set(INTR_CC_MASK);

        let [rsp0, rsp1, rsp2, rsp3] = response;
        if cmd.resp_type & sdmmc_protocol::sdmmc::MMC_RSP_136 != 0 {
            *rsp0 = self.register.response.get(3).unwrap().get();
            *rsp1 = self.register.response.get(2).unwrap().get();
            *rsp2 = self.register.response.get(1).unwrap().get();
            *rsp3 = self.register.response.get(0).unwrap().get();
            dev_log!("response: {}, {}, {}, {}", *rsp0, *rsp1, *rsp2, *rsp3);
        } else {
            *rsp0 = self.register.response.get(0).unwrap().get();
            dev_log!("response: 0x{:x}\n", *rsp0);
        }

        Ok(())
    }

    fn sdmmc_config_interrupt(
        &mut self,
        enable_irq: bool,
        _enable_sdio_irq: bool,
    ) -> Result<(), SdmmcError> {
        if enable_irq {
            self.register.normal_interrupt_signal_enable.set(IRQ_ENABLE_MASK);
            self.register.error_interrupt_signal_enable.set(ERR_ENABLE_MASK);
        }
        else {
            self.register.normal_interrupt_signal_enable.set(0);
            self.register.error_interrupt_signal_enable.set(0);
        }

        return Ok(());
    }

    // Should I remove this method and auto ack the irq in the receive response fcuntion?
    fn sdmmc_ack_interrupt(&mut self) -> Result<(), SdmmcError> {
        Ok(())
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
