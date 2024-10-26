#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use core::panic::PanicInfo;
use turiya::{serial_print, serial_println, exit_qemu, QemuExitCode};

// Define an Interrupt Descriptor Table (IDT) with a double fault handler
lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        
        // Set the double fault handler and specify the stack to use (from the GDT)
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(turiya::gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}

/// Initializes and loads the custom test IDT
pub fn init_test_idt() {
    TEST_IDT.load();
}

/// Entry point for the kernel test, marked as no-mangle to prevent Rust's name mangling.
/// The function name `_start` is recognized by the linker as the entry point.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");

    // Initialize the Global Descriptor Table (GDT) and test IDT
    turiya::gdt::init();
    init_test_idt();

    // Trigger a stack overflow to test the double fault handler
    stack_overflow();

    panic!("Execution continued after stack overflow");
}

/// Function to cause a stack overflow by calling itself recursively.
/// The use of `volatile` prevents compiler optimizations like tail recursion elimination.
#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow(); // Each recursive call pushes a return address onto the stack
    volatile::Volatile::new(0).read(); // Prevents tail-call optimization by reading a volatile value
}

/// Panic handler function called when a panic occurs.
/// Delegates to a custom panic handler from the `turiya` module.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    turiya::test_panic_handler(info)
}

/// Custom double fault handler for the IDT.
/// Prints `[ok]` to the serial output and exits QEMU with a success code.
extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}
