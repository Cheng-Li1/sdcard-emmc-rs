use core::fmt::{self, Write};

pub struct Sdcard {
    manufacture_info: Cid, 
    card_specific_data: Csd, 

}

struct Cid {
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
        ).ok();
        
        buf
    }
}

struct Csd {
    csd_structure: u8,
    taac: u8,
    nsac: u8,
    tran_speed: u8,
    card_capacity: u32,
    max_read_block_len: u16,
    max_write_block_len: u16,
    erase_sector_size: u8,
    supports_partial_write: bool,
}

struct Scr {
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
        Self {buf, pos: 0}
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