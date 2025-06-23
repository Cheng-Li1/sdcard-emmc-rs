use sdmmc_protocol::sdmmc_os::Sleep;

mod odroidc4;
struct MicrokitDelay;

impl Sleep for MicrokitDelay {}