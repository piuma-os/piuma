use core::{arch::asm, panic::PanicInfo};

use crate::{halt, serial_println};

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
