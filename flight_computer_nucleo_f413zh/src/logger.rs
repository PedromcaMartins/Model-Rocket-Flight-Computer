use core::sync::atomic::{AtomicBool, Ordering};

use defmt::{global_logger, Encoder, Logger};
use static_cell::ConstStaticCell;
use crate::{drivers::rtt::{do_write, handle}, tasks::telemetry::LoggerPipeWriter};

#[global_logger]
struct MyLogger;

/// Global logger lock.
static TAKEN: AtomicBool = AtomicBool::new(false);
static mut CS_RESTORE: critical_section::RestoreState = critical_section::RestoreState::invalid();

static mut LOG_RTT: Option<Encoder> = None;
static mut LOG_PIPE_WRITER: Option<LoggerPipeWriter> = None;

#[allow(static_mut_refs)]
pub fn init_logger_rtt() {
    unsafe {
        critical_section::with(|_| {
            defmt::assert!(
                LOG_RTT.is_none(),
                "Tried to init logger rtt when already initialized."
            );
            LOG_RTT = Some(Encoder::new())
        });
    }
}

#[allow(static_mut_refs)]
pub fn init_async_logger(writer: LoggerPipeWriter) {
    unsafe {
        critical_section::with(|_| {
            defmt::assert!(
                LOG_PIPE_WRITER.is_none(),
                "Tried to assign a logger pipe writer when one was already assigned."
            );
            LOG_PIPE_WRITER = Some(writer)
        });
    }
}

unsafe impl Logger for MyLogger {
    fn acquire() {
        // safety: Must be paired with corresponding call to release(), see below
        let restore = unsafe { critical_section::acquire() };

        // safety: accessing the atomic without CAS is OK because we have acquired a critical section.
        if TAKEN.load(Ordering::Relaxed) {
            panic!("defmt logger taken reentrantly")
        }

        // safety: accessing the atomic without CAS is OK because we have acquired a critical section.
        TAKEN.store(true, Ordering::Relaxed);

        // safety: accessing the `static mut` is OK because we have acquired a critical section.
        unsafe { CS_RESTORE = restore };

        // safety: accessing the `static mut` is OK because we have acquired a critical section.
        unsafe {
            if let Some(ref mut rtt) = LOG_RTT {
                rtt.start_frame(do_write);
            }
        }
    }

    unsafe fn release() {        
        // safety: accessing the `static mut` is OK because we have acquired a critical section.
        unsafe {
            if let Some(ref mut rtt) = LOG_RTT {
                rtt.end_frame(do_write);
            }
        }

        // safety: accessing the atomic without CAS is OK because we have acquired a critical section.
        TAKEN.store(false, Ordering::Relaxed);

        // safety: accessing the `static mut` is OK because we have acquired a critical section.
        let restore = unsafe { CS_RESTORE };

        // safety: Must be paired with corresponding call to acquire(), see above
        unsafe {
            critical_section::release(restore);
        }
    }

    unsafe fn write(mut bytes: &[u8]) {
        // safety: accessing the `static mut` is OK because we have acquired a critical section.
        unsafe {
            if let Some(ref mut rtt) = LOG_RTT {
                rtt.write(bytes, do_write);
            }
        }

        unsafe {
            if let Some(ref mut pipe_writer) = LOG_PIPE_WRITER {
                while !bytes.is_empty() {
                    match pipe_writer.try_write(bytes) {
                        Ok(written) => bytes = &bytes[written..],
                        Err(_) => {
                            // TODO: silent diagnostics
                            // panic!();
                            break
                        },
                    }
                }
            }
        }
    }

    unsafe fn flush() {
        // safety: accessing the `&'static _` is OK because we have acquired a critical section.
        handle().flush();
    }
}
