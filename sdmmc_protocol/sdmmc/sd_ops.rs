
use crate::sdmmc_traits::SdmmcHardware;

use super::{
    sdcard::{Scr, Sdcard},
    sdmmc_constant::{MMC_CMD_APP_CMD, SD_CMD_APP_SEND_SCR},
    MmcData, MmcDataFlag, SdmmcCmd, SdmmcError, MMC_RSP_R1,
};

impl Sdcard {
    pub fn sdcard_get_configuration_register<T: SdmmcHardware>(
        hardware: &mut T,
        physical_memory: u64,
        memory: &mut [u8; 64],
        invalidate_cache_fn: fn(),
        rca: u16,
    ) -> Result<Scr, SdmmcError> {
        let mut resp: [u32; 4] = [0; 4];
        let mut cmd: SdmmcCmd = SdmmcCmd {
            cmdidx: MMC_CMD_APP_CMD,
            resp_type: MMC_RSP_R1,
            cmdarg: (rca as u32) << 16,
        };
        hardware.sdmmc_do_request(&cmd, None, &mut resp, 0)?;

        cmd = SdmmcCmd {
            cmdidx: SD_CMD_APP_SEND_SCR,
            resp_type: MMC_RSP_R1,
            cmdarg: 0,
        };
        let data: MmcData = MmcData {
            blocksize: 8,
            blockcnt: 1,
            flags: MmcDataFlag::SdmmcDataRead,
            addr: physical_memory,
        };

        hardware.sdmmc_do_request(&cmd, Some(&data), &mut resp, 0)?;

        invalidate_cache_fn();

        // print out the content of the SCR register
        sel4_microkit_support::debug_log!("SCR register content: ");
        unsafe { crate::sdmmc::print_one_block(memory.as_ptr(), 8) };

        // The sdcard register data is always in big endian format
        // Now we construct the last 32 bits of the scr register
        let scr_raw: u64 = (((memory[0] as u64) << 24) 
            + ((memory[1] as u64) << 16) 
            + ((memory[2] as u64) << 8) 
            + (memory[3] as u64)) << 32;

        let scr: Scr = Scr::new(scr_raw)?;

        Ok(scr)
    }
}
