#![no_std]

#![cfg_attr(test, no_main)]

// enable custom test framework
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]

// reexport the test harness to use our test_runner 
// instead of the default one
#![reexport_test_harness_main = "test_main"]
// to use x86-interrupt calling convention
#![feature(abi_x86_interrupt)]

pub mod serial;
pub mod vga_buffer;
pub mod gdt;
pub mod memory;
pub mod allocator;

use core::panic::PanicInfo;
#[cfg(test)]
use bootloader::{entry_point, BootInfo};

extern crate alloc;
pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        // print the name of the test function
        serial_print!("{}...\t", core::any::type_name::<T>());
        // run the test
        self();
        // if the test passes, print [ok]
        serial_println!("[ok]");
    }
}

// no cf(test) since we want to make this public
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    // exit qemu when tests are done
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

#[cfg(test)]
entry_point!(test_kernel_main);
/// Entry point for `cargo test`
#[cfg(test)]
// #[no_mangle] not required since we are using entry_point macro
// need a start here because lib.rs is tested independently
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

// exit qemu
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub mod interrupts;

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe {
        interrupts::PICS.lock().initialize();
    }
    // enable interrupts i.e. cpu listens to interrupt controller
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}