#![no_std]
#![no_main]
#![feature(asm)]
#![feature(lang_items)]

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
    let inputs = "1c76476f4def4bb94541d57ebba1193381ffa7aa76ada664dd31c16024c43f593034dd2920f673e204fee2811c678745fc819b55d3e9d294e45c9b03a76aef41209dd15ebff5d46c4bd888e51a93cf99a7329636c63514396b4a452003a35bf704bf11ca01483bfa8b34b43561848d28905960114c8ac04049af4b6315a416782bb8324af6cfc93537a2ad1a445cfd0ca2a71acd7ac41fadbf933c2a51be344d120a2a4cf30c1bf9845f20c6fe39e07ea2cce61f0c9bb048165fe5e4de877550111e129f1cf1097710d41c4ac70fcdfa5ba2023c6ff1cbeac322de49d1b6df7c2032c61a830e3c17286de9462bf242fca2883585b93870a73853face6a6bf411198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa";
    let expect = "0000000000000000000000000000000000000000000000000000000000000001";
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
