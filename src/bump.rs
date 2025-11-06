//! Bump allocator implementation
//!
//! A simple, lock-free allocator that allocates memory sequentially
//! without ever freeing it. Optimized for short-lived contract executions.

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::{HEAP_START, DEFAULT_HEAP_SIZE, MAX_HEAP_SIZE};

/// Lock-free bump allocator for TAKO VM
///
/// Allocates memory by atomically incrementing a pointer.
/// Never deallocates individual allocations.
///
/// # Performance
///
/// - Best case (no contention): ~5-10 CU per allocation
/// - Worst case (high contention): ~30-90 CU per allocation
/// - Compare to syscall: ~100 CU per allocation
///
/// # Safety
///
/// Safe when used in TAKO VM with properly mapped heap region.
/// Will return null_mut() when out of memory instead of panicking.
pub struct BumpAllocator {
    next: AtomicUsize,
    end: usize,
}

impl BumpAllocator {
    /// Create allocator with default heap size (32 KB)
    ///
    /// Must match VM-side heap configuration
    pub const fn new() -> Self {
        Self::with_size(DEFAULT_HEAP_SIZE)
    }

    /// Create allocator with custom heap size
    ///
    /// # Panics
    ///
    /// Panics at compile time if heap_size > MAX_HEAP_SIZE
    pub const fn with_size(heap_size: usize) -> Self {
        assert!(
            heap_size <= MAX_HEAP_SIZE,
            "Heap size exceeds maximum (256 KB)"
        );
        assert!(
            heap_size >= 4096,
            "Heap size too small (minimum 4 KB)"
        );

        Self {
            next: AtomicUsize::new(HEAP_START),
            end: HEAP_START + heap_size,
        }
    }

    /// Get current heap usage in bytes
    pub fn used(&self) -> usize {
        self.next.load(Ordering::Relaxed).saturating_sub(HEAP_START)
    }

    /// Get remaining heap space in bytes
    pub fn remaining(&self) -> usize {
        self.end.saturating_sub(self.next.load(Ordering::Relaxed))
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        // Alignment must be power of 2 (guaranteed by Layout)
        debug_assert!(align.is_power_of_two());

        loop {
            let current = self.next.load(Ordering::Relaxed);

            // Saturating operations prevent overflow
            let aligned = current.saturating_add(align - 1) & !(align - 1);
            let new_next = aligned.saturating_add(size);

            // Check if we have enough space
            if new_next > self.end {
                // Out of memory - return null instead of panicking
                return null_mut();
            }

            // Try to atomically update the pointer
            match self.next.compare_exchange(
                current,
                new_next,
                Ordering::Release,  // Success ordering
                Ordering::Relaxed,  // Failure ordering
            ) {
                Ok(_) => {
                    // Successfully allocated
                    return aligned as *mut u8;
                }
                Err(_) => {
                    // CAS failed due to contention, retry
                    continue;
                }
            }
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator never deallocates individual allocations
        // Memory is reclaimed when contract execution ends
    }
}
