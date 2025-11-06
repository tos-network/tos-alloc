//! Minimal test - just write to heap

#![no_std]
#![no_main]

use tako_sdk::*;

#[no_mangle]
pub extern "C" fn entrypoint() -> u64 {
    log("Minimal heap write test");

    // Try to write directly to heap start
    unsafe {
        let heap_start = 0x300000000 as *mut u64;
        log("Writing to heap...");
        *heap_start = 0x42;
        log("Write successful!");

        let value = *heap_start;
        log_u64(value, 0, 0, 0, 0);
    }

    log("Test passed");
    SUCCESS
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
