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
    turiya::hlt_loop();
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

    fn stack_overflow() {
        stack_overflow(); // for each recursion, the return address is pushed
    }

    // trigger a stack overflow
    // stack_overflow();

    // invoke a breakpoint exception
    // x86_64::instructions::interrupts::int3();

    // trigger a page fault
    // unsafe {
    //     // the address 0xdeadbeef is never mapped i.e. invalid address
    //     *(0xdeadbeef as *mut u64) = 42;
    // };

    // trigger a general protection fault
    // let ptr = 0x20426c as *mut u8;
    // unsafe { let x = *ptr; } // works because we are trying to read from a data page
    // println!("read worked");

    // // write to a code page
    // unsafe { *ptr = 42; } // gives a exception because we are trying to write to a code page
    // println!("write worked");

    use x86_64::registers::control::Cr3;

    let (level_4_page_table, _) = Cr3::read();
    println!("Level 4 page table at: {:?}", level_4_page_table.start_address());

    #[cfg(test)]
    test_main();

    println!("It did not crash!");

    // use the hlt_loop to halt the CPU instead of infinite loop
    // this is because the CPU will keep running the loop and consume power
    turiya::hlt_loop();
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}