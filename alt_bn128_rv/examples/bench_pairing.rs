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
    let inputs = alt_bn128_rv::ethereum::ut::ALT_BN128_PAIRING_CASE[8].0;
    let expect = alt_bn128_rv::ethereum::ut::ALT_BN128_PAIRING_CASE[8].0;
    let mut buf0 = [0x00; 4096];
    let mut buf1 = [0x00; 32];
    alt_bn128_rv::ethereum::ut::hex2bin(inputs, &mut buf0[..]);
    assert!(
        alt_bn128_rv::ethereum::alt_bn128_pairing(&buf0[0..inputs.len() / 2], &mut buf1).is_ok()
    );
    alt_bn128_rv::ethereum::ut::hex2bin(expect, &mut buf0[..]);
    assert_eq!(buf0[0..32], buf1[..]);
    exit(0)
}
