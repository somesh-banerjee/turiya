// no std since this will run as os
#![no_std]
// disable all Rust-level entry points
#![no_main]

// overwriting the entrypoint
// no_mangle to disable cryptic function naming
#[no_mangle]
// _start is default entrypoint for most systems
pub extern "C" fn _start() -> ! {
    loop {}
}

use core::panic::PanicInfo;

/// This function is called on panic. originally found in std but we are using no_std env
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}