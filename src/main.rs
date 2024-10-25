// no std since this will run as os
#![no_std]
// disable all Rust-level entry points
#![no_main]
// enable custom test framework
#![feature(custom_test_frameworks)]
// reexport the test harness to use our test_runner instead of the default one
#![test_runner(turiya::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use turiya::println;

#[cfg(not(test))]
/// This function is called on panic. originally found in std but we are using no_std env
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    turiya::test_panic_handler(info)
}

// overwriting the entrypoint
// no_mangle to disable cryptic function naming
#[no_mangle]
// _start is default entrypoint for most systems
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");
    
    turiya::init(); 

    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();

    #[cfg(test)]
    test_main();

    println!("It did not crash!");

    loop {}
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}