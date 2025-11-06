//! TOS Dynamic Memory Allocator (Heap-Based, Solana-Compatible)
//!
//! Provides a global allocator for TOS TAKO VM contracts, enabling
//! the use of Vec, BTreeMap, Box, and other heap-allocated types.
//!
//! **Design**: Stores allocator state on the heap instead of in globals,
//! avoiding the need for writable .data sections in eBPF.
//!
//! # Usage
//!
//! ```no_run
//! #![no_std]
//! #![no_main]
//!
//! extern crate alloc;
//! use tos_alloc::TosAllocator;
//!
//! #[global_allocator]
//! static ALLOCATOR: TosAllocator = TosAllocator;
//!
//! #[no_mangle]
//! pub extern "C" fn entrypoint(input: *const u8) -> u64 {
//!     // MUST initialize allocator first!
//!     unsafe { TosAllocator::init(); }
//!
//!     // Now you can use Vec, BTreeMap, etc.
//!     let v = vec![1, 2, 3];
//!     0
//! }
//! ```

#![no_std]

extern crate alloc;

mod bump;
mod constants;

pub use bump::BumpAllocator;
pub use constants::*;

/// Type alias for convenience
pub type TosAllocator = BumpAllocator;
