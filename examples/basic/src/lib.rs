//! Basic tos-alloc example (Heap-Based Allocator)
//!
//! Demonstrates Vec and BTreeMap usage with heap-based dynamic memory allocation

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
    log("=== TOS Alloc Basic Example (Solana-Style) ===");

    // Test 0: Check heap region info BEFORE any allocation
    log("Test 0: Checking heap region syscall...");
    let (heap_start, heap_size) = get_heap_region();
    log("Heap region received:");
    log_u64(heap_start, heap_size, 0, 0, 0);

    // Validate heap region
    if heap_start != 0x300000000 {
        log("ERROR: Unexpected heap_start!");
        log_u64(heap_start, 0, 0, 0, 0);
        return 1; // Error code
    }
    if heap_size != 32768 {
        log("ERROR: Unexpected heap_size!");
        log_u64(heap_size, 0, 0, 0, 0);
        return 1; // Error code
    }
    log("✓ Heap region is correct");

    // Test 1: Vec operations
    log("Test 1: Vec operations");
    let mut numbers = Vec::new();
    for i in 0..10 {
        numbers.push(i);
    }
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
    let (used, remaining) = TosAllocator::usage();
    log_u64(used as u64, remaining as u64, 0, 0, 0);

    log("=== All tests passed ===");
    SUCCESS
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
