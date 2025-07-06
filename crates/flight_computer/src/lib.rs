#![no_std]
#![deny(unsafe_code)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

pub mod components;
pub mod traits;
