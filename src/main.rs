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
use bootloader::{BootInfo, entry_point};
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};
use turiya::task::{Task, simple_executor, keyboard};

extern crate alloc;

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
// #[no_mangle]
// _start is default entrypoint for most systems
// pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {

// entry_point macro is used to define the entry point and we don't need no_mangle _start anymore
// this macro is provided by bootloader crate  and the advantage is 
// that it provides a function signature that is compatible with the bootloader
entry_point!(kernel_main);
// boot_info is a struct that contains information about the system
// &'static is a lifetime specifier, which means the reference is valid for the entire program
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello World{}", "!");
    
    turiya::init(); 

    // fn stack_overflow() {
    //     stack_overflow(); // for each recursion, the return address is pushed
    // }

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

    use x86_64::{structures::paging::Page, VirtAddr};
    use turiya::{memory, allocator};

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { 
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    
    // map an unused page
    let page = Page::containing_address(VirtAddr::new(0));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // write the string `New!` to the screen through the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    // allocate a number on the heap
    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);

    // create a dynamically sized vector
    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    // create a reference counted vector -> will be freed when count reaches 0
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    println!("reference count is {} now", Rc::strong_count(&cloned_reference));

    let mut executor = simple_executor::SimpleExecutor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();

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

// async function that returns a future
async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}

async fn async_number() -> u32 {
    42
}