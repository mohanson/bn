#[no_mangle]
pub extern "C" fn alt_bn128_add(data: *mut u8, data_len: u32, output: *mut u8) -> u32 {
    unsafe {
        let mut buf0 = [0u8; 128];
        for i in 0..if data_len > 128 { 128 } else { data_len } {
            buf0[i as usize] = *data.offset(i as isize);
        }
        let mut buf1 = [0u8; 64];
        if let Err(_) = alt_bn128::ethereum::alt_bn128_add(&buf0, &mut buf1) {
            return 1;
        }
        for i in 0..64 {
            output.offset(i as isize).write(buf1[i as usize]);
        }
        return 0;
    }
}

#[no_mangle]
pub extern "C" fn alt_bn128_mul(data: *mut u8, data_len: u32, output: *mut u8) -> u32 {
    unsafe {
        let mut buf0 = [0u8; 96];
        for i in 0..if data_len > 96 { 96 } else { data_len } {
            buf0[i as usize] = *data.offset(i as isize);
        }
        let mut buf1 = [0u8; 64];
        if let Err(_) = alt_bn128::ethereum::alt_bn128_mul(&buf0, &mut buf1) {
            return 1;
        }
        for i in 0..64 {
            output.offset(i as isize).write(buf1[i as usize]);
        }
        return 0;
    }
}

#[no_mangle]
pub extern "C" fn alt_bn128_pairing(data: *mut u8, data_len: u32, output: *mut u8) -> u32 {
    unsafe {
        let mut buf0 = [0u8; 3072];
        for i in 0..if data_len > 3072 { 3072 } else { data_len } {
            buf0[i as usize] = *data.offset(i as isize);
        }
        let mut buf1 = [0u8; 32];
        if let Err(_) = alt_bn128::ethereum::alt_bn128_pairing(&buf0, &mut buf1) {
            return 1;
        }
        for i in 0..32 {
            output.offset(i as isize).write(buf1[i as usize]);
        }
        return 0;
    }
}
