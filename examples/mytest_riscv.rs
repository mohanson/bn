#![no_std]
#![no_main]
#![feature(asm)]
#![feature(lang_items)]

#[no_mangle]
pub fn _start() -> ! {
    bn128_pariing();
    exit(0)
}

/// Exit syscall
/// https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0009-vm-syscalls/0009-vm-syscalls.md
pub fn exit(_code: i8) -> ! {
    unsafe {
        // a0 is _code
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

use bn::fields::FieldElement;
use bn::groups::GroupElement;
use bn::{
    arith::U256,
    fields::{self, Fq, Fq12, Fq2, Fq6},
    groups,
};

pub fn hex_decode2(s: &str) -> [u8; 32] {
    let mut a = [0u8; 32];
    for i in (0..s.len()).step_by(2) {
        let b = u8::from_str_radix(&s[i..i + 2], 16).unwrap();
        a[i / 2] = b;
    }
    a
}

fn gen_u256(s: &str) -> U256 {
    U256::from_slice(&hex_decode2(s.strip_prefix("0x").unwrap())).unwrap()
}

fn gen_fields_fq(s: &str) -> bn::fields::Fq {
    bn::fields::Fq::new(gen_u256(s)).unwrap()
}

fn gen_rax_fq(s: &str) -> bn::fields::Fq {
    bn::fields::Fq(gen_u256(s))
}

fn bn128_pariing() {
    let x1 = Fq2::new(
        gen_fields_fq("0x2276cf730cf493cd95d64677bbb75fc42db72513a4c1e387b476d056f80aa75f"),
        gen_fields_fq("0x1213d2149b006137fcfb23036606f848d638d576a120ca981b5b1a5f9300b3ee"),
    );
    let x2 = Fq2::new(
        gen_fields_fq("0x096df1f82dff337dd5972e32a8ad43e28a78a96a823ef1cd4debe12b6552ea5f"),
        gen_fields_fq("0x21ee6226d31426322afcda621464d0611d226783262e21bb3bc86b537e986237"),
    );
    let x3 = Fq2::new(
        gen_fields_fq("0x2eca0c7238bf16e83e7a1e6c5d49540685ff51380f309842a98561558019fc02"),
        gen_fields_fq("0x03d3260361bb8451de5ff5ecd17f010ff22f5c31cdf184e9020b06fa5997db84"),
    );

    let a = bn::groups::AffineG2::new(x1, x2).unwrap();
    let p = bn::groups::AffineG1::new(x3.c0, x3.c1).unwrap();

    let b = a
        .precompute()
        .miller_loop(&p)
        .final_exponentiation()
        .unwrap();
    assert_eq!(
        b.c0.c0.c0,
        gen_rax_fq("0x0f4eabc79f4207cae5c25efed5dd895b483c6f02ae7169a2465a1b0c5d7e87f5")
    );
    assert_eq!(
        b.c0.c0.c1,
        gen_rax_fq("0x1ea573771832738906d7e744788de7aa58c04f1b67485fe4556f89c18606946d")
    );
    assert_eq!(
        b.c0.c1.c0,
        gen_rax_fq("0x225653f8808a23eda89b43521c123a0482c5209467b0b8a98820ef1d0f0f6ff6")
    );
    assert_eq!(
        b.c0.c1.c1,
        gen_rax_fq("0x2470c518434776631bd2e00036fa72262b2b31880ae191cde3bb08911e764eef")
    );
    assert_eq!(
        b.c0.c2.c0,
        gen_rax_fq("0x0d520d77a697c64ebaea6b4f901409aab541f7a95e4218c02c177b71eb13505c")
    );
    assert_eq!(
        b.c0.c2.c1,
        gen_rax_fq("0x28594977c319352c6c7558065cdb68a5297cbed2820ca7c316171ac93c99fc80")
    );
    assert_eq!(
        b.c1.c0.c0,
        gen_rax_fq("0x21bc12ad2febc4c7c2b691208deea520c189dc7c693c5a344c0573d3d863f174")
    );
    assert_eq!(
        b.c1.c0.c1,
        gen_rax_fq("0x0ef0acb2435e8a50fd26d98d6933668dd4325b8e2ebc79b66316f3a1d2640d19")
    );
    assert_eq!(
        b.c1.c1.c0,
        gen_rax_fq("0x09e7b2ab23b2f61ac30c80dd44883abc0da9a50a3fb2216a1c594dca7e4859e8")
    );
    assert_eq!(
        b.c1.c1.c1,
        gen_rax_fq("0x2c191a033b0c28682f64cd2b0da005812dbf182b278d17a0eae786deaa0749b3")
    );
    assert_eq!(
        b.c1.c2.c0,
        gen_rax_fq("0x27f00e6cd47e6cde40df2e14a4d3e83bc5bd714c7fade1979e4ed2675806a283")
    );
    assert_eq!(
        b.c1.c2.c1,
        gen_rax_fq("0x1f8aad6f81013a83ebd3ced4b5ed0da2651932067f8d24f7c9b084c9ce3b799c")
    );
}
