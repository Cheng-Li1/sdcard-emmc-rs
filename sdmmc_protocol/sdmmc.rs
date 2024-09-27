use core::{future::Future, pin::Pin, task::{Context, Poll, Waker}};

pub struct SdmmcCmd {
    pub cmdidx: u32,
    pub resp_type: u32,
    pub cmdarg: u32,
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
    // This error should not be triggered unless there are bugs in program
    EUNDEFINED,
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

    fn sdmmc_receive_response(&self, cmd: &SdmmcCmd, response: &mut [u32; 4]) -> Result<(), SdmmcHalError> {
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
        response: &'c mut [u32; 4]) -> SdmmcCmdFuture<'a, 'b, 'c> {
        SdmmcCmdFuture{
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