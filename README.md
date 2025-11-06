# tos-alloc

Dynamic memory allocator for TOS TAKO VM contracts, enabling the use of `Vec`, `BTreeMap`, `Box`, and other heap-allocated types.

## Features

- **High Performance**: 10-20x faster than syscall-based allocation
  - Best case: ~5-10 CU per allocation
  - Worst case: ~30-90 CU per allocation
  - Compare to syscall: ~100 CU per allocation

- **Lock-Free**: Uses atomic compare-and-swap operations
- **Simple**: Bump allocator design - no deallocation overhead
- **Safe**: Returns null on OOM instead of panicking

## Usage

Add to your TOS contract's `Cargo.toml`:

```toml
[dependencies]
tos-alloc = { path = "../../tos-alloc" }
tako-sdk = { path = "../../tako/sdk" }
```

In your contract code:

```rust
#![no_std]
#![no_main]

extern crate alloc;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

use tako_sdk::*;
use tos_alloc::TosAllocator;

#[global_allocator]
static ALLOCATOR: TosAllocator = TosAllocator::new();

#[no_mangle]
pub extern "C" fn entrypoint() -> u64 {
    // Now you can use heap-allocated types!
    let mut numbers = Vec::new();
    numbers.push(42);

    let mut map = BTreeMap::new();
    map.insert(1, 100);

    SUCCESS
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
```

## Building Examples

```bash
# Build the basic example
./build-example.sh basic

# The output will be in:
# examples/basic/target/tbpf-tos-tos/release/tos_alloc_basic_example.so
```

## Memory Layout

- **Heap Start**: `0x300000000` (matches `tos_tbpf::ebpf::MM_HEAP_START`)
- **Default Size**: 32 KB
- **Maximum Size**: 256 KB (Solana compatible)

## Architecture

The allocator uses a simple bump allocation strategy:

1. Maintains an atomic pointer to the next free address
2. On allocation:
   - Align the pointer to the requested alignment
   - Check if enough space remains
   - Atomically increment the pointer via CAS
   - Return the aligned address
3. Deallocation is a no-op (memory reclaimed when contract execution ends)

This approach is optimal for short-lived contract executions where individual deallocations are unnecessary.

## Testing

The library includes unit tests for the allocator logic:

```bash
cargo test
```

Integration tests are located in `tos/daemon/tests/tako_alloc_integration.rs` and test the allocator within the actual TAKO VM environment.

## License

Apache-2.0

## Repository

https://github.com/tos-network/tos-alloc
