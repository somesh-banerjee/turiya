use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use crate::{gdt, print, println, hlt_loop};

use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;

// Initialize the Programmable Interrupt Controller (PIC) once
// setting the offsets for the pic to range from 32 to 47
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> = 
// unsafe because wrong offsets can cause undefined behavior
// so we use spinlock to ensure safe access using locks
    spin::Mutex::new(unsafe {
        ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
    });

// lazy_static is used to initialize the IDT only once
// and then use it whenever needed
// w/o lazy_static, we have to use mut static and unsafe block prone to data races
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        // idt implements Index trait so we can use it as an array
        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(keyboard_interrupt_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);

}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    print!(".");
    // signal end of interrupt to the PIC
    // because interrupt controller expects an signal to know that the interrupt is handled
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: InterruptStackFrame)
{
    use x86_64::instructions::port::Port;
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;

    // lazy_static is used to initialize the keyboard only once
    // protected by a spinlock to ensure safe access
    lazy_static! {
        // keyboard is a Mutex because it is shared between multiple interrupts
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = 
            // scancode set 1 is the default scancode set for most keyboards
            // US104Key is the layout for a US keyboard with 104 keys
            Mutex::new(Keyboard::new(ScancodeSet1::new(),
            // hamdle control is used to handle control characters
            // we ignore them here and treat them as normal characters
                layouts::Us104Key, HandleControl::Ignore)
            );
    }

    // on each interrupt, lock the keyboard, read the scancode and process it
    let mut keyboard = KEYBOARD.lock();

    // read scancode from the keyboard port
    // 0x60 is the port number for the keyboard
    let mut port = Port::new(0x60);
    // read scancode from the keyboard port is important
    // otherwise the keyboard will not work next time
    let scancode: u8 = unsafe { port.read() };
    
    // // get the key from the scancode using a match statement
    // if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        //     // add_byte translates scancodes into key events
        //     // key events have detailed information about the key & if pressed/released
        //     if let Some(key) = keyboard.process_keyevent(key_event) {
            //         // process_keyevent translates key events into characters if possible
    //         match key {
    //             DecodedKey::Unicode(character) => print!("{}", character),
    //             DecodedKey::RawKey(key) => print!("{:?}", key),
    //         }
    //     }
    // }
    
    // the above code is replaced by the following code
    // which is more efficient and less error-prone
    // it uses async/await to handle the keyboard input
    crate::task::keyboard::add_scancode(scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard, // default value is +1 of previous so no need to specify
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

