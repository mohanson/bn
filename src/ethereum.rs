use crate::{arith::U256, pairing_batch, AffineG1, AffineG2, Fq, Fq2, Fr, Group, Gt, G1, G2};

pub struct Error(pub &'static str);

fn read_fr(buf: &[u8]) -> Result<Fr, Error> {
    Fr::from_slice(buf).map_err(|_| Error("invalid fr"))
}

fn read_pt(buf: &[u8]) -> Result<G1, Error> {
    let px = Fq::from_slice(&buf[0..32]).map_err(|_| Error("invalid pt"))?;
    let py = Fq::from_slice(&buf[32..64]).map_err(|_| Error("invalid pt"))?;
    Ok(if px == Fq::zero() && py == Fq::zero() {
        G1::zero()
    } else {
        AffineG1::new(px, py)
            .map_err(|_| Error("invalid pt"))?
            .into()
    })
}

pub fn bn128_add(data: &[u8], output: &mut [u8; 64]) -> Result<(), Error> {
    let mut buffer = [0u8; 128];
    if data.len() < 128 {
        buffer[0..data.len()].copy_from_slice(&data);
    } else {
        buffer[0..128].copy_from_slice(&data[0..128]);
    }
    let p1 = read_pt(&buffer[0..64])?;
    let p2 = read_pt(&buffer[64..128])?;

    let mut buffer = [0u8; 64];
    if let Some(sum) = AffineG1::from_jacobian(p1 + p2) {
        sum.x().to_big_endian(&mut buffer[0..32]).unwrap();
        sum.y().to_big_endian(&mut buffer[32..64]).unwrap();
    }
    *output = buffer;
    Ok(())
}

pub fn bn128_mul(data: &[u8], output: &mut [u8; 64]) -> Result<(), Error> {
    let mut buffer = [0u8; 96];
    if data.len() < 96 {
        buffer[0..data.len()].copy_from_slice(&data);
    } else {
        buffer[0..96].copy_from_slice(&data[0..96]);
    }
    let pt = read_pt(&buffer[0..64])?;
    let fr = read_fr(&buffer[64..96])?;
    let mut buffer = [0u8; 64];
    if let Some(sum) = AffineG1::from_jacobian(pt * fr) {
        sum.x().to_big_endian(&mut buffer[0..32]).unwrap();
        sum.y().to_big_endian(&mut buffer[32..64]).unwrap();
    }
    *output = buffer;
    Ok(())
}

pub fn bn128_pairing(data: &[u8], output: &mut [u8; 32]) -> Result<(), Error> {
    if data.len() % 192 != 0 {
        return Err(Error(
            "Invalid input length, must be multiple of 192 (3 * (32*2))",
        ));
    }

    let elements = data.len() / 192; // (a, b_a, b_b - each 64-byte affine coordinates)
    let ret_val = if data.len() == 0 {
        U256::one()
    } else {
        let mut vals = [(G1::default(), G2::default()); 16];
        for idx in 0..elements {
            let a_x = Fq::from_slice(&data[idx * 192..idx * 192 + 32])
                .map_err(|_| Error("Invalid a argument x coordinate"))?;

            let a_y = Fq::from_slice(&data[idx * 192 + 32..idx * 192 + 64])
                .map_err(|_| Error("Invalid a argument y coordinate"))?;

            let b_a_y = Fq::from_slice(&data[idx * 192 + 64..idx * 192 + 96])
                .map_err(|_| Error("Invalid b argument imaginary coeff x coordinate"))?;

            let b_a_x = Fq::from_slice(&data[idx * 192 + 96..idx * 192 + 128])
                .map_err(|_| Error("Invalid b argument imaginary coeff y coordinate"))?;

            let b_b_y = Fq::from_slice(&data[idx * 192 + 128..idx * 192 + 160])
                .map_err(|_| Error("Invalid b argument real coeff x coordinate"))?;

            let b_b_x = Fq::from_slice(&data[idx * 192 + 160..idx * 192 + 192])
                .map_err(|_| Error("Invalid b argument real coeff y coordinate"))?;

            let b_a = Fq2::new(b_a_x, b_a_y);
            let b_b = Fq2::new(b_b_x, b_b_y);
            let b = if b_a.is_zero() && b_b.is_zero() {
                G2::zero()
            } else {
                G2::from(
                    AffineG2::new(b_a, b_b)
                        .map_err(|_| Error("Invalid b argument - not on curve"))?,
                )
            };
            let a = if a_x.is_zero() && a_y.is_zero() {
                G1::zero()
            } else {
                G1::from(
                    AffineG1::new(a_x, a_y)
                        .map_err(|_| Error("Invalid a argument - not on curve"))?,
                )
            };
            vals[idx] = (a, b);
        }

        let mul = pairing_batch(&vals[0..elements]);

        if mul == Gt::one() {
            U256::one()
        } else {
            U256::zero()
        }
    };

    ret_val
        .to_big_endian(output)
        .expect("Cannot fail since 0..32 is 32-byte length");

    Ok(())
}
