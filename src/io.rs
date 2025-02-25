use core::arch::asm;

#[inline(always)]
pub unsafe fn outb(port: u16, value: u8) {
    unsafe {
        asm!("out dx, al",
            in("dx") port,
            in("al") value,
            options(nomem, nostack, preserves_flags)
        );
    }
}

#[inline(always)]
pub unsafe fn inb(port: u16) -> u8 {
    unsafe {
        let value: u8;
        asm!("in al, dx",
            out("al") value,
            in("dx") port,
            options(nomem, nostack, preserves_flags)
        );
        value
    }
}
