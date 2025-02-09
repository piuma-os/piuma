#![no_std]
#![no_main]
#![feature(naked_functions)]

use core::arch::{asm, naked_asm};
use core::ptr::null_mut;

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
            let framebuffer = framebuffer.as_raw();

            unsafe {
                let ctx = flanterm::sys::flanterm_fb_init(
                    None,
                    None,
                    framebuffer.address.cast(),
                    framebuffer.width as _,
                    framebuffer.height as _,
                    framebuffer.pitch as _,
                    framebuffer.red_mask_size,
                    framebuffer.red_mask_shift,
                    framebuffer.green_mask_size,
                    framebuffer.green_mask_shift,
                    framebuffer.blue_mask_size,
                    framebuffer.blue_mask_shift,
                    null_mut(),
                    null_mut(),
                    null_mut(),
                    null_mut(),
                    null_mut(),
                    null_mut(),
                    null_mut(),
                    null_mut(),
                    0,
                    0,
                    1,
                    0,
                    0,
                    10,
                );

                let splash = "Piuma\nKernel: 0.0.1\n";
                let booted = "Bootloader: ";
                let separator = " ";

                let info = INFO_REQUEST.get_response().unwrap();

                let name = info.name();
                let version = info.version();

                flanterm::sys::flanterm_write(ctx, splash.as_ptr().cast(), splash.len());
                flanterm::sys::flanterm_write(ctx, booted.as_ptr().cast(), booted.len());
                flanterm::sys::flanterm_write(ctx, name.as_ptr(), name.to_bytes_with_nul().len());
                flanterm::sys::flanterm_write(ctx, separator.as_ptr().cast(), separator.len());
                flanterm::sys::flanterm_write(
                    ctx,
                    version.as_ptr(),
                    version.to_bytes_with_nul().len(),
                );
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
