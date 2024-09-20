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
    pub dest: *const u64,
    pub src: *const u64,
}

pub enum MmcDataFlag {
    SdmmcDataRead,
    SdmmcDataWrite,
}