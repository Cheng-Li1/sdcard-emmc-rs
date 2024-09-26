use core::{future::Future, pin::Pin, task::{Context, Poll}};

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
    fn sdmmc_power_up(&mut self) -> Result<(), SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }

    fn sdmmc_set_ios(&mut self) -> Result<(), SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }

    fn sdmmc_send_command(&mut self, cmd: &SdmmcCmd, data: Option<&MmcData>) -> Result<(), SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }

    fn sdmmc_receive_response(&self, cmd: &mut SdmmcCmd) -> Result<(), SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }

    fn sdmmc_set_interrupt(&mut self, status: bool) -> Result<(), SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }

    fn sdmmc_ack_interrupt(&mut self) -> Result<(), SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }

    fn sdmmc_power_off(&mut self) -> Result<(), SdmmcHalError> {
        return Err(SdmmcHalError::ENOTIMPLEMENTED);
    }
}

pub struct SdmmcProtocol<'a, T: SdmmcHardware> {
    hardware: &'a mut T,
}

impl<'a, T: SdmmcHardware> SdmmcProtocol<'a, T> {
    pub fn new(hardware: &'a mut T) -> Self {
        SdmmcProtocol { hardware }
    }

    pub fn read_block(&self, blocknum: u32, start_idx: u64) {

    }

    async fn send_cmd_and_get_response(cmd: &SdmmcCmd, data: Option<&MmcData>) -> Result<(), SdmmcHalError> {
        Ok(())
    }
}

enum CmdState {
    // Currently sending the command
    NotSend,
    // Waiting for the response
    WaitingForResponse,
    // Error encountered
    Error,
    // Finished
    Finished,
}

pub struct SdmmcCmdFuture<'a, 'b> {
    hardware: &'a mut dyn SdmmcHardware,
    cmd: &'b mut SdmmcCmd,
    data: Option<&'b MmcData>,
    state: CmdState,
    res: Result<(), SdmmcHalError>,
}

/// SdmmcCmdFuture serves as the basic building block for async fn above
/// In the context of Sdmmc device, since the requests are executed linearly under the hood
/// We actually do not need an executor to execute the request
/// The context can be ignored unless someone insist to use an executor for the requests
impl<'a, 'b> Future for SdmmcCmdFuture<'a, 'b> {
    type Output = Result<(), SdmmcHalError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.state {
            CmdState::NotSend => {
                let res;
                {
                    let this = self.as_mut().get_mut();

                    // Now, you can pass `&mut SdmmcCmd` to `sdmmc_send_command`
                    // let cmd = &mut *this.cmd; // Get a mutable reference to `cmd`
                    let cmd = & *this.cmd;
                    let data = this.data;
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
                    let this = self.as_mut().get_mut();
                    let cmd: &mut SdmmcCmd = this.cmd;
                    let hardware: &mut dyn SdmmcHardware = this.hardware;
                    res = hardware.sdmmc_receive_response(cmd);
                }
                if let Ok(()) = res {
                    self.state = CmdState::Finished;
                    return Poll::Ready(res);
                } else {
                    self.state = CmdState::Error;
                    return Poll::Ready(res);
                }

            }
            CmdState::Error => todo!(),
            CmdState::Finished => todo!(),
        }
        Poll::Ready(Ok(()))
    }
}