const COM1: u16 = 0x3F8;

pub unsafe fn init_serial() {
    unsafe {
        outb(COM1 + 1, 0x00); // Disable all interrupts
        outb(COM1 + 3, 0x80); // Enable DLAB (set baud rate divisor)
        outb(COM1 + 0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
        outb(COM1 + 1, 0x00); //                  (hi byte)
        outb(COM1 + 3, 0x03); // 8 bits, no parity, one stop bit
        outb(COM1 + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
        outb(COM1 + 4, 0x0B); // IRQs enabled, RTS/DSR set
    }
}

/// Check if the serial port is ready to send
fn is_transmit_empty() -> bool {
    unsafe { inb(COM1 + 5) & 0x20 != 0 }
}

/// Send a single byte to the serial port
pub fn send(byte: u8) {
    unsafe {
        while !is_transmit_empty() {}
        outb(COM1, byte);
    }
}

/// Print a string to the serial port
pub fn print(s: &str) {
    for byte in s.bytes() {
        send(byte);
    }
}

use core::fmt;

use crate::io::{inb, outb};

pub struct SerialWriter;

impl fmt::Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        print(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let _ = write!(&mut $crate::serial::SerialWriter, $($arg)*);
    });
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}
