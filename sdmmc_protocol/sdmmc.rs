pub struct SdmmcCmd {
    pub cmdidx: u32,
    pub resp_type: u32,
    pub cmdarg: u32,
    pub response: [u32; 4],
}

pub struct MmcData {
    pub blocksize: u32,
    pub blocks: u32,
    pub flags: MmcDataFlag ,
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
}

const MMC_RSP_PRESENT: u32 = 1 << 0;
const MMC_RSP_CRC: u32 = 1 << 2;
const MMC_RSP_OPCODE: u32 = 1 << 4;
pub const MMC_RSP_R7: u32 = MMC_RSP_PRESENT | MMC_RSP_CRC | MMC_RSP_OPCODE;

pub const MMC_RSP_NONE: u32 = 0;