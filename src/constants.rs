//! Memory layout constants for TOS TAKO VM
//!
//! These must match the VM-side configuration

/// Heap memory starts at this address in TAKO VM
/// Same as tos_tbpf::ebpf::MM_HEAP_START
pub const HEAP_START: usize = 0x300000000;

/// Default heap size (32 KB)
pub const DEFAULT_HEAP_SIZE: usize = 32 * 1024;

/// Maximum heap size (256 KB, Solana compatible)
pub const MAX_HEAP_SIZE: usize = 256 * 1024;
