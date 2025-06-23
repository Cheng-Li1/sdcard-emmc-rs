use core::ptr;

use sdmmc_protocol::{sdmmc::{MmcPowerMode, MmcSignalVoltage, SdmmcError}, sdmmc_os::VoltageSwitch};

struct Odroidc4VoltageSwitch;

const AO_RTI_OUTPUT_ENABLE_REG: u64 = 0xff800024;
const AO_RTI_OUTPUT_LEVEL_REG: u64 = 0xff800034;
const AO_RTI_PULL_UP_EN_REG: u64 = 0xff800030;
const GPIO_AO_3: u32 = 1 << 3;

impl VoltageSwitch for Odroidc4VoltageSwitch {
    fn card_voltage_switch(&mut self, voltage: MmcSignalVoltage) -> Result<(), SdmmcError> {
        match voltage {
            MmcSignalVoltage::Voltage330 => {
                let mut value: u32;
                unsafe {
                    value = ptr::read_volatile(AO_RTI_OUTPUT_ENABLE_REG as *const u32);
                }
                value &= !(1 << 6);
                unsafe {
                    ptr::write_volatile(AO_RTI_OUTPUT_ENABLE_REG as *mut u32, value);
                }
                unsafe {
                    value = ptr::read_volatile(AO_RTI_OUTPUT_LEVEL_REG as *const u32);
                }
                value &= !(1 << 6);
                unsafe {
                    ptr::write_volatile(AO_RTI_OUTPUT_LEVEL_REG as *mut u32, value);
                }
            }
            MmcSignalVoltage::Voltage180 => {
                let mut value: u32;
                unsafe {
                    value = ptr::read_volatile(AO_RTI_OUTPUT_ENABLE_REG as *const u32);
                }
                value &= !(1 << 6);
                unsafe {
                    ptr::write_volatile(AO_RTI_OUTPUT_ENABLE_REG as *mut u32, value);
                }
                unsafe {
                    value = ptr::read_volatile(AO_RTI_OUTPUT_LEVEL_REG as *const u32);
                }
                value |= 1 << 6;
                unsafe {
                    ptr::write_volatile(AO_RTI_OUTPUT_LEVEL_REG as *mut u32, value);
                }
            }
            MmcSignalVoltage::Voltage120 => return Err(SdmmcError::EINVAL),
        }
        // Disable pull-up/down for gpioAO_6
        let mut value: u32;
        
        unsafe {
            value = ptr::read_volatile(AO_RTI_PULL_UP_EN_REG as *const u32);
        }
        value &= !(1 << 6);

        unsafe {
            ptr::write_volatile(AO_RTI_PULL_UP_EN_REG as *mut u32, value);
        }

        Ok(())
    }

    fn card_power_cycling(&mut self, power_mode: MmcPowerMode) -> Result<(), SdmmcError> {
        let mut value: u32;
        unsafe {
            value = ptr::read_volatile(AO_RTI_OUTPUT_ENABLE_REG as *const u32);
        }
        // If the GPIO pin is not being set as output, set it to output first
        if value & GPIO_AO_3 != 0 {
            value &= !GPIO_AO_3;
            unsafe {
                ptr::write_volatile(AO_RTI_OUTPUT_ENABLE_REG as *mut u32, value);
            }
        }
        match power_mode {
            MmcPowerMode::On => {
                unsafe {
                    value = ptr::read_volatile(AO_RTI_OUTPUT_LEVEL_REG as *const u32);
                }
                if value & GPIO_AO_3 == 0 {
                    value |= GPIO_AO_3;
                    unsafe {
                        ptr::write_volatile(AO_RTI_OUTPUT_LEVEL_REG as *mut u32, value);
                    }
                }
                self.card_voltage_switch(MmcSignalVoltage::Voltage330)?;
            }
            MmcPowerMode::Off => {
                unsafe {
                    value = ptr::read_volatile(AO_RTI_OUTPUT_LEVEL_REG as *const u32);
                }
                if value & GPIO_AO_3 != 0 {
                    value &= !GPIO_AO_3;
                    unsafe {
                        ptr::write_volatile(AO_RTI_OUTPUT_LEVEL_REG as *mut u32, value);
                    }
                }
            }
            _ => return Err(SdmmcError::EINVAL),
        }
        Ok(())
    }
}