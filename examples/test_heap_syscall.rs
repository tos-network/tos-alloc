//! Test for heap region syscall
//!
//! This test verifies that the tos_get_heap_region syscall works correctly

#![no_std]
#![no_main]

use tako_sdk::*;

#[no_mangle]
pub extern "C" fn entrypoint() -> u64 {
    log("Testing tos_get_heap_region syscall");

    // Call the syscall
    let (heap_start, heap_size) = get_heap_region();

    // Log the results
    log("Heap region information:");
    log_u64(heap_start, heap_size, 0, 0, 0);

    // Verify the values are reasonable
    // Heap should start at 0x300000000 (MM_HEAP_START)
    // Heap size should be 32KB (32768 bytes)
    if heap_start == 0x300000000 && heap_size == 32768 {
        log("SUCCESS: Heap region syscall returned correct values");
        SUCCESS
    } else {
        log("ERROR: Unexpected heap region values");
        ERROR
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
