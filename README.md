# tos-alloc

Dynamic memory allocator for TOS TAKO VM contracts, enabling the use of `Vec`, `BTreeMap`, `Box`, and other heap-allocated types.

## Features

- **Solana-Compatible Design**: Stores allocator state on the heap (no writable `.data` sections needed)
- **Syscall-Based**: Dynamically obtains heap address from VM (avoids eBPF 64-bit constant issues)
- **Simple Bump Allocator**: Sequential allocation, no deallocation overhead
- **Low Overhead**: ~1 CU per allocation (syscall cost)
- **Safe**: Returns null on OOM instead of panicking

## Quick Start

### 1. Add Dependencies

Add to your TOS contract's `Cargo.toml`:

```toml
[dependencies]
tos-alloc = { path = "../../tos-alloc" }
tako-sdk = { path = "../../tako/sdk" }

[profile.release]
opt-level = "z"      # Optimize for size
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization
```

### 2. Use in Your Contract

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
    for i in 0..10 {
        numbers.push(i);
    }

    let mut map = BTreeMap::new();
    map.insert(1, 100);
    map.insert(2, 200);

    // Optional: Check heap usage
    let (used, remaining) = TosAllocator::usage();
    log_u64(used as u64, remaining as u64, 0, 0, 0);

    SUCCESS
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
```

## How It Works

### Architecture

The allocator uses a **Solana-style heap-based bump allocator**:

```
Heap Layout:
┌────────────────────────────────────┐
│ Position Pointer (8 bytes)         │  ← Tracks next free address
├────────────────────────────────────┤
│ User allocations                   │  ← Vec, Box, BTreeMap, etc.
│ ...                                │
└────────────────────────────────────┘
```

### Key Design Decisions

#### 1. **Syscall-Based Heap Discovery**

Unlike hardcoding `HEAP_START = 0x300000000`, we use a syscall:

```rust
let (heap_start, heap_size) = get_heap_region_syscall();
```

**Why?** Hardcoded 64-bit constants in eBPF bytecode can be miscompiled due to immediate value encoding limitations. The syscall approach:
- ✅ Avoids 64-bit constant encoding issues
- ✅ Lets VM provide heap address dynamically
- ✅ Only costs 1 CU per allocation (negligible overhead)

#### 2. **Heap-Based State Storage**

The allocator stores its position pointer at the beginning of the heap, not in global statics:

```rust
let pos_ptr = heap_start as *mut usize;  // First 8 bytes of heap
let mut pos = *pos_ptr;
if pos == 0 {
    pos = heap_start + 8;  // Initialize on first use
}
```

**Why?** This avoids the need for writable `.data` sections in eBPF, which are:
- Not well-supported in eBPF toolchains
- Require complex linker configuration
- Add unnecessary complexity

#### 3. **Bump Allocation Strategy**

Allocations are sequential and never deallocated:

```rust
// Align pointer
pos = (pos + align - 1) & !(align - 1);

// Allocate
let new_pos = pos + layout.size();
if new_pos > heap_end {
    return null_mut();  // Out of memory
}

*pos_ptr = new_pos;
return pos as *mut u8;
```

**Why?** Contract executions are short-lived. Memory is reclaimed when the contract finishes, so individual deallocations are unnecessary overhead.

## Memory Configuration

### Default Settings

- **Heap Start**: `0x300000000` (obtained via syscall, not hardcoded)
- **Default Size**: 32 KB (32,768 bytes)
- **Usable Space**: ~32,760 bytes (8 bytes for position pointer)

### VM Memory Regions

```
0x000000000 - 0x0FFFFFFFF  │  Read-only data (text, rodata)
0x100000000 - 0x1FFFFFFFF  │  (unused)
0x200000000 - 0x2FFFFFFFF  │  Stack (16 KB)
0x300000000 - 0x3FFFFFFFF  │  Heap (32 KB)  ← Allocator uses this
0x400000000 - 0x4FFFFFFFF  │  Input data
```

### Increasing Heap Size

The heap size is configured in the VM executor. To increase:

1. Modify `tos/daemon/src/tako_integration/executor.rs`:
   ```rust
   const HEAP_SIZE: usize = 64 * 1024;  // 64 KB instead of 32 KB
   ```

2. Larger heaps use more host memory but are fine for most contracts

3. Solana maximum: 256 KB (we follow this limit)

## Performance

### Allocation Cost

Each allocation has two components:

1. **Syscall overhead**: 1 CU (to get heap info)
2. **Allocation logic**: ~2-5 CU (alignment, bounds check, pointer update)
3. **Total**: ~3-6 CU per allocation

### Real-World Impact

| Scenario | Allocations | Syscall Overhead | % of 1M CU Budget |
|----------|-------------|------------------|-------------------|
| Simple contract | 5-10 | 5-10 CU | 0.001% |
| Medium contract (Vec, BTreeMap) | 20-50 | 20-50 CU | 0.005% |
| Complex contract | 100-200 | 100-200 CU | 0.020% |

**Conclusion**: Syscall overhead is **negligible** (< 0.02%) for all realistic contracts.

### Comparison with Solana

| Implementation | Per-Allocation Cost | Notes |
|---------------|-------------------|-------|
| **Solana** | 0 CU | Uses linker-injected heap address |
| **TOS (ours)** | 1 CU | Uses syscall to avoid 64-bit constant issues |

**Trade-off**: We accept 1 CU overhead to solve the 64-bit constant encoding problem without complicating the toolchain.

### Optimization Options

If profiling shows syscall overhead > 0.1%, you can cache the heap info:

```rust
use core::sync::atomic::{AtomicUsize, Ordering};

static HEAP_START_CACHE: AtomicUsize = AtomicUsize::new(0);

unsafe fn get_heap_region_cached() -> (usize, usize) {
    let cached = HEAP_START_CACHE.load(Ordering::Relaxed);
    if cached != 0 {
        return (cached, HEAP_SIZE_CACHE.load(Ordering::Relaxed));
    }
    // First call only
    let (start, size) = get_heap_region_syscall();
    HEAP_START_CACHE.store(start, Ordering::Relaxed);
    (start, size)
}
```

**When to optimize**: Only if measurements show it's necessary. Current implementation is sufficient for 99% of contracts.

## Examples

### Basic Example

```bash
cd examples/basic
cargo build --release --target tbpf-tos-tos
```

Tests Vec, BTreeMap, and heap usage statistics:
- See `examples/basic/src/lib.rs` for full source
- Output: Vec length, BTreeMap size, heap usage

### Minimal Example

```bash
cd examples/minimal
cargo build --release --target tbpf-tos-tos
```

Minimal heap write test:
- Verifies basic heap access works
- See `examples/minimal/src/lib.rs`

## Testing

### Unit Tests

```bash
cargo test
```

Runs basic compilation tests.

### Integration Tests

See `TESTING_PLAN.md` for detailed end-to-end testing instructions:

1. Build contract with TOS toolchain
2. Load into TAKO VM
3. Execute and verify Vec/BTreeMap work
4. Check heap usage statistics

### Performance Tests

See `PERFORMANCE_ANALYSIS.md` for detailed performance analysis and optimization recommendations.

## Troubleshooting

### Issue: "StackAccessViolation" Error

**Symptom**: Contract crashes with `StackAccessViolation` when allocating

**Cause**: Syscall not registered or returning wrong address

**Fix**: Ensure `tos_syscalls::register_syscalls()` is called before loading the contract:

```rust
let mut loader = BuiltinProgram::<InvokeContext>::new_loader(config);
tos_syscalls::register_syscalls(&mut loader)?;  // Must be before load!
```

### Issue: "Out of Memory" on Small Allocations

**Symptom**: Allocation fails even with plenty of heap left

**Cause**: Heap size in VM executor doesn't match contract expectations

**Fix**: Check `HEAP_SIZE` constant in `tos/daemon/src/tako_integration/executor.rs` matches your needs (default: 32 KB)

### Issue: Contract Panics on Allocation

**Symptom**: Panic handler triggered during Vec::push or similar

**Cause**: OOM or alignment issues

**Debug**:
1. Add logging before allocations:
   ```rust
   let (used, remaining) = TosAllocator::usage();
   log_u64(used as u64, remaining as u64, 0, 0, 0);
   ```
2. Verify heap size is sufficient
3. Check for memory leaks (allocate without dropping)

## Technical Details

### Why Not Use Solana's Approach Directly?

Solana uses `HEAP_START_ADDRESS` from `solana_program_entrypoint`:

```rust
// Solana's approach
extern "C" {
    static HEAP_START_ADDRESS: usize;  // Provided by linker
}
```

**Why we don't**: This requires:
1. Custom linker scripts
2. Toolchain modifications to inject symbols
3. More complex build process

Our syscall approach is:
- ✅ Simpler (no toolchain changes)
- ✅ More flexible (VM can change heap location)
- ✅ Minimal overhead (1 CU)
- ✅ Solves the 64-bit constant problem

### eBPF 64-bit Constant Problem

eBPF instructions use 32-bit immediate values. Large constants like `0x300000000` require special `lddw` (load double word) instructions:

```
lddw r1, 0x300000000  # Load 64-bit constant
```

Compilers don't always generate `lddw` correctly for hardcoded constants, leading to:
- Wrong addresses being used
- Stack/heap access violations
- Unpredictable behavior

**Our solution**: Don't embed the constant in bytecode. Get it from VM via syscall.

## Design Rationale

### Compared to Option A (Writable .data sections)

**Option A** tried to use writable `.data` sections for global state:
- ❌ Requires eBPF loader changes
- ❌ Same 64-bit constant problem (for initialization)
- ❌ More complex implementation

**Our approach** (Option B + Syscall):
- ✅ No eBPF loader changes
- ✅ Avoids 64-bit constants entirely
- ✅ Simple and maintainable

### Compared to Other Allocation Strategies

| Strategy | Pros | Cons |
|----------|------|------|
| **Bump (ours)** | Simple, fast | No reuse |
| **Free-list** | Memory reuse | Complex, slower |
| **Slab** | Good for fixed sizes | Complex setup |

**Why bump?** Contracts are short-lived. The simplicity and speed of bump allocation outweigh the lack of reuse.

## Contributing

Contributions welcome! Please ensure:
1. Code compiles with `cargo check`
2. Tests pass: `cargo test`
3. Follow existing code style
4. Add tests for new features

## Documentation

- `README.md` - This file (getting started, usage)
- `TESTING_PLAN.md` - End-to-end testing procedures
- `PERFORMANCE_ANALYSIS.md` - Performance analysis and optimization guide
- `examples/` - Working example contracts

## License

Apache-2.0

## Repository

https://github.com/tos-network/tos-alloc
