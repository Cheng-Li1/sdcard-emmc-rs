use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use mmc_struct::{BlockDevice, MmcBusWidth, MmcState, MmcTiming};
use sdcard::{Cid, Csd, Sdcard};
use sdmmc_capability::{SdmmcHostCapability, MMC_CAP_VOLTAGE_TUNE, MMC_TIMING_UHS_SDR12};
use sdmmc_constant::{
    INIT_CLOCK_RATE, MMC_CMD_ALL_SEND_CID, MMC_CMD_APP_CMD, MMC_CMD_GO_IDLE_STATE, MMC_CMD_READ_MULTIPLE_BLOCK, MMC_CMD_READ_SINGLE_BLOCK, MMC_CMD_SELECT_CARD, MMC_CMD_SEND_CSD, MMC_CMD_STOP_TRANSMISSION, MMC_CMD_WRITE_MULTIPLE_BLOCK, MMC_CMD_WRITE_SINGLE_BLOCK, MMC_VDD_165_195, MMC_VDD_32_33, MMC_VDD_33_34, OCR_BUSY, OCR_HCS, OCR_S18R, SD_CMD_APP_SEND_OP_COND, SD_CMD_SEND_IF_COND, SD_CMD_SEND_RELATIVE_ADDR
};

pub mod sdmmc_capability;
mod mmc_struct;
mod sdmmc_constant;
mod sdcard;

pub struct SdmmcCmd {
    pub cmdidx: u32,
    pub resp_type: u32,
    pub cmdarg: u32,
}

pub struct MmcData {
    // The size of the block(sector size), for sdcard should almost always be 512
    pub blocksize: u32,
    // Number of blocks to transfer
    pub blockcnt: u32,
    pub flags: MmcDataFlag,
    pub addr: u64,
}

pub enum MmcDataFlag {
    SdmmcDataRead,
    SdmmcDataWrite,
}

pub enum SdmmcHalError {
    // Error for result not ready yet
    EBUSY,
    ETIMEDOUT,
    EINVAL,
    EIO,
    EUNSUPPORTEDCARD,
    ENOTIMPLEMENTED,
    // This error should not be triggered unless there are bugs in program
    EUNDEFINED,
    // The block transfer succeed, but fail to stop the read/write process
    ESTOPCMD,
}

// Interrupt related define
#[repr(u32)] // Ensures the enum variants are stored as 32-bit integers
pub enum InterruptType {
    Success = 0b0001, // 1st bit
    Error = 0b0010,   // 2nd bit
    SDIO = 0b0100,    // 3rd bit
                      // You can add more flags as needed
}

// Define the MMC response flags
const MMC_RSP_PRESENT: u32 = 1 << 0;
const MMC_RSP_136: u32 = 1 << 1; // 136-bit response
const MMC_RSP_CRC: u32 = 1 << 2; // Expect valid CRC
const MMC_RSP_BUSY: u32 = 1 << 3; // Card may send busy
const MMC_RSP_OPCODE: u32 = 1 << 4; // Response contains opcode

// Define the MMC response types
pub const MMC_RSP_NONE: u32 = 0;
pub const MMC_RSP_R1: u32 = MMC_RSP_PRESENT | MMC_RSP_CRC | MMC_RSP_OPCODE;
pub const MMC_RSP_R1B: u32 = MMC_RSP_PRESENT | MMC_RSP_CRC | MMC_RSP_OPCODE | MMC_RSP_BUSY;
pub const MMC_RSP_R2: u32 = MMC_RSP_PRESENT | MMC_RSP_136 | MMC_RSP_CRC;
pub const MMC_RSP_R3: u32 = MMC_RSP_PRESENT;
pub const MMC_RSP_R4: u32 = MMC_RSP_PRESENT;
pub const MMC_RSP_R5: u32 = MMC_RSP_PRESENT | MMC_RSP_CRC | MMC_RSP_OPCODE;
pub const MMC_RSP_R6: u32 = MMC_RSP_PRESENT | MMC_RSP_CRC | MMC_RSP_OPCODE;
pub const MMC_RSP_R7: u32 = MMC_RSP_PRESENT | MMC_RSP_CRC | MMC_RSP_OPCODE;

// Enums for power_mode
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MmcPowerMode {
    Off = 0,
    Up = 1,
    On = 2,
    Undefined = 3,
}

// Signal voltage
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MmcSignalVoltage {
    Voltage330 = 0,
    Voltage180 = 1,
    Voltage120 = 2,
}

// Driver type
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MmcDriverType {
    TypeB = 0,
    TypeA = 1,
    TypeC = 2,
    TypeD = 3,
}

// Enums for bus_mode
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MmcBusMode {
    OpenDrain = 1,
    PushPull = 2,
}

// Enums for chip_select
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MmcChipSelect {
    DontCare = 0,
    High = 1,
    Low = 2,
}

/// Settings specific to eMMC cards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmmcSettings {
    /// The drive strength of the host driver, typically relevant for eMMC devices.
    ///
    /// - The drive strength affects signal integrity and is selected based on the card's
    ///   operating conditions, such as bus load and speed.
    /// - The eMMC specification defines four possible driver types (A, B, C, D) that
    ///   optimize for different use cases and electrical environments:
    ///   - `DriverType::TypeB`: Default driver strength for most cases.
    ///   - `DriverType::TypeA`, `TypeC`, `TypeD`: Other driver types based on signal
    ///     strength requirements.
    pub drv_type: MmcDriverType,

    /// Specifies whether **HS400 Enhanced Strobe** mode is enabled.
    ///
    /// - Enhanced Strobe is used in **HS400** mode for eMMC devices to improve data
    ///   reliability at high speeds. It allows more accurate data capture by aligning
    ///   strobe signals with data.
    /// - This is only relevant for eMMC cards in **HS400ES** mode.
    pub enhanced_strobe: bool,
}

/// Settings specific to SPI communication mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpiSettings {
    /// The chip select mode used in **SPI mode** communication.
    ///
    /// - This field is relevant only when the SD/MMC host controller is operating in **SPI mode**.
    ///   In **native SD/MMC protocol**, this field is not used.
    ///
    /// - The **chip select (CS)** pin is used to activate or deactivate the SD/MMC card on the SPI bus.
    ///   It allows the host to select which device it is communicating with when multiple devices share the same bus.
    ///
    /// - Possible values:
    ///   - `MmcChipSelect::DontCare`: The chip select state is ignored by the host.
    ///   - `MmcChipSelect::High`: The chip select pin is driven high, indicating that the card is not selected.
    ///   - `MmcChipSelect::Low`: The chip select pin is driven low, indicating that the card is selected and active.
    ///
    /// **Note**:
    /// - In **native SD/MMC mode**, communication happens via dedicated **command and data lines** without the need for chip select.
    /// - In most applications, **SPI mode** is less commonly used, especially in high-performance systems.
    pub chip_select: MmcChipSelect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// The `MmcIos` struct represents the I/O settings for the SD/MMC controller,
/// configuring how the host communicates with the card during various operations.
pub struct MmcIos {
    /// The clock rate (in Hz) used for communication with the SD/MMC card.
    ///
    /// - This field specifies the frequency at which data is transferred between
    ///   the host and the card. The clock can vary depending on the mode the card
    ///   is in (e.g., initialization, data transfer).
    /// - Typically, initialization occurs at a lower clock rate, and high-speed
    ///   data transfer occurs at higher rates.
    pub clock: u64,

    /// The voltage range (VDD) used for powering the SD/MMC card.
    ///
    /// - This field stores the selected voltage range in a bit-encoded format.
    ///   It indicates the voltage level the card is operating at.
    /// - Common voltage levels are 3.3V, 1.8V, and sometimes 1.2V (for eMMC).
    /// - Cards often negotiate their operating voltage during initialization.
    pub vdd: u16,

    /// The power delay (in milliseconds) used after powering the card to ensure
    /// stable operation.
    ///
    /// - After powering up the card, the host controller typically waits for a
    ///   certain period before initiating communication to ensure that the card's
    ///   power supply is stable.
    /// - This delay ensures the card is ready to respond to commands.
    pub power_delay_ms: u32,

    /// The current power supply mode for the SD/MMC card.
    ///
    /// - This field indicates whether the card is powered on, powered off, or
    ///   being powered up. The power mode can affect the card's internal state
    ///   and availability for communication.
    /// - Possible values:
    ///   - `PowerMode::Off`: The card is completely powered off.
    ///   - `PowerMode::Up`: The card is in the process of powering up.
    ///   - `PowerMode::On`: The card is fully powered and ready for communication.
    pub power_mode: MmcPowerMode,

    /// The width of the data bus used for communication between the host and the card.
    ///
    /// - This field specifies whether the bus operates in 1-bit, 4-bit, or 8-bit mode.
    /// - Wider bus widths (4-bit, 8-bit) enable higher data transfer rates, but not all
    ///   cards or host controllers support every bus width.
    /// - Common values:
    ///   - `BusWidth::Width1`: 1-bit data width (lowest speed, used during initialization).
    ///   - `BusWidth::Width4`: 4-bit data width (common for SD cards).
    ///   - `BusWidth::Width8`: 8-bit data width (mainly for eMMC).
    pub bus_width: MmcBusWidth,

    /// The signaling voltage level used for communication with the card.
    ///
    /// - Different SD/MMC cards support different signaling voltage levels. This field
    ///   indicates the voltage level used for signaling between the host and the card.
    /// - Common voltage levels:
    ///   - `SignalVoltage::Voltage330`: 3.3V signaling.
    ///   - `SignalVoltage::Voltage180`: 1.8V signaling.
    ///   - `SignalVoltage::Voltage120`: 1.2V signaling (mainly for newer eMMC devices).
    pub signal_voltage: MmcSignalVoltage,

    /// The bus mode for communication between the host and the card.
    ///
    /// This field defines how the host drives the command lines when communicating with the SD/MMC card.
    /// It is mainly relevant during the initialization process or in certain low-speed configurations.
    ///
    /// - **Open-Drain Mode** (`MmcBusMode::OpenDrain`):
    ///   - In open-drain mode, the command line is driven low by the host or the card but pulled high by a resistor.
    ///   - This mode is typically used during the **initialization phase** or when there are multiple cards on the same bus.
    ///   - It allows multiple devices to safely share the bus without causing signal contention.
    ///
    /// - **Push-Pull Mode** (`MmcBusMode::PushPull`):
    ///   - In push-pull mode, the host actively drives the command line both high and low.
    ///   - This mode is used for **high-speed data transfers** after initialization, where higher performance is required.
    ///
    /// Typically, **push-pull mode** is used once the card is fully initialized and the bus is stable.
    /// In most cases, you don't need to manually configure the bus mode because modern controllers handle this automatically.
    pub bus_mode: Option<MmcBusMode>,

    /// eMMC-specific settings, if applicable.
    ///
    /// This field is `None` if the card is not an eMMC card.
    pub emmc: Option<EmmcSettings>,

    /// SPI-specific settings, if applicable.
    ///
    /// This field is `None` if the card is not operating in SPI mode.
    pub spi: Option<SpiSettings>,
}

pub struct HostInfo {
    max_frequency: u64,
    min_frequency: u64,
    max_block_per_req: u32,
    enabled_irq: u32,
}

/// Program async Rust can be very dangerous if you do not know what is happening understand the hood
/// Power up and power off cannot be properly implemented if I do not have access to control gpio/ regulator and timer
pub trait SdmmcHardware {
    fn sdmmc_power_up(&mut self) -> Result<(), SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }

    fn sdmmc_init(
        &mut self,
    ) -> (
        Result<(), SdmmcHalError>,
        Option<MmcIos>,
        Option<HostInfo>,
        Option<u128>,
    ) {
        return (Err(SdmmcHalError::ENOTIMPLEMENTED), None, None, None);
    }

    // Change the clock, return the value or do not change it at all
    fn sdmmc_config_clock(&mut self, freq: u64) -> Result<u64, SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }

    fn sdmmc_config_bus_width(&mut self, bus_width: MmcBusWidth) -> Result<(), SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }

    fn sdmmc_tune_signal_voltage(&mut self, voltage: u32) -> Result<(), SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }

    fn sdmmc_send_command(
        &mut self,
        cmd: &SdmmcCmd,
        data: Option<&MmcData>,
    ) -> Result<(), SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }

    fn sdmmc_receive_response(
        &self,
        cmd: &SdmmcCmd,
        response: &mut [u32; 4],
    ) -> Result<(), SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }

    fn sdmmc_enable_interrupt(&mut self, irq_to_enable: &mut u32) -> Result<(), SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }

    fn sdmmc_ack_interrupt(&mut self, irq_enabled: &u32) -> Result<(), SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }

    fn sdmmc_power_off(&mut self) -> Result<(), SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }
}

/// TODO: Add more variables for SdmmcProtocol to track the state of the sdmmc controller and card correctly
pub struct SdmmcProtocol<'a, T: SdmmcHardware> {
    pub hardware: &'a mut T,

    mmc_ios: MmcIos,

    host_info: HostInfo,

    cap: SdmmcHostCapability,
}

impl<T> Unpin for SdmmcProtocol<'_, T> where T: Unpin + SdmmcHardware {}

impl<'a, T: SdmmcHardware> SdmmcProtocol<'a, T> {
    pub fn new(hardware: &'a mut T) -> Result<Self, SdmmcHalError> {
        let (res, ios, info, cap) = hardware.sdmmc_init();
        res.unwrap_err();
        let host_cap: SdmmcHostCapability =
            SdmmcHostCapability(cap.ok_or(SdmmcHalError::ENOTIMPLEMENTED)?);
        let mmc_ios: MmcIos = ios.ok_or(SdmmcHalError::ENOTIMPLEMENTED)?;
        let host_info: HostInfo = info.ok_or(SdmmcHalError::ENOTIMPLEMENTED)?;

        Ok(SdmmcProtocol {
            hardware,
            mmc_ios,
            host_info,
            cap: host_cap,
        })
    }

    // Funtion that is not completed
    pub fn setup_card(&mut self) -> Result<BlockDevice, SdmmcHalError> {
        {
            let mut irq: u32 = 0;
            let _ = self.hardware.sdmmc_enable_interrupt(&mut irq);
        }

        // TODO: Different sdcard and eMMC support different voltages, figure those out
        if self.mmc_ios.vdd != 330 {
            self.hardware.sdmmc_power_up()?;
            self.mmc_ios.vdd = 330;
        }

        let clock = self.hardware.sdmmc_config_clock(INIT_CLOCK_RATE)?;

        self.mmc_ios.clock = clock;

        self.hardware.sdmmc_config_bus_width(MmcBusWidth::Width1)?;

        self.mmc_ios.bus_width = MmcBusWidth::Width1;

        let mut resp: [u32; 4] = [0; 4];

        let mut cmd = SdmmcCmd {
            cmdidx: MMC_CMD_GO_IDLE_STATE,
            resp_type: MMC_RSP_NONE,
            cmdarg: 0,
        };

        // This command does not expect a response
        self.hardware.sdmmc_send_command(&cmd, None)?;

        cmd = SdmmcCmd {
            cmdidx: SD_CMD_SEND_IF_COND,
            resp_type: MMC_RSP_R7,
            cmdarg: 0x000001AA, // Voltage supply and check pattern
        };

        let res =
            SdmmcProtocol::<'a, T>::send_cmd_and_receive_resp(self.hardware, &cmd, None, &mut resp);

        // If the result is OK and the resp is 0x1AA, the card we are initializing is a SDHC/SDXC
        // If the result is error, it is either the voltage not being set up correctly, which mean a bug in hardware layer
        // or the card is eMMC or legacy sdcard
        // For now, we only deal with the situation it is a sdcard
        if res.is_ok() && resp[0] == 0x1AA {
            let card: Sdcard = self.setup_sdhc_cont()?;
            Ok(BlockDevice::Sdcard(card))
        } else {
            // TODO: Implement setup for eMMC and legacy sdcard(SDSC) here
            Err(SdmmcHalError::EUNSUPPORTEDCARD)
        }
    }

    fn setup_sdhc_cont(&mut self) -> Result<Sdcard, SdmmcHalError> {
        let mut cmd: SdmmcCmd;
        let mut resp: [u32; 4] = [0; 4];
        // Uboot define this value to be 1000...
        let mut timeout: u16 = 1000;
        loop {
            // Prepare CMD55 (APP_CMD)
            cmd = SdmmcCmd {
                cmdidx: MMC_CMD_APP_CMD,
                resp_type: MMC_RSP_R1,
                cmdarg: 0,
            };

            // Send CMD55
            Self::send_cmd_and_receive_resp(
                self.hardware,
                &cmd,
                None,
                &mut resp,
            )?;

            cmd = SdmmcCmd {
                cmdidx: SD_CMD_APP_SEND_OP_COND,
                resp_type: MMC_RSP_R3,
                cmdarg: 1,
            };

            // Set the HCS bit if version is SD Version 2
            // since the flow has reached here, the card would be at least SD Version 2
            cmd.cmdarg |= OCR_HCS;

            // TODO: Right now the operating voltage is hardcoded, could this have unintended behavior for legacy device
            // And maybe change this when we are trying to support UHS I
            // Change this when we decide to support spi or SDSC as well
            cmd.cmdarg |= (MMC_VDD_33_34 | MMC_VDD_32_33) & 0xff8000;

            if self.cap.contains(sdmmc_capability::SdmmcHostCapability(
                MMC_TIMING_UHS_SDR12 | MMC_CAP_VOLTAGE_TUNE,
            )) {
                cmd.cmdarg |= OCR_S18R;
                cmd.cmdarg |= MMC_VDD_165_195;
            }

            // Send ACMD41
            SdmmcProtocol::<'a, T>::send_cmd_and_receive_resp(
                self.hardware,
                &cmd,
                None,
                &mut resp,
            )?;

            // Check if card is ready (OCR_BUSY bit)
            if (resp[0] & OCR_BUSY) != 0 {
                break;
            }

            // Timeout handling
            if timeout <= 0 {
                return Err(SdmmcHalError::EUNSUPPORTEDCARD);
            }
            timeout -= 1;
        }

        // 4. Send CMD2 to get the CID register
        cmd = SdmmcCmd {
            cmdidx: MMC_CMD_ALL_SEND_CID,
            resp_type: MMC_RSP_R2,
            cmdarg: 0,
        };
        Self::send_cmd_and_receive_resp(self.hardware, &cmd, None, &mut resp)?;

        let cid = Cid::new(resp);

        let card_id = ((resp[0] as u128) << 96)
        | ((resp[1] as u128) << 64)
        | ((resp[2] as u128) << 32)
        | (resp[3] as u128);

        // 5. Send CMD3 to set and receive the RCA
        cmd = SdmmcCmd {
            cmdidx: SD_CMD_SEND_RELATIVE_ADDR,
            resp_type: MMC_RSP_R6,
            cmdarg: 0,
        };

        Self::send_cmd_and_receive_resp(self.hardware, &cmd, None, &mut resp)?;

        let rca: u16 = (resp[0] >> 16) as u16; // Store RCA from response

        // 6. Send CMD9 to get the CSD register
        cmd = SdmmcCmd {
            cmdidx: MMC_CMD_SEND_CSD,
            resp_type: MMC_RSP_R2,
            cmdarg: (rca as u32) << 16,
        };
        Self::send_cmd_and_receive_resp(self.hardware, &cmd, None, &mut resp)?;

        let (csd, card_version) = Csd::new(resp);

        // 7. Send CMD7 to select the card
        cmd = SdmmcCmd {
            cmdidx: MMC_CMD_SELECT_CARD,
            resp_type: MMC_RSP_R1,
            cmdarg: (rca as u32) << 16,
        };
        Self::send_cmd_and_receive_resp(self.hardware, &cmd, None, &mut resp)?;

        let card_state: MmcState = MmcState {
            timing: MmcTiming::Legacy,
            bus_width: MmcBusWidth::Width1,
        };

        // Continue working on it next week
        Ok(Sdcard {
            card_id,
            manufacture_info: cid,
            card_specific_data: csd,
            card_version,
            relative_card_addr: rca,
            card_state,
            card_config: None,
        })
    }

    pub fn enable_interrupt(&mut self, irq_to_enable: &mut u32) -> Result<(), SdmmcHalError> {
        let res = self.hardware.sdmmc_enable_interrupt(irq_to_enable);
        self.host_info.enabled_irq = *irq_to_enable;
        res
    }

    pub fn ack_interrupt(&mut self) -> Result<(), SdmmcHalError> {
        self.hardware
            .sdmmc_ack_interrupt(&self.host_info.enabled_irq)
    }

    pub async fn read_block(
        self,
        blockcnt: u32,
        start_idx: u64,
        destination: u64,
    ) -> (Result<(), SdmmcHalError>, Option<SdmmcProtocol<'a, T>>) {
        let mut cmd: SdmmcCmd;
        let mut res: Result<(), SdmmcHalError>;
        // TODO: Figure out a way to support cards with 4 KB sector size
        let data: MmcData = MmcData {
            blocksize: 512,
            blockcnt,
            flags: MmcDataFlag::SdmmcDataRead,
            addr: destination,
        };
        let mut resp: [u32; 4] = [0; 4];
        // TODO: Add more validation check in the future
        // Like sdmmc card usually cannot transfer arbitrary number of blocks at once

        // The cmd arg for read operation is different between some card variation as showed by uboot code below
        /*
            if (mmc->high_capacity)
                cmd.cmdarg = start;
            else
                cmd.cmdarg = start * mmc->read_bl_len;
        */
        // For now we default to assume the card is high_capacity
        // TODO: Fix it when we properly implement card boot up
        // TODO: If we boot the card by ourself or reset the card, remember to send block len cmd
        let cmd_arg: u64 = start_idx;
        if blockcnt == 1 {
            cmd = SdmmcCmd {
                cmdidx: MMC_CMD_READ_SINGLE_BLOCK,
                resp_type: MMC_RSP_R1,
                cmdarg: cmd_arg as u32,
            };
            let future = SdmmcCmdFuture::new(self.hardware, &cmd, Some(&data), &mut resp);
            res = future.await;

            // TODO: Figure out a generic model for every sd controller, like what if the sd controller only send interrupt
            // for read/write requests?
            let _ = self
                .hardware
                .sdmmc_ack_interrupt(&self.host_info.enabled_irq);

            return (res, Some(self));
        } else {
            cmd = SdmmcCmd {
                cmdidx: MMC_CMD_READ_MULTIPLE_BLOCK,
                resp_type: MMC_RSP_R1,
                cmdarg: cmd_arg as u32,
            };
            let future = SdmmcCmdFuture::new(self.hardware, &cmd, Some(&data), &mut resp);
            res = future.await;

            // TODO: Figure out a generic model for every sd controller, like what if the sd controller only send interrupt
            // for read/write requests?
            let _ = self
                .hardware
                .sdmmc_ack_interrupt(&self.host_info.enabled_irq);

            if let Ok(()) = res {
                // Uboot code for determine response type in this case
                // cmd.resp_type = (IS_SD(mmc) || write) ? MMC_RSP_R1b : MMC_RSP_R1;
                // TODO: Add mmc checks here
                cmd = SdmmcCmd {
                    cmdidx: MMC_CMD_STOP_TRANSMISSION,
                    resp_type: MMC_RSP_R1B,
                    cmdarg: 0,
                };
                let future = SdmmcCmdFuture::new(self.hardware, &cmd, None, &mut resp);
                res = future.await;

                // TODO: Figure out a generic model for every sd controller, like what if the sd controller only send interrupt
                // for read/write requests?
                let _ = self
                    .hardware
                    .sdmmc_ack_interrupt(&self.host_info.enabled_irq);

                return (res.map_err(|_| SdmmcHalError::ESTOPCMD), Some(self));
            } else {
                return (res, Some(self));
            }
        }
    }

    // Almost the same with read_block aside from the cmd being sent is a bit different
    // For any future code add to read_block/write_block, remember to change both
    // Should read_block/write_block be the same function?
    pub async fn write_block(
        self,
        blockcnt: u32,
        start_idx: u64,
        source: u64,
    ) -> (Result<(), SdmmcHalError>, Option<SdmmcProtocol<'a, T>>) {
        let mut cmd: SdmmcCmd;
        let mut res: Result<(), SdmmcHalError>;
        // TODO: Figure out a way to support cards with 4 KB sector size
        let data: MmcData = MmcData {
            blocksize: 512,
            blockcnt,
            flags: MmcDataFlag::SdmmcDataWrite,
            addr: source,
        };
        let mut resp: [u32; 4] = [0; 4];
        // TODO: Add more validation check in the future

        let cmd_arg: u64 = start_idx;
        if blockcnt == 1 {
            cmd = SdmmcCmd {
                cmdidx: MMC_CMD_WRITE_SINGLE_BLOCK,
                resp_type: MMC_RSP_R1,
                cmdarg: cmd_arg as u32,
            };
            let future = SdmmcCmdFuture::new(self.hardware, &cmd, Some(&data), &mut resp);
            res = future.await;

            // TODO: Figure out a generic model for every sd controller, like what if the sd controller only send interrupt
            // for read/write requests?
            let _ = self
                .hardware
                .sdmmc_ack_interrupt(&self.host_info.enabled_irq);

            return (res, Some(self));
        } else {
            cmd = SdmmcCmd {
                cmdidx: MMC_CMD_WRITE_MULTIPLE_BLOCK,
                resp_type: MMC_RSP_R1,
                cmdarg: cmd_arg as u32,
            };
            let future = SdmmcCmdFuture::new(self.hardware, &cmd, Some(&data), &mut resp);
            res = future.await;

            // TODO: Figure out a generic model for every sd controller, like what if the sd controller only send interrupt
            // for read/write requests?
            let _ = self
                .hardware
                .sdmmc_ack_interrupt(&self.host_info.enabled_irq);

            if let Ok(()) = res {
                // Uboot code for determine response type in this case
                // cmd.resp_type = (IS_SD(mmc) || write) ? MMC_RSP_R1b : MMC_RSP_R1;
                // TODO: Add mmc checks here
                cmd = SdmmcCmd {
                    cmdidx: MMC_CMD_STOP_TRANSMISSION,
                    resp_type: MMC_RSP_R1B,
                    cmdarg: 0,
                };
                let future = SdmmcCmdFuture::new(self.hardware, &cmd, None, &mut resp);
                res = future.await;

                // TODO: Figure out a generic model for every sd controller, like what if the sd controller only send interrupt
                // for read/write requests?
                let _ = self
                    .hardware
                    .sdmmc_ack_interrupt(&self.host_info.enabled_irq);

                return (res.map_err(|_| SdmmcHalError::ESTOPCMD), Some(self));
            } else {
                return (res, Some(self));
            }
        }
    }

    // Not used right now, but would be useful in the future once we want to execute some command synchronously
    fn send_cmd_and_receive_resp(
        hardware: &mut T,
        cmd: &SdmmcCmd,
        data: Option<&MmcData>,
        resp: &mut [u32; 4],
    ) -> Result<(), SdmmcHalError> {
        // TODO: Add temporarily disable interrupt here

        // Send the command using the hardware layer
        let mut res = hardware.sdmmc_send_command(cmd, data);
        if res.is_err() {
            return res;
        }

        // TODO: Change it to use the sleep function provided by the hardware layer
        // This is a busy poll retry, we could poll infinitely if we trust the device to be correct
        let mut retry: u32 = 1000000;

        while retry > 0 {
            // Try to receive the response
            res = hardware.sdmmc_receive_response(cmd, resp);

            if let Err(SdmmcHalError::EBUSY) = res {
                // Busy response, retry
                retry -= 1;
                // hardware.sleep(1); // Placeholder: Implement a sleep function in SdmmcHardware trait
            } else {
                // If any other error or success, break the loop
                break;
            }
        }

        // TODO: Add renable interrupt here
        res // Return the final result (Ok or Err)
    }
}

enum CmdState {
    // Currently sending the command
    // State at the start
    NotSent,
    // Waiting for the response
    WaitingForResponse,
    // Error encountered
    Error,
    // Finished
    Finished,
}

pub struct SdmmcCmdFuture<'a, 'b, 'c> {
    hardware: &'a mut dyn SdmmcHardware,
    cmd: &'b SdmmcCmd,
    data: Option<&'b MmcData>,
    waker: Option<Waker>,
    state: CmdState,
    response: &'c mut [u32; 4],
}

impl<'a, 'b, 'c> SdmmcCmdFuture<'a, 'b, 'c> {
    pub fn new(
        hardware: &'a mut dyn SdmmcHardware,
        cmd: &'b SdmmcCmd,
        data: Option<&'b MmcData>,
        response: &'c mut [u32; 4],
    ) -> SdmmcCmdFuture<'a, 'b, 'c> {
        SdmmcCmdFuture {
            hardware,
            cmd,
            data,
            waker: None,
            state: CmdState::NotSent,
            response,
        }
    }
}

/// SdmmcCmdFuture serves as the basic building block for async fn above
/// In the context of Sdmmc device, since the requests are executed linearly under the hood
/// We actually do not need an executor to execute the request
/// The context can be ignored unless someone insist to use an executor for the requests
/// So for now, the context is being stored in waker but this waker will not be used
/// Beside, we are in no std environment and can barely use any locks
impl<'a, 'b, 'c> Future for SdmmcCmdFuture<'a, 'b, 'c> {
    type Output = Result<(), SdmmcHalError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // As I have said above, this waker is not being used so it do not need to be shared data
        // But store the waker provided anyway
        // Beware you need to update waker every polling, for more details about why
        // read Asynchronous Programming in Rust
        self.waker = Some(cx.waker().clone());

        match self.state {
            CmdState::NotSent => {
                let res: Result<(), SdmmcHalError>;
                {
                    let this: &mut SdmmcCmdFuture<'a, 'b, 'c> = self.as_mut().get_mut();

                    // Now, you can pass `&SdmmcCmd` to `sdmmc_send_command`
                    let cmd: &SdmmcCmd = this.cmd;
                    let data: Option<&MmcData> = this.data;
                    res = this.hardware.sdmmc_send_command(cmd, data);
                }
                if let Ok(()) = res {
                    self.state = CmdState::WaitingForResponse;
                    return Poll::Pending;
                } else {
                    self.state = CmdState::Error;
                    return Poll::Ready(res);
                }
            }
            CmdState::WaitingForResponse => {
                let res;
                {
                    let this: &mut SdmmcCmdFuture<'a, 'b, 'c> = self.as_mut().get_mut();
                    let cmd: &SdmmcCmd = this.cmd;
                    let response: &mut [u32; 4] = this.response;
                    let hardware: &mut dyn SdmmcHardware = this.hardware;
                    res = hardware.sdmmc_receive_response(cmd, response);
                }
                if let Err(SdmmcHalError::EBUSY) = res {
                    return Poll::Pending;
                } else if let Ok(()) = res {
                    self.state = CmdState::Finished;
                    return Poll::Ready(res);
                } else {
                    self.state = CmdState::Error;
                    return Poll::Ready(res);
                }
            }
            CmdState::Error => return Poll::Ready(Err(SdmmcHalError::EUNDEFINED)),
            CmdState::Finished => return Poll::Ready(Err(SdmmcHalError::EUNDEFINED)),
        }
    }
}
