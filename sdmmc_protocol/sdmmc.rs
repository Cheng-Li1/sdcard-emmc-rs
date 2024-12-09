use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use core::fmt::Write; // For write! macro with a buffer

use bitflags::Flags;
use mmc_struct::{MmcBusWidth, MmcDevice, MmcState, MmcTiming, TuningState};
use sdcard::{Cid, Csd, Sdcard};
use sdmmc_capability::{
    SdcardCapability, SdmmcHostCapability, MMC_CAP_4_BIT_DATA, MMC_CAP_VOLTAGE_TUNE, MMC_EMPTY_CAP,
    MMC_TIMING_LEGACY, MMC_TIMING_SD_HS, MMC_TIMING_UHS_DDR50, MMC_TIMING_UHS_SDR104,
    MMC_TIMING_UHS_SDR12, MMC_TIMING_UHS_SDR25, MMC_TIMING_UHS_SDR50,
};
use sdmmc_constant::{
    MMC_CMD_ALL_SEND_CID, MMC_CMD_APP_CMD, MMC_CMD_GO_IDLE_STATE, MMC_CMD_READ_MULTIPLE_BLOCK, MMC_CMD_READ_SINGLE_BLOCK, MMC_CMD_SELECT_CARD, MMC_CMD_SEND_CSD, MMC_CMD_STOP_TRANSMISSION, MMC_CMD_WRITE_MULTIPLE_BLOCK, MMC_CMD_WRITE_SINGLE_BLOCK, MMC_VDD_31_32, MMC_VDD_32_33, MMC_VDD_33_34, OCR_BUSY, OCR_HCS, OCR_S18R, SD_CMD_APP_SEND_OP_COND, SD_CMD_APP_SET_BUS_WIDTH, SD_CMD_SEND_IF_COND, SD_CMD_SEND_RELATIVE_ADDR, SD_CMD_SWITCH_FUNC, SD_CMD_SWITCH_UHS18V, SD_SWITCH_FUNCTION_GROUP_ONE, SD_SWITCH_FUNCTION_GROUP_ONE_CHECK_LEGACY, SD_SWITCH_FUNCTION_GROUP_ONE_CHECK_SDHS, SD_SWITCH_FUNCTION_GROUP_ONE_CHECK_UHS_DDR50, SD_SWITCH_FUNCTION_GROUP_ONE_CHECK_UHS_SDR104, SD_SWITCH_FUNCTION_GROUP_ONE_CHECK_UHS_SDR12, SD_SWITCH_FUNCTION_GROUP_ONE_CHECK_UHS_SDR25, SD_SWITCH_FUNCTION_GROUP_ONE_CHECK_UHS_SDR50, SD_SWITCH_FUNCTION_GROUP_ONE_SET_LEGACY, SD_SWITCH_FUNCTION_GROUP_ONE_SET_SDHS, SD_SWITCH_FUNCTION_GROUP_ONE_SET_UHS_DDR50, SD_SWITCH_FUNCTION_GROUP_ONE_SET_UHS_SDR104, SD_SWITCH_FUNCTION_GROUP_ONE_SET_UHS_SDR12, SD_SWITCH_FUNCTION_GROUP_ONE_SET_UHS_SDR25, SD_SWITCH_FUNCTION_GROUP_ONE_SET_UHS_SDR50, SD_SWITCH_FUNCTION_SELECTION_GROUP_ONE
};
use sel4_microkit::{debug_print, debug_println};

pub mod mmc_struct;
mod sdcard;
pub mod sdmmc_capability;
mod sdmmc_constant;

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

#[derive(Debug)]
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
    ENOCARD,
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

    pub enabled_irq: u32,

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
    pub max_frequency: u64,
    pub min_frequency: u64,
    pub max_block_per_req: u32,
    pub irq_capability: u32,
}

/// Program async Rust can be very dangerous if you do not know what is happening understand the hood
/// Power up and power off cannot be properly implemented if I do not have access to control gpio/ regulator and timer
pub trait SdmmcHardware {
    fn sdmmc_set_power(&mut self, power_mode: MmcPowerMode) -> Result<MmcPowerMode, SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }

    fn sdmmc_init(&mut self) -> Result<(MmcIos, HostInfo, u128), SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }

    /// Change the clock, return the value or do not change it at all
    /// If the freq is set to zero, this function should try to stop the clock completely
    /// Beware at higher frequency, you may need to play with delay, adjust and clock phase
    /// to ensure that the clock edges (sampling points) occur just in time for the valid data window.
    fn sdmmc_config_timing(&mut self, timing: MmcTiming) -> Result<u64, SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }

    fn sdmmc_config_bus_width(&mut self, bus_width: MmcBusWidth) -> Result<(), SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }

    fn sdmmc_set_signal_voltage(&mut self, voltage: MmcSignalVoltage) -> Result<(), SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }

    /// Reads the current state of the SD card data lanes.
    ///
    /// This function is specifically used during voltage switching to check if the SD card
    /// acknowledges the switch by setting the data signal to low or high.
    ///
    /// Returns:
    /// - `u8`: The current state of the data lanes, where each bit represents a data line:
    ///   - `DAT0` corresponds to the least significant bit (LSB).
    ///   - `DAT7` corresponds to the most significant bit (MSB).
    ///
    /// Note:
    /// - This function is not yet implemented and currently returns an `ENOTIMPLEMENTED` error.
    fn sdmmc_read_datalanes(&self) -> Result<u8, SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }

    /// Sends a command to the SD/MMC card, ensuring that busy signal handling is managed appropriately.
    ///
    /// ### Busy Signal Handling
    /// The hardware layer is responsible for delaying the actual sending of the command if the card is busy.
    /// For example, when the protocol layer sends a command expecting an R1B response (which indicates a busy state),
    /// and immediately sends another command afterward, the hardware layer must ensure that the new command is sent
    /// only after the busy signal from the card has cleared.
    ///
    /// ### Hardware Busy Signal Detection
    /// Many modern host controllers support automatic busy signal detection, in which case the hardware layer
    /// does not need to implement any additional checks or delays—the controller will wait internally until
    /// the busy state is cleared before completing the command.
    ///
    /// ### Manual Busy Waiting
    /// If the host controller does not support hardware busy signal detection, the hardware layer must
    /// implement this behavior manually by monitoring the card's busy state and delaying the next command
    /// until the card is ready. This approach aligns with Linux’s handling of busy signals in its MMC/SD subsystem.
    ///
    /// ### Parameters
    /// * `cmd` - The SD/MMC command to send.
    /// * `data` - Optional data associated with the command, if applicable.
    ///
    /// ### Returns
    /// * `Ok(())` on success.
    /// * `Err(SdmmcHalError::ENOTIMPLEMENTED)` if the function is not implemented.
    ///
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

    /// At higher clock frequencies, timing mismatches can occur between the host's sampling point and the valid data window
    /// from the SD card during read operations. This can lead to CRC errors, as the host may sample incoming data outside the
    /// stable data window, even when the SD card’s response appears normal.
    ///
    /// To address this, the `sdmmc_tune_sampling` function is introduced. This function aims to adjust the host's sampling
    /// timing to align with the SD card’s data output window, reducing errors caused by timing misalignment.
    ///
    /// In some cases, a similar function (e.g., `sdmmc_tune_sending_data_window`) may be needed to tune the timing of data
    /// signals sent from the host to the SD card. This would ensure that the SD card reliably receives data, especially
    /// at high frequencies. However, output timing tends to be more stable, and a specific function for tuning host-to-card
    /// data timing is often not implemented or needed, as seen in the Linux driver.
    ///
    /// The cooperation between protocol layer and hardware layer by this function is like: the protocol layer send
    /// the MMC_CMD_SEND_TUNING_BLOCK request. And if the result receive back is in error, the protocol layer will call this
    /// tune_sampling function once again. If tune sampling has run out of option, return an error.
    /// It is suggest that hardware layer also do some book keeping about the suitable delay to making the tuning
    /// sampling process faster.
    /// Performs sampling point tuning for the SD/MMC interface.
    ///
    /// The `TuningState` enum controls the tuning process, guiding the adjustments
    /// of the sampling point to achieve reliable communication at high speeds.
    ///
    /// # Tuning Process
    ///
    /// - Before tuning begins, the protocol layer calls `sdmmc_tune_sampling`
    ///   with the `TuningStart` state to initialize the tuning process.
    /// - The protocol layer then attempts to verify if the current sampling adjustment is effective.
    /// - If verification fails, the protocol layer calls `sdmmc_tune_sampling`
    ///   with the `TuningContinue` state to adjust the sampling point further.
    /// - This adjustment and verification process repeats until a reliable sampling
    ///   point is found.
    /// - Once a working delay is identified, the protocol layer calls
    ///   `sdmmc_tune_sampling` with the `TuningComplete` state to finalize the tuning process.
    ///
    /// # Parameters
    /// - `state`: The current tuning state, represented by the `TuningState` enum:
    ///   - `TuningStart`: Initializes the tuning process.
    ///   - `TuningContinue`: Adjusts the sampling point incrementally.
    ///   - `TuningComplete`: Finalizes the tuning once a reliable sampling point is found.
    ///
    /// # Returns
    /// - `Ok(())`: Indicates successful completion of the requested tuning step.
    /// - `Err(SdmmcHalError)`: An error occurred during the tuning process.
    fn sdmmc_tune_sampling(&mut self, state: TuningState) -> Result<(), SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }
}

#[cfg(feature = "sel4-microkit")] 
use sel4_microkit_support::program_wait_unreliable;

#[cfg(not(feature = "sel4-microkit"))]
fn program_wait_unreliable(time_ns: u64) {
    for _ in 0..time_ns {
        hint::spin_loop(); // Use spin loop hint to reduce contention during the wait
    }
}

/// TODO: Add more variables for SdmmcProtocol to track the state of the sdmmc controller and card correctly
pub struct SdmmcProtocol<T: SdmmcHardware> {
    pub hardware: T,

    mmc_ios: MmcIos,

    host_info: HostInfo,

    host_capability: SdmmcHostCapability,

    /// This mmc device is optional because there may not always be a card in the slot!
    mmc_device: Option<MmcDevice>,
}

impl<T> Unpin for SdmmcProtocol<T> where T: Unpin + SdmmcHardware {}

impl<T: SdmmcHardware> SdmmcProtocol<T> {
    pub fn new(mut hardware: T) -> Result<Self, SdmmcHalError> {
        let (ios, info, cap) = hardware.sdmmc_init()?;

        Ok(SdmmcProtocol {
            hardware,
            mmc_ios: ios,
            host_info: info,
            host_capability: SdmmcHostCapability(cap),
            mmc_device: None,
        })
    }

    // Funtion that is not completed
    pub fn setup_card(&mut self) -> Result<(), SdmmcHalError> {
        {
            let mut irq: u32 = 0;
            self.hardware.sdmmc_enable_interrupt(&mut irq)?;
            self.mmc_ios.enabled_irq = irq;
        }

        // TODO: Different sdcard and eMMC support different voltages, figure those out
        if self.mmc_ios.vdd != 330 {
            self.hardware.sdmmc_set_power(MmcPowerMode::On)?;
            // TODO: Right now we know the power will always be up and this function should not be called
            // But when we encounter scenerio that may actually call this function, we should wait for the time specified in ios
            // Right now this whole power up related thing does not work
            self.mmc_ios.vdd = 330;
        }

        let clock = self.hardware.sdmmc_config_timing(MmcTiming::CardSetup)?;

        self.mmc_ios.clock = clock;

        // This line of code may not be needed?
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

        // TODO: A delay should be insert here, now it is using debug print to simulate the delay
        // Check the sdcard soecification to see if this delay is indeed needed
        debug_println!("Add some delay here!");

        program_wait_unreliable(10000);

        cmd = SdmmcCmd {
            cmdidx: SD_CMD_SEND_IF_COND,
            resp_type: MMC_RSP_R7,
            cmdarg: 0x000001AA, // Voltage supply and check pattern
        };

        let res = Self::send_cmd_and_receive_resp(&mut self.hardware, &cmd, None, &mut resp);

        // If the result is OK and the resp is 0x1AA, the card we are initializing is a SDHC/SDXC
        // If the result is error, it is either the voltage not being set up correctly, which mean a bug in hardware layer
        // or the card is eMMC or legacy sdcard
        // For now, we only deal with the situation it is a sdcard
        if res.is_ok() && resp[0] == 0x1AA {
            let card: Sdcard = self.setup_sdcard_cont()?;
            self.mmc_device = Some(MmcDevice::Sdcard(card));
            Ok(())
        } else {
            // TODO: Implement setup for eMMC and legacy sdcard(SDSC) here
            debug_println!("Driver right now only support SDHC/SDXC card, please check if you are running this driver on SDIO/SDSC/EMMC card!");
            Err(SdmmcHalError::EUNSUPPORTEDCARD)
        }
    }

    /// From uboot
    /// Most cards do not answer if some reserved bits
    /// in the ocr are set. However, Some controller
    /// can set bit 7 (reserved for low voltages), but
    /// how to manage low voltages SD card is not yet
    /// specified.
    fn setup_sdcard_cont(&mut self) -> Result<Sdcard, SdmmcHalError> {
        let mut cmd: SdmmcCmd;
        let mut resp: [u32; 4] = [0; 4];
        // Uboot define this value to be 1000...
        let mut timeout: u16 = 1000;

        loop {
            debug_println!("Sending SD_CMD_APP_SEND_OP_COND!");
            // Prepare CMD55 (APP_CMD)
            cmd = SdmmcCmd {
                cmdidx: MMC_CMD_APP_CMD,
                resp_type: MMC_RSP_R1,
                cmdarg: 0,
            };

            // Send CMD55
            Self::send_cmd_and_receive_resp(&mut self.hardware, &cmd, None, &mut resp)?;

            cmd = SdmmcCmd {
                cmdidx: SD_CMD_APP_SEND_OP_COND,
                resp_type: MMC_RSP_R3,
                cmdarg: 0,
            };

            // Set the HCS bit if version is SD Version 2
            // since the flow has reached here, the card would be at least SD Version 2
            cmd.cmdarg |= OCR_HCS;

            // TODO: Right now the operating voltage is hardcoded, it should be changed to be provided by the hal!
            // And maybe change this when we are trying to support UHS I
            // Change this when we decide to support spi or SDSC as well
            cmd.cmdarg |= (MMC_VDD_33_34 | MMC_VDD_32_33 | MMC_VDD_31_32) & 0xff8000;

            if self
                .host_capability
                .contains(sdmmc_capability::SdmmcHostCapability(
                    MMC_TIMING_UHS_SDR12 | MMC_CAP_VOLTAGE_TUNE,
                ))
            {
                cmd.cmdarg |= OCR_S18R;
                // cmd.cmdarg |= MMC_VDD_165_195;
            }

            // Send ACMD41
            Self::send_cmd_and_receive_resp(&mut self.hardware, &cmd, None, &mut resp)?;

            debug_println!("OCR: {:08x}", resp[0]);

            // Check if card is ready (OCR_BUSY bit)
            if (resp[0] & OCR_BUSY) != 0 {
                break;
            }

            // Timeout handling
            if timeout <= 0 {
                debug_println!("SDMMC: Setting voltage failed, card not supported!");
                return Err(SdmmcHalError::EUNSUPPORTEDCARD);
            }
            timeout -= 1;
        }

        // TODO: Add check here to return error when enocounter SDSC card
        if self
            .host_capability
            .contains(sdmmc_capability::SdmmcHostCapability(
                MMC_TIMING_UHS_SDR12 | MMC_CAP_VOLTAGE_TUNE,
            ))
        {
            // TODO: If the sdcard fail at this stage, a power circle and reinit should be performed
            self.tune_sdcard_switch_uhs18v()?;
            self.mmc_ios.signal_voltage = MmcSignalVoltage::Voltage180;
        }

        // 4. Send CMD2 to get the CID register
        cmd = SdmmcCmd {
            cmdidx: MMC_CMD_ALL_SEND_CID,
            resp_type: MMC_RSP_R2,
            cmdarg: 0,
        };
        Self::send_cmd_and_receive_resp(&mut self.hardware, &cmd, None, &mut resp)?;

        let cid: Cid = Cid::new(resp);

        let card_id = ((resp[0] as u128) << 96)
            | ((resp[1] as u128) << 64)
            | ((resp[2] as u128) << 32)
            | (resp[3] as u128);

        // TODO: Figure out a better way to do this than adding microkit crate
        sel4_microkit::debug_println!(
            "CID: {:08x} {:08x} {:08x} {:08x}",
            resp[0],
            resp[1],
            resp[2],
            resp[3]
        );

        // 5. Send CMD3 to set and receive the RCA
        cmd = SdmmcCmd {
            cmdidx: SD_CMD_SEND_RELATIVE_ADDR,
            resp_type: MMC_RSP_R6,
            cmdarg: 0,
        };

        Self::send_cmd_and_receive_resp(&mut self.hardware, &cmd, None, &mut resp)?;

        let rca: u16 = (resp[0] >> 16) as u16; // Store RCA from response

        sel4_microkit::debug_println!("RCA: {:04x}", rca);

        // 6. Send CMD9 to get the CSD register
        cmd = SdmmcCmd {
            cmdidx: MMC_CMD_SEND_CSD,
            resp_type: MMC_RSP_R2,
            cmdarg: (rca as u32) << 16,
        };
        Self::send_cmd_and_receive_resp(&mut self.hardware, &cmd, None, &mut resp)?;

        sel4_microkit::debug_println!(
            "CSD: {:08x} {:08x} {:08x} {:08x}",
            resp[0],
            resp[1],
            resp[2],
            resp[3]
        );

        let (csd, card_version) = Csd::new(resp);

        // 7. Send CMD7 to select the card
        cmd = SdmmcCmd {
            cmdidx: MMC_CMD_SELECT_CARD,
            resp_type: MMC_RSP_R1,
            cmdarg: (rca as u32) << 16,
        };
        Self::send_cmd_and_receive_resp(&mut self.hardware, &cmd, None, &mut resp)?;

        // SDHC/SDXC default to 512 bytes sector size so I did not manually set it here

        self.mmc_ios.clock = self.hardware.sdmmc_config_timing(MmcTiming::Legacy)?;

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
            card_cap: sdmmc_capability::SdcardCapability(MMC_EMPTY_CAP),
            card_config: None,
        })
    }

    /// Placeholder function for validating if the host satisfy the minimal operation condition
    fn validate_host_capibaility(host_info: HostInfo, cap: SdmmcHostCapability) {
        todo!()
    }

    /// A function that tune the card speed
    /// This function does not do roll back so if this function return an error, reset up the card
    /// Do NOT call this function again if your card is already tuned as this function is not that cheap!
    /// But you should call this function the card being turned into power saving mode
    /// `stolen_memory` is a specific memory region used to read data from the SD card through CMD6.
    ///
    /// ### Important Considerations:
    ///
    /// 1. **Memory Region Constraints**:
    ///    - `stolen_memory` is used as a buffer to hold the 64-byte response from the SD card
    ///      when executing CMD6. This response contains the function switch status and other
    ///      function-related information.
    ///    - The memory region from `stolen_memory` to `stolen_memory + 64 bytes` must not overlap
    ///      with any other data structures, device registers, or memory-mapped peripherals to
    ///      avoid conflicts or unintended behavior.
    ///
    /// 2. **Cache Considerations**:
    ///    - If the system uses caching, you may need to ensure that cache effects do not interfere
    ///      with the integrity of the data read from the SD card.
    ///    - If caching is enabled, consider using cache invalidation before reading and cache flushing
    ///      after writing data to ensure consistency between the memory and the SD card.
    ///      For example, you might need to use cache control instructions or APIs specific to your
    ///      platform to manage this.
    ///
    /// 3. **Alignment and Access Requirements**:
    ///    - `stolen_memory` should be aligned to at least 4 bytes (or preferably 8 bytes) to avoid
    ///      misaligned memory access issues, which could lead to performance penalties or even faults
    ///      on some architectures.
    /// Tunes SD card performance by adjusting data bus width and speed mode.
    ///
    /// # Parameters
    /// - `addr_and_invalidate_cache_fn`: An optional tuple containing:
    ///     - `*mut [u8; 64]`: A memory address used as a buffer for certain commands (e.g., CMD6). The memory
    ///       should be suitable for DMA into(e.g. aligned to 8 bytes memory border and not conflict
    ///       with other structure or device registers).
    ///     - `fn()`: A function pointer that, when called, invalidates the cache for the range
    ///       `addr` to `addr + 64 bytes`. This function should ensure cache consistency for
    ///       that specific memory range. If `None`, no buffer is used, and the tune performance function
    ///       will not attempt to change the card speed class. The fn should not take any variables and would
    ///       not be stored. By this way, the protocol layer only has the minimal privilege it required for cache invalidation.
    ///
    /// # Returns
    /// - `Result<(), SdmmcHalError>`: `Ok(())` if tuning was successful, or an error otherwise.
    pub fn tune_performance(
        &mut self,
        memory_and_invalidate_cache_fn: Option<(&mut [u8; 64], fn())>,
    ) -> Result<(), SdmmcHalError> {
        // For testing
        let mmc_device = self.mmc_device.as_mut().ok_or(SdmmcHalError::ENOCARD)?;

        // Turn down the clock frequency
        self.mmc_ios.clock = self.hardware.sdmmc_config_timing(MmcTiming::CardSetup)?;

        match mmc_device {
            MmcDevice::Sdcard(sdcard) => {
                sdcard.card_state.timing = MmcTiming::CardSetup;
                self.tune_sdcard_performance(memory_and_invalidate_cache_fn)
            }
            MmcDevice::EMmc(emmc) => Err(SdmmcHalError::ENOTIMPLEMENTED),
            MmcDevice::Unknown => Err(SdmmcHalError::ENOTIMPLEMENTED),
        }
    }

    /// Delete it later
    #[allow(dead_code)]
    unsafe fn print_one_block(ptr: *const u8, num: usize) {
        unsafe {
            // Iterate over the number of bytes and print each one in hexadecimal format
            for i in 0..num {
                let byte = *ptr.add(i);
                if i % 16 == 0 {
                    debug_print!("\n{:04x}: ", i);
                }
                debug_print!("{:02x} ", byte);
            }
            debug_println!();
        }
    }

    pub fn test_read_one_block(&mut self, start_idx: u64, destination: u64) {
        let data: MmcData = MmcData {
            blocksize: 512,
            blockcnt: 1,
            flags: MmcDataFlag::SdmmcDataRead,
            addr: destination,
        };
        debug_println!("Gonna test read one block!");
        let mut resp: [u32; 4] = [0; 4];
        let cmd_arg: u64 = start_idx;
        let cmd = SdmmcCmd {
            cmdidx: MMC_CMD_READ_SINGLE_BLOCK,
            resp_type: MMC_RSP_R1,
            cmdarg: cmd_arg as u32,
        };
        if let Err(error) =
            Self::send_cmd_and_receive_resp(&mut self.hardware, &cmd, Some(&data), &mut resp)
        {
            debug_println!("Error in reading");
        }
        unsafe { Self::print_one_block(destination as *mut u8, 512) };
    }

    fn test_sampling(
        &mut self,
        memory_addr: u64,
        cache_invalidate_function: fn(),
    ) -> Result<(), SdmmcHalError> {
        let mut resp: [u32; 4] = [0; 4];

        let data = MmcData {
            blocksize: 64,
            blockcnt: 1,
            flags: MmcDataFlag::SdmmcDataRead,
            addr: memory_addr,
        };

        let cmd = SdmmcCmd {
            cmdidx: SD_CMD_SWITCH_FUNC,
            resp_type: MMC_RSP_R1,
            cmdarg: 0x00FFFFFF,
        };

        Self::send_cmd_and_receive_resp(&mut self.hardware, &cmd, Some(&data), &mut resp)
        // TODO: Add validate the content of data returned here, but it is optional because
        // if there is not error it must has passed the CRC check!
    }

    fn process_sampling(
        &mut self,
        memory_addr: u64,
        cache_invalidate_function: fn(),
    ) -> Result<(), SdmmcHalError> {
        self.hardware
            .sdmmc_tune_sampling(TuningState::TuningStart)?;
        loop {
            // Call test_sampling
            match self.test_sampling(memory_addr, cache_invalidate_function) {
                Ok(_) => {
                    self.hardware
                        .sdmmc_tune_sampling(TuningState::TuningComplete)?;
                    // If test_sampling succeeds, return Ok
                    return Ok(());
                }
                Err(SdmmcHalError::EIO) => {
                    // If test_sampling fails with EIO, try tuning the sampling point
                    self.hardware
                        .sdmmc_tune_sampling(TuningState::TuningContinue)?;
                }
                Err(e) => {
                    // If test_sampling fails with an error other than EIO, return that error
                    return Err(e);
                }
            }
        }
    }

    /// Switch voltage function
    fn tune_sdcard_switch_uhs18v(&mut self) -> Result<(), SdmmcHalError> {
        sel4_microkit::debug_println!("Entering tune sdcard signal voltage function!");
        let mut resp: [u32; 4] = [0; 4];
        let cmd = SdmmcCmd {
            cmdidx: SD_CMD_SWITCH_UHS18V,
            resp_type: MMC_RSP_R1,
            cmdarg: 0, // Argument for 4-bit mode (0 for 1-bit mode)
        };
        Self::send_cmd_and_receive_resp(&mut self.hardware, &cmd, None, &mut resp)?;
        sel4_microkit::debug_println!("Switch voltage prepared!");

        self.mmc_ios.clock = self.hardware.sdmmc_config_timing(MmcTiming::ClockStop)?;

        /* 
        for _ in 0..10 {
            debug_println!("Wait for clock to stable!");
        }
        */

        program_wait_unreliable(10000);

        let mut signal: u8 = 0xFF;

        for _ in 0..100 {
            signal = self.hardware.sdmmc_read_datalanes()?;
            program_wait_unreliable(1000);
            sel4_microkit::debug_println!("data signal value: 0b{:b}", signal);
            if signal & 0xF == 0x0 {
                break;
            }
        }

        if signal & 0xF != 0x0 {
            return Err(SdmmcHalError::EUNSUPPORTEDCARD);
        }

        self.hardware
            .sdmmc_set_signal_voltage(MmcSignalVoltage::Voltage180)?;

        /* for _ in 0..100 {
            debug_println!("Print to wait until voltage is ready!");
        }*/

        program_wait_unreliable(10000000);

        self.mmc_ios.clock = self.hardware.sdmmc_config_timing(MmcTiming::CardSetup)?;

        /* for _ in 0..10 {
            debug_println!("Wait for clock to stable!");
        }*/

        program_wait_unreliable(10000);

        for _ in 0..100 {
            signal = self.hardware.sdmmc_read_datalanes()?;
            program_wait_unreliable(1000);
            sel4_microkit::debug_println!("data signal value: 0b{:b}", signal);
            if signal & 0xF == 0xF {
                break;
            }
        }

        if signal & 0xF != 0xF {
            return Err(SdmmcHalError::EUNSUPPORTEDCARD);
        }

        Ok(())
    }

    /// Right now the speed switch is done hackily
    /// The speed switch is hardcoded to switch to UHS SDR104
    /// If the card does not support UHS SDR104, the switch will fail!
    /// Implement this sdcard switch function to avoid this hackiness!
    /// Like first get the speed classes the sdcard support by this function
    /// and then switch to the proper speed class!
    fn sdcard_switch_speed(
        &mut self,
        target: MmcTiming,
        memory: &mut [u8; 64],
        invalidate_cache_fn: fn(),
    ) -> Result<(), SdmmcHalError> {
        let data: MmcData = MmcData {
            blocksize: 64,
            blockcnt: 1,
            flags: MmcDataFlag::SdmmcDataRead,
            addr: memory.as_ptr() as u64,
        };

        let mut resp: [u32; 4] = [0; 4];

        // As this function only change speed class,
        let mut cmdarg: u32 = 0x80FFFFF0;
        match target {
            MmcTiming::Legacy => cmdarg |= SD_SWITCH_FUNCTION_GROUP_ONE_SET_LEGACY as u32,
            MmcTiming::SdHs => cmdarg |= SD_SWITCH_FUNCTION_GROUP_ONE_SET_SDHS as u32,
            MmcTiming::UhsSdr12 => cmdarg |= SD_SWITCH_FUNCTION_GROUP_ONE_SET_UHS_SDR12 as u32,
            MmcTiming::UhsSdr25 => cmdarg |= SD_SWITCH_FUNCTION_GROUP_ONE_SET_UHS_SDR25 as u32,
            MmcTiming::UhsSdr50 => cmdarg |= SD_SWITCH_FUNCTION_GROUP_ONE_SET_UHS_SDR50 as u32,
            MmcTiming::UhsSdr104 => cmdarg |= SD_SWITCH_FUNCTION_GROUP_ONE_SET_UHS_SDR104 as u32,
            MmcTiming::UhsDdr50 => cmdarg |= SD_SWITCH_FUNCTION_GROUP_ONE_SET_UHS_DDR50 as u32,
            _ => return Err(SdmmcHalError::EUNDEFINED),
        }
        let cmd = SdmmcCmd {
            cmdidx: SD_CMD_SWITCH_FUNC,
            resp_type: MMC_RSP_R1,
            cmdarg,
        };
        Self::send_cmd_and_receive_resp(&mut self.hardware, &cmd, Some(&data), &mut resp)?;
        // Error handling here
        invalidate_cache_fn();
        // Since we are using u8 here, the endianness does matter
        // 0xF indicate function switch error
        // TODO: Double check here and deliberately trigger swithc error to see if the field
        // is being set to 0xF
        if memory[SD_SWITCH_FUNCTION_SELECTION_GROUP_ONE] & 0xF == 0xF {
            return Err(SdmmcHalError::EINVAL);
        }
        Ok(())
    }

    fn sdcard_check_supported_speed_class(
        &mut self,
        memory: &mut [u8; 64],
        invalidate_cache_fn: fn(),
    ) -> Result<(), SdmmcHalError> {
        let data: MmcData = MmcData {
            blocksize: 64,
            blockcnt: 1,
            flags: MmcDataFlag::SdmmcDataRead,
            addr: memory.as_ptr() as u64,
        };

        let mut resp: [u32; 4] = [0; 4];

        let mut card_cap: SdcardCapability = sdmmc_capability::SdcardCapability(MMC_EMPTY_CAP);
        let cmd = SdmmcCmd {
            cmdidx: SD_CMD_SWITCH_FUNC,
            resp_type: MMC_RSP_R1,
            cmdarg: 0x00FFFFFF,
        };
        Self::send_cmd_and_receive_resp(&mut self.hardware, &cmd, Some(&data), &mut resp)?;
        invalidate_cache_fn();
        // Typical speed class supported: 80 03
        match self.mmc_ios.signal_voltage {
            MmcSignalVoltage::Voltage330 => {
                let speed_class_byte: u8 = memory[SD_SWITCH_FUNCTION_GROUP_ONE];
                if speed_class_byte & SD_SWITCH_FUNCTION_GROUP_ONE_CHECK_SDHS != 0 {
                    card_cap.insert(sdmmc_capability::SdcardCapability(MMC_TIMING_SD_HS));
                }
            }
            MmcSignalVoltage::Voltage180 => {
                let speed_class_byte: u8 = memory[SD_SWITCH_FUNCTION_GROUP_ONE];
                if speed_class_byte & SD_SWITCH_FUNCTION_GROUP_ONE_CHECK_UHS_SDR12 != 0 {
                    card_cap.insert(sdmmc_capability::SdcardCapability(MMC_TIMING_UHS_SDR12));
                }
                if speed_class_byte & SD_SWITCH_FUNCTION_GROUP_ONE_CHECK_UHS_SDR25 != 0 {
                    card_cap.insert(sdmmc_capability::SdcardCapability(MMC_TIMING_UHS_SDR25));
                }
                if speed_class_byte & SD_SWITCH_FUNCTION_GROUP_ONE_CHECK_UHS_SDR50 != 0 {
                    card_cap.insert(sdmmc_capability::SdcardCapability(MMC_TIMING_UHS_SDR50));
                }
                if speed_class_byte & SD_SWITCH_FUNCTION_GROUP_ONE_CHECK_UHS_SDR104 != 0 {
                    card_cap.insert(sdmmc_capability::SdcardCapability(MMC_TIMING_UHS_SDR104));
                }
                if speed_class_byte & SD_SWITCH_FUNCTION_GROUP_ONE_CHECK_UHS_DDR50 != 0 {
                    card_cap.insert(sdmmc_capability::SdcardCapability(MMC_TIMING_UHS_DDR50));
                }
            }
            // For sdcard, the signal voltage cannot be 1.2V
            MmcSignalVoltage::Voltage120 => return Err(SdmmcHalError::EUNDEFINED),
        }
        // Add the card cap to self
        if let Some(MmcDevice::Sdcard(ref mut sdcard)) = &mut self.mmc_device {
            sdcard.card_cap |= card_cap;
        } else {
            return Err(SdmmcHalError::EINVAL);
        }
        Ok(())
    }

    /// Since we are using u8 here, the endianness does matter
    fn tune_sdcard_performance(
        &mut self,
        memory_and_invalidate_cache_fn: Option<(&mut [u8; 64], fn())>,
    ) -> Result<(), SdmmcHalError> {
        let mut resp: [u32; 4] = [0; 4];

        if self.mmc_ios.bus_width == MmcBusWidth::Width1
            && self
                .host_capability
                .contains(SdmmcHostCapability(MMC_CAP_4_BIT_DATA))
        {
            // Switch data bits per transfer
            let relative_card_address: u16;
            if let Some(MmcDevice::Sdcard(ref sdcard)) = self.mmc_device {
                relative_card_address = sdcard.relative_card_addr;
            } else {
                return Err(SdmmcHalError::EINVAL);
            }
            // CMD55 + ACMD6 to set the card to 4-bit mode (if supported by host and card)
            let cmd = SdmmcCmd {
                cmdidx: MMC_CMD_APP_CMD,
                resp_type: MMC_RSP_R1,
                cmdarg: (relative_card_address as u32) << 16,
            };
            Self::send_cmd_and_receive_resp(&mut self.hardware, &cmd, None, &mut resp)?;

            let cmd = SdmmcCmd {
                cmdidx: SD_CMD_APP_SET_BUS_WIDTH,
                resp_type: MMC_RSP_R1,
                cmdarg: 2, // Argument for 4-bit mode (0 for 1-bit mode)
            };
            Self::send_cmd_and_receive_resp(&mut self.hardware, &cmd, None, &mut resp)?;

            self.hardware.sdmmc_config_bus_width(MmcBusWidth::Width4)?;

            sel4_microkit::debug_println!("Tuning datalanes succeed!");

            // If any of the cmd above fail, the card should be completely reinit
            self.mmc_ios.bus_width = MmcBusWidth::Width4;
            // TODO: Change sdcard bus width here, or get rid of that field completely
        }

        if let Some((memory, cache_invalidate_function)) = memory_and_invalidate_cache_fn {
            if let Some(MmcDevice::Sdcard(ref mut sdcard)) = self.mmc_device {
                match self.mmc_ios.signal_voltage {
                    MmcSignalVoltage::Voltage330 => {
                        sdcard.card_state.timing = MmcTiming::Legacy;
                        if !sdcard
                            .card_cap
                            .contains(SdcardCapability(MMC_TIMING_LEGACY))
                        {
                            // When the target speed is None, the sdcard_switch_speed update sdcard speed capability
                            self.sdcard_check_supported_speed_class(
                                memory,
                                cache_invalidate_function,
                            )?;
                        }
                    }
                    MmcSignalVoltage::Voltage180 => {
                        sdcard.card_state.timing = MmcTiming::UhsSdr12;
                        if !sdcard
                            .card_cap
                            .contains(SdcardCapability(MMC_TIMING_UHS_SDR12))
                        {
                            self.sdcard_check_supported_speed_class(
                                memory,
                                cache_invalidate_function,
                            )?;
                        }
                    }
                    MmcSignalVoltage::Voltage120 => return Err(SdmmcHalError::EUNDEFINED),
                };
            } else {
                return Err(SdmmcHalError::EUNDEFINED);
            }

            let mut timing;
            let sdcard_cap: SdcardCapability;
            if let Some(MmcDevice::Sdcard(ref sdcard)) = self.mmc_device {
                sdcard_cap = sdcard.card_cap.clone();
                timing = sdcard.card_state.timing;
                'tune_speed: {
                    match self.mmc_ios.signal_voltage {
                        MmcSignalVoltage::Voltage330 => {
                            if sdcard_cap.contains(SdcardCapability(MMC_TIMING_SD_HS))
                                && self
                                    .host_capability
                                    .contains(SdmmcHostCapability(MMC_TIMING_SD_HS))
                            {
                                // When the target speed is Some, the sdcard_switch_speed try to switch the speed class to target
                                self.sdcard_switch_speed(
                                    MmcTiming::SdHs,
                                    memory,
                                    cache_invalidate_function,
                                )?;
                                timing = MmcTiming::SdHs;
                            }
                        }
                        MmcSignalVoltage::Voltage180 => {
                            if sdcard_cap.contains(SdcardCapability(MMC_TIMING_UHS_SDR104))
                                && self
                                    .host_capability
                                    .contains(SdmmcHostCapability(MMC_TIMING_UHS_SDR104))
                            {
                                self.sdcard_switch_speed(
                                    MmcTiming::UhsSdr104,
                                    memory,
                                    cache_invalidate_function,
                                )?;
                                timing = MmcTiming::UhsSdr104;
                                // I want to break here!
                                break 'tune_speed;
                            }
                            if !sdcard_cap.contains(SdcardCapability(MMC_TIMING_UHS_DDR50))
                                && self
                                    .host_capability
                                    .contains(SdmmcHostCapability(MMC_TIMING_UHS_DDR50))
                            {
                                self.sdcard_switch_speed(
                                    MmcTiming::UhsDdr50,
                                    memory,
                                    cache_invalidate_function,
                                )?;
                                timing = MmcTiming::UhsDdr50;
                                break 'tune_speed;
                            }
                            if !sdcard_cap.contains(SdcardCapability(MMC_TIMING_UHS_SDR50))
                                && self
                                    .host_capability
                                    .contains(SdmmcHostCapability(MMC_TIMING_UHS_SDR50))
                            {
                                self.sdcard_switch_speed(
                                    MmcTiming::UhsSdr50,
                                    memory,
                                    cache_invalidate_function,
                                )?;
                                timing = MmcTiming::UhsSdr50;
                                break 'tune_speed;
                            }
                            if !sdcard_cap.contains(SdcardCapability(MMC_TIMING_UHS_SDR25))
                                && self
                                    .host_capability
                                    .contains(SdmmcHostCapability(MMC_TIMING_UHS_SDR25))
                            {
                                self.sdcard_switch_speed(
                                    MmcTiming::UhsSdr25,
                                    memory,
                                    cache_invalidate_function,
                                )?;
                                timing = MmcTiming::UhsSdr25;
                                break 'tune_speed;
                            }
                        }
                        MmcSignalVoltage::Voltage120 => return Err(SdmmcHalError::EUNDEFINED),
                    }
                };
                self.mmc_ios.clock = self.hardware.sdmmc_config_timing(timing)?;
                self.process_sampling(memory.as_ptr() as u64, cache_invalidate_function)?;

                debug_println!("Current frequency: {}Hz", self.mmc_ios.clock);
            } else {
                return Err(SdmmcHalError::EUNDEFINED);
            }
            if let Some(MmcDevice::Sdcard(ref mut sdcard)) = self.mmc_device {
                sdcard.card_state.timing = timing;
            }
        }

        Ok(())
    }

    pub fn enable_interrupt(&mut self, irq_to_enable: &mut u32) -> Result<(), SdmmcHalError> {
        let res = self.hardware.sdmmc_enable_interrupt(irq_to_enable);
        self.mmc_ios.enabled_irq = *irq_to_enable;
        res
    }

    pub fn ack_interrupt(&mut self) -> Result<(), SdmmcHalError> {
        self.hardware.sdmmc_ack_interrupt(&self.mmc_ios.enabled_irq)
    }

    pub async fn read_block(
        mut self,
        blockcnt: u32,
        start_idx: u64,
        destination: u64,
    ) -> (Result<(), SdmmcHalError>, SdmmcProtocol<T>) {
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
            let future = SdmmcCmdFuture::new(&mut self.hardware, &cmd, Some(&data), &mut resp);
            res = future.await;

            // TODO: Figure out a generic model for every sd controller, like what if the sd controller only send interrupt
            // for read/write requests?
            let _ = self.hardware.sdmmc_ack_interrupt(&self.mmc_ios.enabled_irq);

            return (res, self);
        } else {
            cmd = SdmmcCmd {
                cmdidx: MMC_CMD_READ_MULTIPLE_BLOCK,
                resp_type: MMC_RSP_R1,
                cmdarg: cmd_arg as u32,
            };
            let future = SdmmcCmdFuture::new(&mut self.hardware, &cmd, Some(&data), &mut resp);
            res = future.await;

            // TODO: Figure out a generic model for every sd controller, like what if the sd controller only send interrupt
            // for read/write requests?
            let _ = self.hardware.sdmmc_ack_interrupt(&self.mmc_ios.enabled_irq);

            if let Ok(()) = res {
                // Uboot code for determine response type in this case
                // cmd.resp_type = (IS_SD(mmc) || write) ? MMC_RSP_R1b : MMC_RSP_R1;
                // TODO: Add mmc checks here
                cmd = SdmmcCmd {
                    cmdidx: MMC_CMD_STOP_TRANSMISSION,
                    resp_type: MMC_RSP_R1B,
                    cmdarg: 0,
                };
                let future = SdmmcCmdFuture::new(&mut self.hardware, &cmd, None, &mut resp);
                res = future.await;

                // TODO: Figure out a generic model for every sd controller, like what if the sd controller only send interrupt
                // for read/write requests?
                let _ = self.hardware.sdmmc_ack_interrupt(&self.mmc_ios.enabled_irq);

                return (res.map_err(|_| SdmmcHalError::ESTOPCMD), self);
            } else {
                return (res, self);
            }
        }
    }

    // Almost the same with read_block aside from the cmd being sent is a bit different
    // For any future code add to read_block/write_block, remember to change both
    // Should read_block/write_block be the same function?
    pub async fn write_block(
        mut self,
        blockcnt: u32,
        start_idx: u64,
        source: u64,
    ) -> (Result<(), SdmmcHalError>, SdmmcProtocol<T>) {
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
            let future = SdmmcCmdFuture::new(&mut self.hardware, &cmd, Some(&data), &mut resp);
            res = future.await;

            // TODO: Figure out a generic model for every sd controller, like what if the sd controller only send interrupt
            // for read/write requests?
            let _ = self.hardware.sdmmc_ack_interrupt(&self.mmc_ios.enabled_irq);

            return (res, self);
        } else {
            cmd = SdmmcCmd {
                cmdidx: MMC_CMD_WRITE_MULTIPLE_BLOCK,
                resp_type: MMC_RSP_R1,
                cmdarg: cmd_arg as u32,
            };
            let future = SdmmcCmdFuture::new(&mut self.hardware, &cmd, Some(&data), &mut resp);
            res = future.await;

            // TODO: Figure out a generic model for every sd controller, like what if the sd controller only send interrupt
            // for read/write requests?
            let _ = self.hardware.sdmmc_ack_interrupt(&self.mmc_ios.enabled_irq);

            if let Ok(()) = res {
                // Uboot code for determine response type in this case
                // cmd.resp_type = (IS_SD(mmc) || write) ? MMC_RSP_R1b : MMC_RSP_R1;
                // TODO: Add mmc checks here
                cmd = SdmmcCmd {
                    cmdidx: MMC_CMD_STOP_TRANSMISSION,
                    resp_type: MMC_RSP_R1B,
                    cmdarg: 0,
                };
                let future = SdmmcCmdFuture::new(&mut self.hardware, &cmd, None, &mut resp);
                res = future.await;

                // TODO: Figure out a generic model for every sd controller, like what if the sd controller only send interrupt
                // for read/write requests?
                let _ = self.hardware.sdmmc_ack_interrupt(&self.mmc_ios.enabled_irq);

                return (res.map_err(|_| SdmmcHalError::ESTOPCMD), self);
            } else {
                return (res, self);
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
        let mut retry: u32 = 100000000;

        // sel4_microkit::debug_println!("Request sent! Let us wait!");

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

        if let Err(_) = res {
            sel4_microkit::debug_println!("Resp[0] value {:08x}", resp[0]);
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
                    let hardware: &dyn SdmmcHardware = this.hardware;
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
