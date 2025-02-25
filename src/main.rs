#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(naked_functions)]

use core::arch::{asm, naked_asm};

use flanterm::Context;
use limone::{
    BaseRevision,
    requests::{
        RequestsEndMarker, RequestsStartMarker, bootloader_info::BootloaderInfoRequest,
        framebuffer::FramebufferRequest,
    },
};
use serial::init_serial;

pub mod io;
pub mod panic;
pub mod serial;

#[used]
#[unsafe(link_section = ".limine_requests_start")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".limine_requests_end")]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
static BASE_REVISION: BaseRevision = BaseRevision::LATEST;

#[used]
#[unsafe(link_section = ".limine_requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".limine_requests")]
static INFO_REQUEST: BootloaderInfoRequest = BootloaderInfoRequest::new();

/// Initializes basic simds and fpu
#[naked]
unsafe extern "C" fn init_cpu_features() {
    unsafe {
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
}

#[unsafe(no_mangle)]
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
