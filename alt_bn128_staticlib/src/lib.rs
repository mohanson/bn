#![no_std]
#![feature(asm)]

pub mod ethereum;

pub fn exit(_: i8) -> ! {
    unsafe {
        asm!("li a7, 93");
        asm!("ecall");
    }
    loop {}
}

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    exit(-128);
}
