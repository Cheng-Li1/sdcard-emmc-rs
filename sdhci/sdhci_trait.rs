use sdmmc_protocol::sdmmc::SdmmcError;

pub trait SdhciHardware {
    fn get_clock_rate(&mut self) -> Result<u64, SdmmcError> {
        return Err(SdmmcError::ENOTIMPLEMENTED);
    }

    fn execute_tuning(&mut self)  -> Result<(), SdmmcError> {
        return Err(SdmmcError::ENOTIMPLEMENTED);
    }
}
