// no std since this will run as os
#![no_std]
// disable all Rust-level entry points
#![no_main]

use core::panic::PanicInfo;

/// This function is called on panic. originally found in std but we are using no_std env
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static HELLO: &[u8] = b"Hello World!";

// overwriting the entrypoint
// no_mangle to disable cryptic function naming
#[no_mangle]
// _start is default entrypoint for most systems
pub extern "C" fn _start() -> ! {

    let vga_buffer = 0xb8000 as *mut u8;

    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }

    loop {}
}