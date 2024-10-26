use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use lazy_static::lazy_static;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};

// 0th Interrupt Stack Table (IST) entry is used for handling double faults
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

// Initialize the Task State Segment (TSS) once, which cannot be done at compile time
lazy_static! {
    static ref TSS: TaskStateSegment = {
        // Create a new Task State Segment
        let mut tss = TaskStateSegment::new();
        
        // Allocate a dedicated stack for the double fault handler
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            // Define the stack size (5 pages, each 4096 bytes)
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            // Get the starting address of the stack and calculate the end
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end // Set the end of the stack in the IST entry
        };
        tss
    };
}

// Define the Global Descriptor Table (GDT) and necessary segment selectors
lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        // Initialize a new Global Descriptor Table
        let mut gdt = GlobalDescriptorTable::new();
        
        // Add a kernel code segment descriptor to the GDT
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        
        // Add the TSS segment descriptor to the GDT
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        
        // Return the GDT with the associated selectors for code and TSS segments
        (gdt, Selectors { code_selector, tss_selector })
    };
}

// A struct to hold the segment selectors for code and TSS segments
struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}   

/// Initializes the GDT and loads the TSS by setting the appropriate segment registers
pub fn init() {
    use x86_64::instructions::segmentation::{CS, Segment};
    use x86_64::instructions::tables::load_tss;

    // Load the GDT into the CPU's GDTR register
    GDT.0.load();

    unsafe {
        // Set the code segment register (CS) to the GDT's code segment selector
        CS::set_reg(GDT.1.code_selector);
        
        // Load the Task State Segment (TSS) by setting the TSS segment selector
        load_tss(GDT.1.tss_selector);
    }
}
