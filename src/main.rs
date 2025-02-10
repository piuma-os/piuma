#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(naked_functions)]

use core::arch::{asm, naked_asm};

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

#[no_mangle]
unsafe extern "C" fn kmain() -> ! {
    unsafe {
        init_cpu_features();
    }

    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response() {
        if let Some(framebuffer) = framebuffer_response.framebuffers().get(0) {
            if let Some(mut ctx) = Context::from_framebuffer(framebuffer) {
                let info = INFO_REQUEST.get_response().unwrap();

                let name = info.name();
                let version = info.version();

                ctx.write(b"Piuma\nKernel: 0.0.1\n");
                ctx.write(b"Bootloader: ");
                ctx.write(name.to_bytes_with_nul());
                ctx.write(b" ");
                ctx.write(version.to_bytes_with_nul());
            }
        }
    }

    panic!("Kernel stopped");
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
