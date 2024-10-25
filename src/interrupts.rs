use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::println;

use lazy_static::lazy_static;

// lazy_static is used to initialize the IDT only once
// and then use it whenever needed
// w/o lazy_static, we have to use mut static and unsafe block prone to data races
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);

}

#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}