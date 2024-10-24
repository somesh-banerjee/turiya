// since its a separate module, we need to import everything we need
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(turiya::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use turiya::{println, serial_print, serial_println};

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    turiya::test_panic_handler(info)
}

#[test_case]
fn test_println() {
    println!("test_println output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_serial_print() {
    serial_print!("test_serial_print output");
}

#[test_case]
fn test_serial_println() {
    serial_println!("test_serial_println output");
}

// #[test_case]
// fn test_serial_println_many() {
//     for _ in 0..200 {
//         serial_println!("test_serial_println_many output");
//     }
// }