#![no_std]
#![no_main]
#![feature(asm)]
#![feature(lang_items)]

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

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[no_mangle]
pub fn abort() -> ! {
    panic!("abort!")
}

mod ut_common;

#[no_mangle]
pub fn _start() -> ! {
    ut_common::test_alt_bn128_add();
    ut_common::test_alt_bn128_mul();
    ut_common::test_alt_bn128_pairing();
    exit(0)
}
