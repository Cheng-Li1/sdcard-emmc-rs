use sdmmc_protocol::sdmmc::{mmc_struct::MmcTiming, SdmmcError};

pub trait SdhciHardware {
    /// Get the clock rate for sdcard host
    fn get_host_clock(&mut self) -> Result<u64, SdmmcError> {
        return Err(SdmmcError::ENOTIMPLEMENTED);
    }

    fn execute_tuning(&mut self) -> Result<(), SdmmcError> {
        return Err(SdmmcError::ENOTIMPLEMENTED);
    }

    /// Set the clock rate for sdcard
    fn set_card_freq(&mut self, timing: MmcTiming) -> Result<u64, SdmmcError> {
        // call timing_to_freq() in sdhci
        // sdhci_set_clock()
        return Err(SdmmcError::ENOTIMPLEMENTED);
    }
}
