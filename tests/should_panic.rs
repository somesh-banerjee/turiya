#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use turiya::{exit_qemu, QemuExitCode, serial_print, serial_println};


// this should panic only works for single test cases 
// so we dont need runner for this
// directly use in main
#[no_mangle]
pub extern "C" fn _start() -> ! {
    should_fail(); // this should panic and go to panic handler
    serial_println!("[test did not panic]");
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

fn should_fail() {
    serial_print!("should_panic::should_fail...\t");
    assert_eq!(0, 1);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}
