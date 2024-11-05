use core::fmt::{self, Write};

use super::mmc_struct::{self, MmcState};

pub(crate) struct Sdcard {
    pub card_id: u128,
    pub manufacture_info: Cid,
    pub card_specific_data: Csd,
    pub card_version: SdVersion,
    pub relative_card_addr: u16,
    pub card_state: MmcState,
    pub card_config: Option<Scr>,
}

/// Placeholder eMMC struct that is not implemented
pub struct EMmc {
    pub card_id: u128,
}

// Beware this struct is meant to track the cmd set that the sdcard should support
// For example, if the SdVersion is set to V3_0, it does not mean the card version is 3.0
// But mean that the sdcard support cmd at least up to specification 3.0
// The SD card specification is cumulative, meaning that if an SD card reports support for a
// particular version (say 4.0), it implicitly supports all earlier versions as well.
#[derive(Debug, PartialEq, Eq)]
pub enum SdVersion {
    V1_0 = 1,
    V2_0 = 2,
    V3_0 = 3,
    V4_0 = 4,
}

pub struct Cid {
    manufacturer_id: u8,
    oem_id: u16,
    product_name: [u8; 5],
    product_revision: u8,
    serial_number: u32,
    manufacturing_date: (u32, u8), // (year, month)
}

impl Cid {
    pub fn new(cid: [u32; 4]) -> Cid {
        // Combine the 4 u32 words into a single 128-bit CID value for easy bit manipulation
        let cid_combined = ((cid[0] as u128) << 96)
            | ((cid[1] as u128) << 64)
            | ((cid[2] as u128) << 32)
            | (cid[3] as u128);

        // Extract each field based on the CID structure
        let manufacturer_id = ((cid_combined >> 120) & 0xFF) as u8;
        let oem_id = ((cid_combined >> 104) & 0xFFFF) as u16;

        // Extract product name, which is 5 bytes (40 bits)
        let product_name = [
            ((cid_combined >> 96) & 0xFF) as u8,
            ((cid_combined >> 88) & 0xFF) as u8,
            ((cid_combined >> 80) & 0xFF) as u8,
            ((cid_combined >> 72) & 0xFF) as u8,
            ((cid_combined >> 64) & 0xFF) as u8,
        ];

        let product_revision = ((cid_combined >> 56) & 0xFF) as u8;
        let serial_number = ((cid_combined >> 24) & 0xFFFFFFFF) as u32;

        // Extract year and month from the manufacturing date
        let year = ((cid_combined >> 12) & 0x0F) as u32 + 2000;
        let month = ((cid_combined >> 8) & 0x0F) as u8;

        Cid {
            manufacturer_id,
            oem_id,
            product_name,
            product_revision,
            serial_number,
            manufacturing_date: (year, month),
        }
    }
}

trait ToArray {
    fn to_array(&self) -> [u8; 128]; // A fixed buffer size that you can change as needed
}

impl ToArray for Cid {
    fn to_array(&self) -> [u8; 128] {
        let mut buf = [0u8; 128];
        let mut writer = ArrayWriter::new(&mut buf);

        write!(
            writer,
            "Manufacturer ID: {}\nOEM ID: {}\nProduct Name: {}\nProduct Revision: {}\n\
            Serial Number: {}\nManufacturing Date: {}-{}\n",
            self.manufacturer_id,
            self.oem_id,
            core::str::from_utf8(&self.product_name).unwrap_or("?????"),
            self.product_revision,
            self.serial_number,
            self.manufacturing_date.0,
            self.manufacturing_date.1,
        )
        .ok();

        buf
    }
}

// This struct is super unreliable, I am thinking
pub struct Csd {
    csd_structure: u8,
    card_capacity: u64,
    max_read_block_len: u16,
    max_write_block_len: u16,
    erase_sector_size: u32,
    supports_partial_write: bool,
}

impl Csd {
    pub fn new(csd: [u32; 4]) -> (Csd, SdVersion) {
        // Combine the four 32-bit words into a single 128-bit value for easier bit manipulation
        let csd_combined = ((csd[0] as u128) << 96)
            | ((csd[1] as u128) << 64)
            | ((csd[2] as u128) << 32)
            | (csd[3] as u128);

        // Extract the CSD structure version
        let csd_structure = ((csd_combined >> 126) & 0x3) as u8; // Bits 126–127
        let sd_version = match csd_structure {
            0 => SdVersion::V1_0,                             // CSD Version 1.0
            1 => SdVersion::V2_0,                             // CSD Version 2.0
            _ => panic!("Unsupported CSD structure version"), // CSD structures beyond 2.0 are not supported here
        };

        // Parse fields based on CSD version
        let (card_capacity, erase_sector_size) = match sd_version {
            SdVersion::V1_0 => {
                // CSD Version 1.0 capacity calculation
                let c_size = ((csd_combined >> 62) & 0xFFF) as u64; // Bits 62–73
                let c_size_mult = ((csd_combined >> 47) & 0x7) as u64; // Bits 47–49
                let read_bl_len = ((csd_combined >> 80) & 0xF) as u64; // Bits 80–83
                let card_capacity = (c_size + 1) * (1 << (c_size_mult + 2)) * (1 << read_bl_len);

                // Erase sector size is calculated differently in CSD Version 1.0
                let sector_size = ((csd_combined >> 39) & 0x7F) as u32 + 1; // Bits 39–45
                (card_capacity, sector_size)
            }
            SdVersion::V2_0 => {
                // CSD Version 2.0 capacity calculation for SDHC/SDXC
                let c_size = ((csd_combined >> 48) & 0x3FFFF) as u64; // Bits 48–69
                let card_capacity = (c_size + 1) * 512 * 1024; // Capacity formula for SDHC/SDXC

                // Erase sector size calculation for CSD Version 2.0
                let sector_size = ((csd_combined >> 39) & 0x7F + 1) as u32 * 512; // Bits 39–45

                (card_capacity, sector_size)
            }
            SdVersion::V3_0 => unreachable!(),
            SdVersion::V4_0 => unreachable!(),
        };

        // Block lengths (same for both versions)
        let read_bl_len = ((csd_combined >> 80) & 0xF) as u16; // Bits 80–83
        let max_read_block_len = 1 << read_bl_len;

        let write_bl_len = ((csd_combined >> 22) & 0xF) as u16; // Bits 22–25
        let max_write_block_len = 1 << write_bl_len;

        // Partial write support (same for both versions)
        let supports_partial_write = ((csd_combined >> 21) & 0x1) != 0; // Bit 21

        // Return the constructed CSD struct along with the SD version
        (
            Csd {
                csd_structure,
                card_capacity,
                max_read_block_len,
                max_write_block_len,
                erase_sector_size,
                supports_partial_write,
            },
            sd_version,
        )
    }
}

pub struct Scr {
    sd_spec: u8,
    data_stat_after_erase: bool,
    sd_security: u8,
    sd_bus_widths: u8,
    sd_spec3: bool,
    sd_spec4: bool,
    supports_cmd23: bool,
}

struct ArrayWriter<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> ArrayWriter<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, pos: 0 }
    }
}

impl<'a> Write for ArrayWriter<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        let len = bytes.len();

        if self.pos + len > self.buf.len() {
            return Err(fmt::Error);
        }

        self.buf[self.pos..self.pos + len].copy_from_slice(bytes);

        self.pos += len;
        Ok(())
    }
}
