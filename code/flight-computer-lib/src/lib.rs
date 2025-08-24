// make `std` available when testing
#![cfg_attr(not(test), no_std)]
#![deny(unsafe_code)]
// Crate used for single-threaded
#![allow(clippy::future_not_send)]
#![deny(unused_must_use)]
#![allow(clippy::cast_precision_loss)]

pub mod model;
pub mod tasks;
pub mod device;
