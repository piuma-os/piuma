#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(naked_functions)]

use core::{
    arch::{asm, naked_asm},
    panic::PanicInfo,
};

use flanterm::Context;
use limone::{
    requests::{
        bootloader_info::BootloaderInfoRequest, framebuffer::FramebufferRequest, RequestsEndMarker,
        RequestsStartMarker,
    },
    BaseRevision,
};

#[used]
#[link_section = ".limine_requests_start"]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[link_section = ".limine_requests_end"]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

#[used]
#[link_section = ".limine_requests"]
static BASE_REVISION: BaseRevision = BaseRevision::LATEST;

#[used]
#[link_section = ".limine_requests"]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[link_section = ".limine_requests"]
static INFO_REQUEST: BootloaderInfoRequest = BootloaderInfoRequest::new();

/// Initializes basic simds and fpu
#[naked]
unsafe extern "C" fn init_cpu_features() {
    naked_asm!(
        // Load CR0, modify its bits
        "mov rax, cr0",   // Get current CR0 value
        "and ax, 0xFFFB", // Clear EM (bit 2)
        "or ax, 0x2",     // Set MP (bit 1)
        "mov cr0, rax",   // Write back to CR0
        // Load CR4, modify its bits
        "mov rax, cr4",  // Get current CR4 value
        "or ax, 3 << 9", // Set OSFXSR (bit 9) and OSXMMEXCPT (bit 10)
        "mov cr4, rax",  // Write back to CR4
        "ret",           // Return from function
    );
}

const COM1: u16 = 0x3F8;

#[inline(always)]
unsafe fn outb(port: u16, value: u8) {
    asm!("out dx, al",
        in("dx") port,
        in("al") value,
        options(nomem, nostack, preserves_flags)
    );
}

#[inline(always)]
unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!("in al, dx",
        out("al") value,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
    );
    value
}

unsafe fn init_serial() {
    outb(COM1 + 1, 0x00); // Disable all interrupts
    outb(COM1 + 3, 0x80); // Enable DLAB (set baud rate divisor)
    outb(COM1 + 0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
    outb(COM1 + 1, 0x00); //                  (hi byte)
    outb(COM1 + 3, 0x03); // 8 bits, no parity, one stop bit
    outb(COM1 + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
    outb(COM1 + 4, 0x0B); // IRQs enabled, RTS/DSR set
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
        let _ = write!(&mut $crate::SerialWriter, $($arg)*);
    });
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}

#[no_mangle]
unsafe extern "C" fn kmain() -> ! {
    unsafe {
        init_cpu_features();
        init_serial();
    }

    serial_println!("Piuma version 0.0.1");
    serial_println!("Acquiring framebuffer");

    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response() {
        if let Some(framebuffer) = framebuffer_response.framebuffers().get(0) {
            if let Some(mut ctx) = Context::from_framebuffer(framebuffer) {
                serial_println!("Initialized flanterm context");

                let info = INFO_REQUEST.get_response().unwrap();

                let name = info.name();
                let version = info.version();

                ctx.write(b"Piuma version 0.0.1\n");
                ctx.write(b"Bootloader: ");
                ctx.write(name.to_bytes_with_nul());
                ctx.write(b" ");
                ctx.write(version.to_bytes_with_nul());
            }
        }
    }

    halt()
}

fn halt() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Disable interrupts first thing to ensure clean output
    unsafe {
        asm!("cli");
    }

    serial_println!("*** KERNEL PANIC ***");

    serial_println!("\nMessage: {}", info.message());

    if let Some(location) = info.location() {
        serial_println!(
            "Location: {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    }

    halt()
}
