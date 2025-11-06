//! TOS Dynamic Memory Allocator
//!
//! Provides a global allocator for TOS TAKO VM contracts, enabling
//! the use of Vec, BTreeMap, Box, and other heap-allocated types.
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
//! static ALLOCATOR: TosAllocator = TosAllocator::new();
//!
//! // Now you can use Vec, BTreeMap, etc.
//! ```

#![no_std]

extern crate alloc;  // ← 必须声明，用于GlobalAlloc

mod constants;
mod bump;

pub use constants::*;
pub use bump::BumpAllocator;

/// Type alias for convenience
pub type TosAllocator = BumpAllocator;
