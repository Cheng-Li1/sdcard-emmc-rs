#![no_std] // Don't link the standard library
#![no_main] // Don't use the default entry point

extern crate alloc;

mod sddf_blk;

use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use alloc::boxed::Box;
use sddf_blk::{
    blk_dequeue_req_helper, blk_enqueue_resp_helper, blk_queue_empty_req_helper,
    blk_queue_full_resp_helper, blk_queue_init_helper, BlkOp, BlkRequest, BlkStatus,
};
use sdmmc_hal::meson_gx_mmc::MesonSdmmcRegisters;

use sdmmc_protocol::sdmmc::{
    sdmmc_capability::{MMC_INTERRUPT_END_OF_CHAIN, MMC_INTERRUPT_ERROR},
    SdmmcHalError, SdmmcHardware, SdmmcProtocol,
};
use sel4_microkit::{debug_print, debug_println, protection_domain, Channel, Handler, Infallible};

const BLK_VIRTUALIZER: sel4_microkit::Channel = sel4_microkit::Channel::new(0);

const INTERRUPT: sel4_microkit::Channel = sel4_microkit::Channel::new(1);

const SDCARD_SECTOR_SIZE: u32 = 512;
const SDDF_TRANSFER_SIZE: u32 = 4096;
const SDDF_TO_REAL_SECTOR: u32 = SDDF_TRANSFER_SIZE / SDCARD_SECTOR_SIZE;

const RETRY_CHANCE: u16 = 5;

// Debug function for printing out content in one block
#[allow(dead_code)]
fn print_one_block(ptr: *const u8, num: usize) {
    unsafe {
        // Iterate over the number of bytes and print each one in hexadecimal format
        for i in 0..num {
            let byte = *ptr.add(i);
            if i % 16 == 0 {
                debug_print!("\n{:04x}: ", i);
            }
            debug_print!("{:02x} ", byte);
        }
        debug_println!();
    }
}

// No-op waker implementations, they do nothing.
unsafe fn noop(_data: *const ()) {}
unsafe fn noop_clone(_data: *const ()) -> RawWaker {
    RawWaker::new(_data, &VTABLE)
}

/// Since in .system file, the page we are providing to tune_performance function is uncached
/// we do not need to provide a real cache invalidate function
fn dummy_cache_invalidate_function() {}

// A VTable that points to the no-op functions
static VTABLE: RawWakerVTable = RawWakerVTable::new(noop_clone, noop, noop, noop);

// Function to create a dummy Waker
fn create_dummy_waker() -> Waker {
    let raw_waker = RawWaker::new(core::ptr::null(), &VTABLE);
    unsafe { Waker::from_raw(raw_waker) }
}

#[protection_domain(heap_size = 0x10000)]
fn init() -> HandlerImpl<'static, MesonSdmmcRegisters> {
    debug_println!("Driver init!");
    unsafe {
        blk_queue_init_helper();
    }
    let meson_hal: &mut MesonSdmmcRegisters = MesonSdmmcRegisters::new();

    let unsafe_stolen_memory: u32;

    // This line of code actually is very unsafe!
    // Considering the memory is stolen from the memory that has sdcard registers mapped in
    {
        unsafe_stolen_memory = (meson_hal as *const _ as usize + 0x800) as u32;
        assert!(unsafe_stolen_memory % 8 == 0);
        
    }

    // Handling result in two different ways, by matching and unwrap_or_else
    let res = SdmmcProtocol::new(meson_hal);
    let mut sdmmc_host = match res {
        Ok(host) => host,
        Err(err) => panic!("SDMMC: Error at init {:?}", err),
    };

    sdmmc_host
        .setup_card()
        .unwrap_or_else(|error| panic!("SDMMC: Error at setup {:?}", error));

    // TODO: Should tuning be possible to fail?
    sdmmc_host
        .tune_performance(Some((unsafe_stolen_memory, dummy_cache_invalidate_function)))
        .unwrap_or_else(|error| panic!("SDMMC: Error at tuning performance {:?}", error));

    let mut irq_to_enable = MMC_INTERRUPT_ERROR | MMC_INTERRUPT_END_OF_CHAIN;

    // Should always succeed, at least for odroid C4
    let _ = sdmmc_host.enable_interrupt(&mut irq_to_enable);
    HandlerImpl {
        future: None,
        sdmmc: Some(sdmmc_host),
        request: None,
        retry: RETRY_CHANCE,
    }
}

struct HandlerImpl<'a, T: SdmmcHardware> {
    future: Option<
        Pin<
            Box<
                dyn Future<Output = (Result<(), SdmmcHalError>, Option<SdmmcProtocol<'a, T>>)> + 'a,
            >,
        >,
    >,
    sdmmc: Option<SdmmcProtocol<'a, T>>,
    request: Option<BlkRequest>,
    retry: u16,
}

impl<'a, T: SdmmcHardware> Handler for HandlerImpl<'a, T> {
    type Error = Infallible;

    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        debug_println!("SDMMC_DRIVER: MESSAGE FROM CHANNEL: {}", channel.index());

        if channel.index() != INTERRUPT.index() && channel.index() != BLK_VIRTUALIZER.index() {
            debug_println!(
                "SDMMC_DRIVER: Unknown channel sent me message: {}",
                channel.index()
            );
            return Ok(());
        }

        let mut notify_virt: bool = false;
        loop {
            // Polling if receive any notification, it is to poll even the notification is not from the interrupt as polling is cheap
            if let Some(request) = &mut self.request {
                if let Some(future) = &mut self.future {
                    let waker = create_dummy_waker();
                    let mut cx = Context::from_waker(&waker);
                    match future.as_mut().poll(&mut cx) {
                        Poll::Ready((result, sdmmc)) => {
                            // debug_println!("SDMMC_DRIVER: Future completed with result");
                            self.future = None; // Reset the future once done
                            self.sdmmc = sdmmc;
                            if result.is_err() {
                                debug_println!(
                                    "SDMMC_DRIVER: DISK ERROR ENCOUNTERED, possibly retry!"
                                );
                                self.retry -= 1;
                            } else {
                                // Deduct finished count from count
                                request.success_count += request.count_to_do;
                                request.count -= request.count_to_do as u16;
                            }
                            if request.count == 0 {
                                let resp_status = BlkStatus::BlkRespOk;
                                notify_virt = true;
                                unsafe {
                                    blk_enqueue_resp_helper(
                                        resp_status,
                                        request.success_count / SDDF_TO_REAL_SECTOR,
                                        request.id,
                                    );
                                }
                                self.request = None;
                            } else if self.retry == 0 {
                                let resp_status = BlkStatus::BlkRespSeekError;
                                notify_virt = true;
                                unsafe {
                                    blk_enqueue_resp_helper(
                                        resp_status,
                                        request.success_count / SDDF_TO_REAL_SECTOR,
                                        request.id,
                                    );
                                }
                                self.request = None;
                            }
                        }
                        Poll::Pending => {
                            // debug_println!("SDMMC_DRIVER: Future is not ready, polling again...");
                            // Since the future is not ready, no other request can be dequeued, exit the big loop
                            break;
                        }
                    }
                }
            }

            while self.request.is_none()
                && unsafe { blk_queue_empty_req_helper() == 0 && blk_queue_full_resp_helper() == 0 }
            {
                let mut request: BlkRequest = BlkRequest {
                    request_code: BlkOp::BlkReqFlush,
                    io_or_offset: 0,
                    block_number: 0,
                    count: 0,
                    success_count: 0,
                    count_to_do: 0,
                    id: 0,
                };
                unsafe {
                    blk_dequeue_req_helper(
                        &mut request.request_code as *mut BlkOp,
                        &mut request.io_or_offset as *mut u64,
                        &mut request.block_number as *mut u32,
                        &mut request.count as *mut u16,
                        &mut request.id as *mut u32,
                    );
                }
                request.block_number = request.block_number * SDDF_TO_REAL_SECTOR;
                request.count = request.count * SDDF_TO_REAL_SECTOR as u16;
                // Print the retrieved values
                /*
                debug_println!("io_or_offset: 0x{:x}", request.io_or_offset);// Simple u64
                debug_println!("block_number: {}", request.block_number);    // Simple u32
                debug_println!("count: {}", request.count);                  // Simple u16
                debug_println!("id: {}", request.id);                        // Simple u32
                */
                match request.request_code {
                    BlkOp::BlkReqRead => {
                        // Reset retry chance here
                        self.retry = RETRY_CHANCE;
                        self.request = Some(request);
                        break;
                    }
                    BlkOp::BlkReqWrite => {
                        // Reset retry chance here
                        self.retry = RETRY_CHANCE;
                        self.request = Some(request);
                        break;
                    }
                    _ => {
                        // For other request, enqueue response
                        notify_virt = true;
                        unsafe {
                            blk_enqueue_resp_helper(BlkStatus::BlkRespOk, 0, request.id);
                        }
                    }
                }
            }
            // If future is empty
            if let Some(request) = &mut self.request {
                if let None = self.future {
                    match request.request_code {
                        BlkOp::BlkReqRead => {
                            // TODO: The MAX_BLOCK_PER_TRANSFER is got by hackily get the defines in hardware layer which is wrong, check that to get properly from protocol layer
                            request.count_to_do = core::cmp::min(
                                request.count as u32,
                                sdmmc_hal::meson_gx_mmc::MAX_BLOCK_PER_TRANSFER,
                            );
                            if let Some(sdmmc) = self.sdmmc.take() {
                                self.future = Some(Box::pin(sdmmc.read_block(
                                    request.count_to_do as u32,
                                    request.block_number as u64 + request.success_count as u64,
                                    request.io_or_offset
                                        + request.success_count as u64 * SDCARD_SECTOR_SIZE as u64,
                                )));
                            } else {
                                panic!("SDMMC_DRIVER: The sdmmc should be here and the future should be empty!!!")
                            }
                        }
                        BlkOp::BlkReqWrite => {
                            // TODO: The MAX_BLOCK_PER_TRANSFER is got by hackily get the defines in hardware layer which is wrong, check that to get properly from protocol layer
                            request.count_to_do = core::cmp::min(
                                request.count as u32,
                                sdmmc_hal::meson_gx_mmc::MAX_BLOCK_PER_TRANSFER,
                            );
                            if let Some(sdmmc) = self.sdmmc.take() {
                                self.future = Some(Box::pin(sdmmc.write_block(
                                    request.count_to_do as u32,
                                    request.block_number as u64 + request.success_count as u64,
                                    request.io_or_offset
                                        + request.success_count as u64 * SDCARD_SECTOR_SIZE as u64,
                                )));
                            } else {
                                panic!("SDMMC_DRIVER: The sdmmc should be here and the future should be empty!!!")
                            }
                        }
                        _ => {
                            panic!("SDMMC_DRIVER: You should not reach here!")
                        }
                    }
                }
            } else {
                // If Request is empty, means there are no future available, so we do not need to poll again
                break;
            }
        }
        if notify_virt == true {
            // debug_println!("SDMMC_DRIVER: Notify the BLK_VIRTUALIZER!");
            BLK_VIRTUALIZER.notify();
        }
        // Ack irq
        if channel.index() == INTERRUPT.index() {
            let err = channel.irq_ack();
            if err.is_err() {
                panic!("SDMMC: Cannot acknowledge interrupt for CPU!")
            }
        }
        Ok(())
    }
}
