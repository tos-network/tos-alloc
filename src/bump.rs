//! Heap-based bump allocator implementation (Solana-compatible)
//!
//! Stores allocator state at the beginning of the heap region instead of using
//! global statics. This avoids the need for writable .data sections in eBPF.
//!
//! # Design (Based on Solana's custom_heap)
//!
//! ```text
//! Heap Layout:
//! ┌────────────────────────────────────┐
//! │ Position Pointer (8 bytes)         │  ← Current allocation position
//! ├────────────────────────────────────┤
//! │ User allocations                   │  ← Vec, Box, etc.
//! │ ...                                │
//! └────────────────────────────────────┘
//! ```
//!
//! The position pointer is stored at HEAP_START and tracks the next free address.
//! On first allocation, it's initialized to HEAP_START + size_of::<usize>().

use core::alloc::{GlobalAlloc, Layout};
use core::mem::size_of;
use core::ptr::null_mut;

/// Syscall to get heap region information from the VM
///
/// This avoids hardcoded 64-bit constants which are problematic in eBPF bytecode.
unsafe fn get_heap_region_syscall() -> (usize, usize) {
    let mut heap_start: u64 = 0;
    let mut heap_size: u64 = 0;

    // Call the tos_get_heap_region syscall
    extern "C" {
        fn tos_get_heap_region(heap_start_ptr: *mut u64, heap_size_ptr: *mut u64) -> u64;
    }

    tos_get_heap_region(&mut heap_start as *mut u64, &mut heap_size as *mut u64);

    (heap_start as usize, heap_size as usize)
}

/// Heap-based bump allocator for TAKO VM
///
/// **Solana-Compatible Design**: Stores state on heap instead of in globals.
///
/// # Usage
///
/// ```rust,no_run
/// use tos_alloc::BumpAllocator;
///
/// #[global_allocator]
/// static ALLOCATOR: BumpAllocator = BumpAllocator::new();
///
/// // No manual init needed! Allocator initializes on first use.
/// let v = vec![1, 2, 3];
/// ```
///
/// # Performance
///
/// - First allocation: ~10-15 CU (initialization)
/// - Subsequent allocations: ~5-10 CU
pub struct BumpAllocator;

impl Default for BumpAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl BumpAllocator {
    /// Create a new allocator instance
    ///
    /// This is a zero-sized type, so creating instances has no cost.
    pub const fn new() -> Self {
        Self
    }

    /// Get heap usage statistics (for debugging)
    ///
    /// Returns (used_bytes, remaining_bytes)
    pub fn usage() -> (usize, usize) {
        unsafe {
            let (heap_start, heap_size) = get_heap_region_syscall();

            // SECURITY: Validate syscall succeeded
            if heap_start == 0 || heap_size == 0 {
                return (0, 0); // Syscall failed, return safe values
            }

            let pos_ptr = heap_start as *mut usize;
            let heap_end = heap_start.saturating_add(heap_size);
            let heap_bottom = heap_start.saturating_add(size_of::<usize>());

            let pos = *pos_ptr;
            if pos == 0 {
                // Not initialized yet
                (0, heap_size - size_of::<usize>())
            } else {
                // SECURITY: Validate pos is within valid heap range
                if pos < heap_bottom || pos > heap_end {
                    // Position pointer was corrupted, return safe values
                    return (0, heap_size - size_of::<usize>());
                }

                let used = pos - heap_bottom;
                let remaining = heap_end - pos;
                (used, remaining)
            }
        }
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Based on Solana's custom_heap implementation
        // See: agave/programs/sbf/rust/custom_heap/src/lib.rs

        // Get heap region from syscall to avoid hardcoded constant issues
        let (heap_start, heap_size) = get_heap_region_syscall();

        // SECURITY: Validate syscall returned valid heap info
        if heap_start == 0 || heap_size == 0 {
            return null_mut(); // Syscall failed
        }

        let pos_ptr = heap_start as *mut usize;
        let heap_end = heap_start.saturating_add(heap_size);
        let heap_bottom = heap_start.saturating_add(size_of::<usize>());

        let mut pos = *pos_ptr;
        if pos == 0 {
            // First allocation - initialize position
            pos = heap_bottom;
        }

        // SECURITY: Validate pos is within valid heap range to prevent pointer forgery
        // A contract could write an arbitrary address to the position pointer to
        // access memory outside the heap region. We must validate and clamp it.
        if pos < heap_bottom || pos > heap_end {
            // Position pointer was corrupted/forged, reset to safe value
            pos = heap_bottom;
            *pos_ptr = pos;
        }

        // Align the position
        let align = layout.align();
        pos = pos.saturating_add(align - 1) & !(align - 1);

        // Calculate new position
        let new_pos = pos.saturating_add(layout.size());

        // SECURITY: Check if we have enough space AND new_pos is valid
        // Also check for overflow where new_pos wraps around
        if new_pos > heap_end || new_pos < pos {
            // Out of memory or overflow
            return null_mut();
        }

        // Update position pointer
        *pos_ptr = new_pos;

        pos as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator never deallocates individual allocations
        // Memory is reclaimed when contract execution ends
    }
}
