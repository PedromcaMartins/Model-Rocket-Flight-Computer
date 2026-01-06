// make `std` available when testing
#![cfg_attr(not(test), no_std)]
#![deny(unsafe_code)]
#![deny(unused_must_use)]

#![allow(async_fn_in_trait)]

// Crate used for single-threaded
#![allow(clippy::future_not_send)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::unused_async)]

#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

pub(crate) mod core;
pub(crate) mod config;
pub(crate) mod events;
pub(crate) mod sync;

pub mod interfaces;
pub mod tasks;

pub use embassy_sync;
