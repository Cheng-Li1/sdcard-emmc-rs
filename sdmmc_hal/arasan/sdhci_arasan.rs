use sdhci::sdhci_trait::SdhciHardware;
use sdmmc_protocol::dev_log;
use sdmmc_protocol::sdmmc::SdmmcError;

use tock_registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

const PS_REF_CLK: u64 = 33_333_333;

register_bitfields![u32,
    IOPLL_CTRL [
        RESET OFFSET(0) NUMBITS(1) [
            Off = 0,
            On = 1,
        ],
        BYPASS OFFSET(3) NUMBITS(1) [
            Off = 0,
            On = 1,
        ],
        FBDIV OFFSET(8) NUMBITS(7) [],
        DIV2 OFFSET(16) NUMBITS(1) [
            Off = 0,
            On = 1,
        ],
        PRE_SRC_IS_NOT_PS_REF_CLK OFFSET(20) NUMBITS(1) [
            Off = 0,
            On = 1,
        ]
    ],
    SDIO1_REF_CTRL [
        SRCSEL OFFSET(0) NUMBITS(3) [
            IOPLL = 0,
            RPLL = 2,
            VPLL_CLK_TO_LPD = 3,
        ],
        DIVISOR0 OFFSET(8) NUMBITS(6) [],
        DIVISOR1 OFFSET(16) NUMBITS(6) [],
        CLKACT OFFSET(20) NUMBITS(1) [
            Disable = 0,
            Enable = 1,
        ],
    ],
];

tock_registers::register_structs! {
    pub CrlApbRegister {
        (0x000 => _reserved0),
        (0x020 => iopll_ctrl: ReadWrite<u32, IOPLL_CTRL::Register>),
        (0x024 => _reserved1),
        (0x070 => sdio1_ref_ctrl: ReadWrite<u32, SDIO1_REF_CTRL::Register>),
        (0x074 => _reserved2),
        (0x200 => @END),
    },
    pub IouSlcrRegister {
        (0x00000 => _reserved0),
        (0x0030C => sdio_clk_ctrl: ReadWrite<u32>),
        (0x00310 => ctrl_reg_sd: ReadWrite<u32>),
        (0x00314 => sd_itapdly: ReadWrite<u32>),
        (0x00318 => sd_otapdly: ReadWrite<u32>),
        (0x0031C => sd_config_reg1: ReadWrite<u32>),
        (0x00320 => sd_config_reg2: ReadWrite<u32>),
        (0x00324 => sd_config_reg3: ReadWrite<u32>),
        (0x00328 => _reserved1),
        (0x10000 => @END),
    }
}

impl CrlApbRegister {
    unsafe fn new(register_base: u64) -> &'static mut CrlApbRegister {
        unsafe { &mut *(register_base as *mut CrlApbRegister) }
    }
}

impl IouSlcrRegister {
    unsafe fn new(register_base: u64) -> &'static mut IouSlcrRegister {
        unsafe { &mut *(register_base as *mut IouSlcrRegister) }
    }
}

pub struct SdhciArasan {
    crl_apb: &'static mut CrlApbRegister,
    iou_slcr: &'static mut IouSlcrRegister,
}

impl SdhciArasan {
    pub fn new(crl_apb_register_base: u64, iou_slcr_register_base: u64) -> Self {
        let crl_apb: &'static mut CrlApbRegister =
            unsafe { CrlApbRegister::new(crl_apb_register_base) };

        let iou_slcr: &'static mut IouSlcrRegister =
            unsafe { IouSlcrRegister::new(iou_slcr_register_base) };

        SdhciArasan { crl_apb, iou_slcr }
    }
}

impl SdhciHardware for SdhciArasan {
    fn get_clock_rate(&mut self) -> Result<u64, SdmmcError> {
        if self
            .crl_apb
            .iopll_ctrl
            .is_set(IOPLL_CTRL::PRE_SRC_IS_NOT_PS_REF_CLK)
        {
            panic!("invalid iopll source");
        }

        let iopll_clk = PS_REF_CLK * (self.crl_apb.iopll_ctrl.read(IOPLL_CTRL::FBDIV) as u64)
            / (1 << self.crl_apb.iopll_ctrl.read(IOPLL_CTRL::DIV2));
        dev_log!("IOPLL clock: {} Hz\n", iopll_clk);

        if !self
            .crl_apb
            .sdio1_ref_ctrl
            .matches_all(SDIO1_REF_CTRL::SRCSEL::IOPLL)
        {
            panic!("invalid sd source");
        }

        let sd1_baseclk = iopll_clk
            * (self.crl_apb.sdio1_ref_ctrl.is_set(SDIO1_REF_CTRL::CLKACT) as u64)
            / (self.crl_apb.sdio1_ref_ctrl.read(SDIO1_REF_CTRL::DIVISOR0) as u64)
            / (self.crl_apb.sdio1_ref_ctrl.read(SDIO1_REF_CTRL::DIVISOR1) as u64);

        dev_log!("SD1 base clock: {}\n", sd1_baseclk);

        return Ok(sd1_baseclk);
    }

    fn execute_tuning(&mut self) -> Result<(), SdmmcError> {
        return Err(SdmmcError::ENOTIMPLEMENTED);
    }
}
