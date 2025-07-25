use sdhci::sdhci_trait::SdhciHardware;
use sdmmc_protocol::sdmmc::SdmmcError;

pub struct SdhciArasan {

}

impl SdhciArasan {
    pub fn new() -> Self {
        SdhciArasan {}
    }
}

impl SdhciHardware for SdhciArasan {
    fn get_clock_rate(&mut self) -> Result<u64, SdmmcError> {
        return Err(SdmmcError::ENOTIMPLEMENTED);
    }

    fn execute_tuning(&mut self)  -> Result<(), SdmmcError> {
        return Err(SdmmcError::ENOTIMPLEMENTED);
    }
}