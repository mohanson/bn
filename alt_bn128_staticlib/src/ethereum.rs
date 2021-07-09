#[no_mangle]
pub extern "C" fn alt_bn128_add(data: *mut u8, data_len: u32, output: *mut u8) -> u32 {
    unsafe {
        let mut buf0 = [0u8; 128];
        for i in 0..data_len {
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
