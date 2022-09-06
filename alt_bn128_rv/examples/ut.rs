#![no_std]
#![no_main]
#![feature(lang_items)]

use core::arch::asm;

fn exit(code: i8) -> ! {
    unsafe {
        asm!("mv a0, {0}",
             "li a7, 93",
             "ecall",
             in(reg) code,
        )
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
fn abort() -> ! {
    panic!("abort!")
}

#[no_mangle]
fn _start() -> ! {
    alt_bn128_rv::ethereum::ut::test_alt_bn128_add();
    alt_bn128_rv::ethereum::ut::test_alt_bn128_mul();
    alt_bn128_rv::ethereum::ut::test_alt_bn128_pairing();
    exit(0)
}
