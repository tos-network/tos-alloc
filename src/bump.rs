//! Solana-compatible bump allocator for TOS TAKO VM
//!
//! This implementation matches Solana's allocator exactly to ensure compatibility.

use core::alloc::{GlobalAlloc, Layout};
use core::mem::size_of;
use core::ptr::null_mut;

/// Heap start address (matches Solana's MM_HEAP_START)
pub const HEAP_START_ADDRESS: usize = 0x300000000;

/// Default heap size (32 KB, matches Solana)
pub const HEAP_LENGTH: usize = 32 * 1024;

/// Solana-compatible bump allocator
///
/// **Key design decisions (matching Solana exactly)**:
/// 1. **Hardcoded heap address** (0x300000000) - No syscalls needed
/// 2. **Allocates from high to low** - Position starts at heap_top, moves down
/// 3. **Position pointer at heap start** - First 8 bytes store current position
///
/// # Heap Layout
///
/// ```text
/// 0x300000000: Position Pointer (8 bytes) ← Stores current allocation position
/// 0x300000008: User allocations start here
/// ...          Allocations grow upward
/// 0x300008000: Heap top (initial position value)
/// ```
///
/// # Usage
///
/// ```rust,no_run
/// use tos_alloc::TosAllocator;
///
/// #[global_allocator]
/// static ALLOCATOR: TosAllocator = TosAllocator::new();
///
/// let v = vec![1, 2, 3];  // Works immediately
/// ```
pub struct BumpAllocator {
    pub start: usize,
    pub len: usize,
}

impl BumpAllocator {
    /// Create a new allocator with default heap configuration
    pub const fn new() -> Self {
        Self {
            start: HEAP_START_ADDRESS,
            len: HEAP_LENGTH,
        }
    }

    /// Get heap usage statistics
    ///
    /// Returns (used_bytes, remaining_bytes)
    pub fn usage() -> (usize, usize) {
        unsafe {
            let allocator = Self::new();
            let pos_ptr = allocator.start as *mut usize;
            let pos = *pos_ptr;

            if pos == 0 {
                // Not initialized yet
                (0, allocator.len - size_of::<usize>())
            } else {
                // Position moves from top to bottom
                let heap_top = allocator.start + allocator.len;
                let used = heap_top - pos;
                let remaining = pos - (allocator.start + size_of::<usize>());
                (used, remaining)
            }
        }
    }
}

impl Default for BumpAllocator {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Solana's bump allocator implementation
        // Source: agave/sdk/program/src/entrypoint.rs

        let pos_ptr = self.start as *mut usize;
        let mut pos = *pos_ptr;

        if pos == 0 {
            // First allocation: start from heap top
            pos = self.start + self.len;
        }

        // Allocate from high to low (move position downward)
        pos = pos.saturating_sub(layout.size());

        // Align the position
        pos &= !(layout.align().wrapping_sub(1));

        // Check bounds
        if pos < self.start + size_of::<*mut u8>() {
            return null_mut();  // Out of memory
        }

        // Update position pointer
        *pos_ptr = pos;

        pos as *mut u8
    }

    #[inline]
    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {
        // Bump allocator never deallocates
        // Contract memory is reclaimed when execution finishes
    }
}

/// Type alias for compatibility with existing code
#[allow(dead_code)]
pub type TosAllocator = BumpAllocator;
