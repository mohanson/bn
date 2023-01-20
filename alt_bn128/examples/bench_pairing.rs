use alt_bn128::ethereum::alt_bn128_pairing;
use alt_bn128::ethereum::ut::{hex2bin, ALT_BN128_PAIRING_CASE};

fn main() {
    let mut buf0 = [0x00; 4096];
    let mut buf1 = [0x00; 32];
    for (_, (inputs, expect)) in ALT_BN128_PAIRING_CASE.iter().enumerate() {
        let a = std::time::SystemTime::now();
        for _ in 0..20 {
            hex2bin(inputs, &mut buf0[..]);
            assert!(alt_bn128_pairing(&buf0[0..inputs.len() / 2], &mut buf1).is_ok());
            hex2bin(expect, &mut buf0[..]);
            assert_eq!(buf0[0..32], buf1[..]);
        }
        let b = a.elapsed().unwrap().as_nanos();
        println!("{:8} ns/op", b / 20);
    }
}
