#![no_std]  // Don't link the standard library
#![no_main] // Don't use the default entry point

pub mod meson_gx_mmc;

use sel4_microkit::{debug_println, protection_domain, Handler, Infallible};


const SDIO_BASE: u64 = 0xffe03000;

#[protection_domain]
fn init() -> HandlerImpl {
    debug_println!("Try!");
    HandlerImpl
}

struct HandlerImpl;

impl Handler for HandlerImpl {
    type Error = Infallible;
    
}