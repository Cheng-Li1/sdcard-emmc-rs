pub struct SdmmcCmd {
    pub cmdidx: u32,
    pub resp_type: u32,
    pub cmdarg: u32,
    pub response: [u32; 4],
}

pub struct MmcData {
    pub blocksize: u32,
    pub blocks: u32,
    pub flags: MmcDataFlag,
    pub addr: u32,
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
    ENOTIMPLEMENTED,
}

// Define the MMC response flags
const MMC_RSP_PRESENT: u32 = 1 << 0;
const MMC_RSP_136: u32 = 1 << 1;       // 136-bit response
const MMC_RSP_CRC: u32 = 1 << 2;       // Expect valid CRC
const MMC_RSP_BUSY: u32 = 1 << 3;      // Card may send busy
const MMC_RSP_OPCODE: u32 = 1 << 4;    // Response contains opcode

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


// Program async Rust can be very dangerous if you do not know what is happening understand the hood
pub trait SdmmcHardware {
    fn power_up(&mut self) -> Result<(), SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }

    fn set_ios(&mut self) -> Result<(), SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }

    fn send_command(&mut self, cmd: &SdmmcCmd, data: Option<&MmcData>) -> Result<(), SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }

    fn receive_response(&self, cmd: &mut SdmmcCmd) -> Result<(), SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }

    fn set_interrupt(&mut self, status: bool) -> Result<(), SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }

    fn ack_interrupt(&mut self) -> Result<(), SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }

    fn power_off(&mut self) -> Result<(), SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }
}

