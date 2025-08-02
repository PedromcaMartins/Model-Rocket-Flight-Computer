#![no_std]
#![deny(unsafe_code)]
// Crate used for single-threaded
#![allow(clippy::future_not_send)]
#![deny(unused_must_use)]

pub(crate) mod model;
pub mod tasks;
