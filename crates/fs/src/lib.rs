#![no_std]

// compile with --crate-type=staticlib
#[cfg(not(feature = "lib"))]
pub mod cffi;
mod virtio;
