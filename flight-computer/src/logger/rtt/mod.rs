mod channel;

use core::sync::atomic::AtomicUsize;

use channel::Channel;

/// RTT buffer size (default: 1024).
///
/// Can be customized by setting the `DEFMT_RTT_BUFFER_SIZE` environment variable.
/// Use a power of 2 for best performance.
const BUF_SIZE: usize = 1024;

const MODE_MASK: usize = 0b11;
/// Block the application if the RTT buffer is full, wait for the host to read data.
const MODE_BLOCK_IF_FULL: usize = 2;
/// Don't block if the RTT buffer is full. Truncate data to output as much as fits.
const MODE_NON_BLOCKING_TRIM: usize = 1;

pub fn do_write(bytes: &[u8]) {
    unsafe { handle().write_all(bytes) }
}

#[repr(C)]
struct Header {
    id: [u8; 16],
    max_up_channels: usize,
    max_down_channels: usize,
    up_channel: Channel,
}

// make sure we only get shared references to the header/channel (avoid UB)
/// # Safety
/// `Channel` API is not re-entrant; this handle should not be held from different execution
/// contexts (e.g. thread-mode, interrupt context)
pub unsafe fn handle() -> &'static Channel {
    // NOTE the `rtt-target` API is too permissive. It allows writing arbitrary data to any
    // channel (`set_print_channel` + `rprint*`) and that can corrupt defmt log frames.
    // So we declare the RTT control block here and make it impossible to use `rtt-target` together
    // with this crate.
    #[no_mangle]
    static mut _SEGGER_RTT: Header = Header {
        id: *b"SEGGER RTT\0\0\0\0\0\0",
        max_up_channels: 1,
        max_down_channels: 0,
        up_channel: Channel {
            name: &NAME as *const _ as *const u8,
            #[allow(static_mut_refs)]
            buffer: unsafe { BUFFER.as_mut_ptr() },
            size: BUF_SIZE,
            write: AtomicUsize::new(0),
            read: AtomicUsize::new(0),
            flags: AtomicUsize::new(MODE_NON_BLOCKING_TRIM),
        },
    };

    #[cfg_attr(target_os = "macos", link_section = ".uninit,defmt-rtt.BUFFER")]
    #[cfg_attr(not(target_os = "macos"), link_section = ".uninit.defmt-rtt.BUFFER")]
    static mut BUFFER: [u8; BUF_SIZE] = [0; BUF_SIZE];

    // Place NAME in data section, so the whole RTT header can be read from RAM.
    // This is useful if flash access gets disabled by the firmware at runtime.
    #[link_section = ".data"]
    static NAME: [u8; 6] = *b"defmt\0";

    unsafe { &*core::ptr::addr_of!(_SEGGER_RTT.up_channel) }
}
