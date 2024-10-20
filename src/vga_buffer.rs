/// The vga_buffer module contains the code for writing to the VGA text buffer.

/// disable unused code warning
#[allow(dead_code)]
/// enable copy semantics by deriving the Copy trait for Color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The repr(u8) attribute tells the compiler to represent each enum variant as an u8.
#[repr(u8)]
/// The Color enum represents the 16 different colors that can be displayed in text mode.
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// The ColorCode struct represents a complete color code byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The repr(transparent) attribute tells the compiler to represent ColorCode as a single u8 in memory.
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

/// The ScreenChar struct represents a character in the VGA buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The repr(C) attribute guarantees that the struct's fields are laid out exactly as they would be in C.
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

/// the height and width of the text buffer
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

// use the volatile crate to prevent the compiler from optimizing away writes to the VGA buffer
use volatile::Volatile; 

/// The Buffer struct represents the entire VGA text buffer.
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// The Writer struct represents the state of the VGA text buffer.
/// It keeps track of the current position of the cursor and the color code.
/// The 'static lifetime indicates that the Writer can be stored for the entire duration of the program.
/// the writer always writes to the last line of the buffer and scrolls the buffer when it reaches the end.
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    
    /// The write_string method writes a string to the buffer at the current cursor position.
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            // since rust strings are UTF-8 encoded, we need to handle the case where the byte is not a valid ASCII character
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }

        }
    }

    /// The write_byte method writes a byte to the buffer at the current cursor position.
    pub fn write_byte(&mut self, byte: u8) {
        // match the byte to check if it is a newline character    
        match byte {
            // if it is a newline character, call the new_line method
            b'\n' => self.new_line(),
            // if it is any other byte, write it to the buffer
            byte => {
                // if the current line is full, call the new_line method
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                // get the current row and column position
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                // get the color code
                let color_code = self.color_code;
                // write the byte to the buffer at the current position
                // we have to use write method instead of simply assigning value cause the buffer is volatile
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                // increment the column position
                self.column_position += 1;
            }
        }
    }

    /// The new_line method scrolls the buffer by one line.
    fn new_line(&mut self) {
        // iterate over each row in the buffer
        for row in 1..BUFFER_HEIGHT {
            // iterate over each column in the buffer
            for col in 0..BUFFER_WIDTH {
                // get the character at the current position
                let character = self.buffer.chars[row][col].read();
                // write the character to the row above
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        // clear the last row   
        self.clear_row(BUFFER_HEIGHT - 1);
        // reset the column position to 0   
        self.column_position = 0;
    }

    /// The clear_row method clears a row in the buffer by writing spaces to each column.
    fn clear_row(&mut self, row: usize) {
        // create a blank character with a space character and the current color code
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        // iterate over each column in the row and write the blank character
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

use core::fmt;

// implement the fmt::Write trait for the Writer struct
// this allows us to use the write! macro to write formatted strings to the VGA buffer
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// lazy_static is a crate that provides a macro for declaring lazily evaluated statics in Rust
// this allows us to initialize the WRITER static variable with the Writer struct
// normally static variables are initialized at compile time,
// so we need to use lazy_static to ensure that the initialization is done when first accessed at runtime   
use lazy_static::lazy_static;
// the spin crate provides a Mutex type that can be used to safely share mutable data between threads
// we use Mutex to ensure that the WRITER static variable can be safely accessed from multiple threads
// we use the concept of a spinlock to implement the Mutex type,
// which means that the lock is held by spinning in a loop until it can be acquired
use spin::Mutex;

lazy_static! {
    /// The WRITER static variable provides a global interface for writing to the VGA buffer.
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

/// Like the `print!` macro in the standard library, but prints to the VGA text buffer.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

/// Like the `println!` macro in the standard library, but prints to the VGA text buffer.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Prints the given formatted string to the VGA text buffer through the global `WRITER` instance.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}