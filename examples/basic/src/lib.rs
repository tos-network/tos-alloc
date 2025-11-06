//! Basic tos-alloc example
//!
//! Demonstrates Vec and BTreeMap usage with dynamic memory allocation

#![no_std]
#![no_main]

extern crate alloc;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

use tako_sdk::*;
use tos_alloc::TosAllocator;

#[global_allocator]
static ALLOCATOR: TosAllocator = TosAllocator::new();

/// Main contract entrypoint
#[no_mangle]
pub extern "C" fn entrypoint() -> u64 {
    log("=== TOS Alloc Basic Example ===");

    // Test 1: Vec operations
    log("Test 1: Vec operations");
    let mut numbers = Vec::new();
    for i in 0..10 {
        numbers.push(i);
    }
    // Log the Vec length (log_u64 requires 5 u64 parameters)
    log_u64(numbers.len() as u64, 0, 0, 0, 0);

    // Test 2: BTreeMap operations
    log("Test 2: BTreeMap operations");
    let mut map = BTreeMap::new();
    map.insert(1u32, 100u32);
    map.insert(2u32, 200u32);
    map.insert(3u32, 300u32);
    log_u64(map.len() as u64, 0, 0, 0, 0);

    // Test 3: Heap usage
    log("Test 3: Heap usage");
    log_u64(ALLOCATOR.used() as u64, ALLOCATOR.remaining() as u64, 0, 0, 0);

    log("=== All tests passed ===");
    SUCCESS
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
