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

pub fn alt_bn128_add(data: &[u8], output: &mut [u8; 64]) -> Result<(), Error> {
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

pub fn alt_bn128_mul(data: &[u8], output: &mut [u8; 64]) -> Result<(), Error> {
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

pub fn alt_bn128_pairing(data: &[u8], output: &mut [u8; 32]) -> Result<(), Error> {
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

pub mod ut {
    use super::{alt_bn128_add, alt_bn128_mul, alt_bn128_pairing};

    pub fn hex2bin(s: &str, output: &mut [u8]) {
        for i in (0..s.len()).step_by(2) {
            let b = u8::from_str_radix(&s[i..i + 2], 16).unwrap();
            output[i / 2] = b;
        }
    }

    pub fn test_alt_bn128_add() {
        // Taking from
        // https://github.com/ethereum/go-ethereum/blob/master/core/vm/testdata/precompiles/bn256Add.json
        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "18b18acfb4c2c30276db5411368e7185b311dd124691610c5d3b74034e093dc9063c909c4720840cb5134cb9f59fa749755796819658d32efc0d288198f3726607c2b7f58a84bd6145f00c9c2bc0bb1a187f20ff2c92963a88019e7c6a014eed06614e20c147e940f2d70da3f74c9a17df361706a4485c742bd6788478fa17d7";
        let expect = "2243525c5efd4b9c3d3c45ac0ca3fe4dd85e830a4ce6b65fa1eeaee202839703301d1d33be6da8e509df21cc35964723180eed7532537db9ae5e7d48f195c915";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_add(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "2243525c5efd4b9c3d3c45ac0ca3fe4dd85e830a4ce6b65fa1eeaee202839703301d1d33be6da8e509df21cc35964723180eed7532537db9ae5e7d48f195c91518b18acfb4c2c30276db5411368e7185b311dd124691610c5d3b74034e093dc9063c909c4720840cb5134cb9f59fa749755796819658d32efc0d288198f37266";
        let expect = "2bd3e6d0f3b142924f5ca7b49ce5b9d54c4703d7ae5648e61d02268b1a0a9fb721611ce0a6af85915e2f1d70300909ce2e49dfad4a4619c8390cae66cefdb204";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_add(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let expect = "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_add(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let expect = "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_add(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let expect = "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_add(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "";
        let expect = "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_add(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let expect = "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_add(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002";
        let expect = "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_add(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let expect = "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_add(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002";
        let expect = "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_add(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let expect = "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_add(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let expect = "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_add(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002";
        let expect = "030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd315ed738c0e0a7c92e7845f96b2ae9c0a68a6a449e3538fc7ff3ebf7a5a18a2c4";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_add(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let expect = "030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd315ed738c0e0a7c92e7845f96b2ae9c0a68a6a449e3538fc7ff3ebf7a5a18a2c4";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_add(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "17c139df0efee0f766bc0204762b774362e4ded88953a39ce849a8a7fa163fa901e0559bacb160664764a357af8a9fe70baa9258e0b959273ffc5718c6d4cc7c039730ea8dff1254c0fee9c0ea777d29a9c710b7e616683f194f18c43b43b869073a5ffcc6fc7a28c30723d6e58ce577356982d65b833a5a5c15bf9024b43d98";
        let expect = "15bf2bb17880144b5d1cd2b1f46eff9d617bffd1ca57c37fb5a49bd84e53cf66049c797f9ce0d17083deb32b5e36f2ea2a212ee036598dd7624c168993d1355f";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_add(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "17c139df0efee0f766bc0204762b774362e4ded88953a39ce849a8a7fa163fa901e0559bacb160664764a357af8a9fe70baa9258e0b959273ffc5718c6d4cc7c17c139df0efee0f766bc0204762b774362e4ded88953a39ce849a8a7fa163fa92e83f8d734803fc370eba25ed1f6b8768bd6d83887b87165fc2434fe11a830cb00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let expect = "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_add(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);
    }

    pub fn test_alt_bn128_mul() {
        // Taking from
        // https://github.com/ethereum/go-ethereum/blob/master/core/vm/testdata/precompiles/bn256ScalarMul.json
        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "2bd3e6d0f3b142924f5ca7b49ce5b9d54c4703d7ae5648e61d02268b1a0a9fb721611ce0a6af85915e2f1d70300909ce2e49dfad4a4619c8390cae66cefdb20400000000000000000000000000000000000000000000000011138ce750fa15c2";
        let expect = "070a8d6a982153cae4be29d434e8faef8a47b274a053f5a4ee2a6c9c13c31e5c031b8ce914eba3a9ffb989f9cdd5b0f01943074bf4f0f315690ec3cec6981afc";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_mul(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "070a8d6a982153cae4be29d434e8faef8a47b274a053f5a4ee2a6c9c13c31e5c031b8ce914eba3a9ffb989f9cdd5b0f01943074bf4f0f315690ec3cec6981afc30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd46";
        let expect = "025a6f4181d2b4ea8b724290ffb40156eb0adb514c688556eb79cdea0752c2bb2eff3f31dea215f1eb86023a133a996eb6300b44da664d64251d05381bb8a02e";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_mul(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "025a6f4181d2b4ea8b724290ffb40156eb0adb514c688556eb79cdea0752c2bb2eff3f31dea215f1eb86023a133a996eb6300b44da664d64251d05381bb8a02e183227397098d014dc2822db40c0ac2ecbc0b548b438e5469e10460b6c3e7ea3";
        let expect = "14789d0d4a730b354403b5fac948113739e276c23e0258d8596ee72f9cd9d3230af18a63153e0ec25ff9f2951dd3fa90ed0197bfef6e2a1a62b5095b9d2b4a27";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_mul(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "1a87b0584ce92f4593d161480614f2989035225609f08058ccfa3d0f940febe31a2f3c951f6dadcc7ee9007dff81504b0fcd6d7cf59996efdc33d92bf7f9f8f6ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        let expect = "2cde5879ba6f13c0b5aa4ef627f159a3347df9722efce88a9afbb20b763b4c411aa7e43076f6aee272755a7f9b84832e71559ba0d2e0b17d5f9f01755e5b0d11";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_mul(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "1a87b0584ce92f4593d161480614f2989035225609f08058ccfa3d0f940febe31a2f3c951f6dadcc7ee9007dff81504b0fcd6d7cf59996efdc33d92bf7f9f8f630644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000000";
        let expect = "1a87b0584ce92f4593d161480614f2989035225609f08058ccfa3d0f940febe3163511ddc1c3f25d396745388200081287b3fd1472d8339d5fecb2eae0830451";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_mul(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "1a87b0584ce92f4593d161480614f2989035225609f08058ccfa3d0f940febe31a2f3c951f6dadcc7ee9007dff81504b0fcd6d7cf59996efdc33d92bf7f9f8f60000000000000000000000000000000100000000000000000000000000000000";
        let expect = "1051acb0700ec6d42a88215852d582efbaef31529b6fcbc3277b5c1b300f5cf0135b2394bb45ab04b8bd7611bd2dfe1de6a4e6e2ccea1ea1955f577cd66af85b";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_mul(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "1a87b0584ce92f4593d161480614f2989035225609f08058ccfa3d0f940febe31a2f3c951f6dadcc7ee9007dff81504b0fcd6d7cf59996efdc33d92bf7f9f8f60000000000000000000000000000000000000000000000000000000000000009";
        let expect = "1dbad7d39dbc56379f78fac1bca147dc8e66de1b9d183c7b167351bfe0aeab742cd757d51289cd8dbd0acf9e673ad67d0f0a89f912af47ed1be53664f5692575";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_mul(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "1a87b0584ce92f4593d161480614f2989035225609f08058ccfa3d0f940febe31a2f3c951f6dadcc7ee9007dff81504b0fcd6d7cf59996efdc33d92bf7f9f8f60000000000000000000000000000000000000000000000000000000000000001";
        let expect = "1a87b0584ce92f4593d161480614f2989035225609f08058ccfa3d0f940febe31a2f3c951f6dadcc7ee9007dff81504b0fcd6d7cf59996efdc33d92bf7f9f8f6";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_mul(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "17c139df0efee0f766bc0204762b774362e4ded88953a39ce849a8a7fa163fa901e0559bacb160664764a357af8a9fe70baa9258e0b959273ffc5718c6d4cc7cffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        let expect = "29e587aadd7c06722aabba753017c093f70ba7eb1f1c0104ec0564e7e3e21f6022b1143f6a41008e7755c71c3d00b6b915d386de21783ef590486d8afa8453b1";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_mul(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "17c139df0efee0f766bc0204762b774362e4ded88953a39ce849a8a7fa163fa901e0559bacb160664764a357af8a9fe70baa9258e0b959273ffc5718c6d4cc7c30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000000";
        let expect = "17c139df0efee0f766bc0204762b774362e4ded88953a39ce849a8a7fa163fa92e83f8d734803fc370eba25ed1f6b8768bd6d83887b87165fc2434fe11a830cb";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_mul(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "17c139df0efee0f766bc0204762b774362e4ded88953a39ce849a8a7fa163fa901e0559bacb160664764a357af8a9fe70baa9258e0b959273ffc5718c6d4cc7c0000000000000000000000000000000100000000000000000000000000000000";
        let expect = "221a3577763877920d0d14a91cd59b9479f83b87a653bb41f82a3f6f120cea7c2752c7f64cdd7f0e494bff7b60419f242210f2026ed2ec70f89f78a4c56a1f15";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_mul(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "17c139df0efee0f766bc0204762b774362e4ded88953a39ce849a8a7fa163fa901e0559bacb160664764a357af8a9fe70baa9258e0b959273ffc5718c6d4cc7c0000000000000000000000000000000000000000000000000000000000000009";
        let expect = "228e687a379ba154554040f8821f4e41ee2be287c201aa9c3bc02c9dd12f1e691e0fd6ee672d04cfd924ed8fdc7ba5f2d06c53c1edc30f65f2af5a5b97f0a76a";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_mul(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "17c139df0efee0f766bc0204762b774362e4ded88953a39ce849a8a7fa163fa901e0559bacb160664764a357af8a9fe70baa9258e0b959273ffc5718c6d4cc7c0000000000000000000000000000000000000000000000000000000000000001";
        let expect = "17c139df0efee0f766bc0204762b774362e4ded88953a39ce849a8a7fa163fa901e0559bacb160664764a357af8a9fe70baa9258e0b959273ffc5718c6d4cc7c";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_mul(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "039730ea8dff1254c0fee9c0ea777d29a9c710b7e616683f194f18c43b43b869073a5ffcc6fc7a28c30723d6e58ce577356982d65b833a5a5c15bf9024b43d98ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        let expect = "00a1a234d08efaa2616607e31eca1980128b00b415c845ff25bba3afcb81dc00242077290ed33906aeb8e42fd98c41bcb9057ba03421af3f2d08cfc441186024";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_mul(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "039730ea8dff1254c0fee9c0ea777d29a9c710b7e616683f194f18c43b43b869073a5ffcc6fc7a28c30723d6e58ce577356982d65b833a5a5c15bf9024b43d9830644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000000";
        let expect = "039730ea8dff1254c0fee9c0ea777d29a9c710b7e616683f194f18c43b43b8692929ee761a352600f54921df9bf472e66217e7bb0cee9032e00acc86b3c8bfaf";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_mul(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "039730ea8dff1254c0fee9c0ea777d29a9c710b7e616683f194f18c43b43b869073a5ffcc6fc7a28c30723d6e58ce577356982d65b833a5a5c15bf9024b43d980000000000000000000000000000000100000000000000000000000000000000";
        let expect = "1071b63011e8c222c5a771dfa03c2e11aac9666dd097f2c620852c3951a4376a2f46fe2f73e1cf310a168d56baa5575a8319389d7bfa6b29ee2d908305791434";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_mul(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "039730ea8dff1254c0fee9c0ea777d29a9c710b7e616683f194f18c43b43b869073a5ffcc6fc7a28c30723d6e58ce577356982d65b833a5a5c15bf9024b43d980000000000000000000000000000000000000000000000000000000000000009";
        let expect = "19f75b9dd68c080a688774a6213f131e3052bd353a304a189d7a2ee367e3c2582612f545fb9fc89fde80fd81c68fc7dcb27fea5fc124eeda69433cf5c46d2d7f";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_mul(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);

        let mut buf0 = [0x00; 1024];
        let mut buf1 = [0x00; 64];
        let inputs = "039730ea8dff1254c0fee9c0ea777d29a9c710b7e616683f194f18c43b43b869073a5ffcc6fc7a28c30723d6e58ce577356982d65b833a5a5c15bf9024b43d980000000000000000000000000000000000000000000000000000000000000001";
        let expect = "039730ea8dff1254c0fee9c0ea777d29a9c710b7e616683f194f18c43b43b869073a5ffcc6fc7a28c30723d6e58ce577356982d65b833a5a5c15bf9024b43d98";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_mul(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..64], buf1[..]);
    }

    pub fn test_alt_bn128_pairing() {
        let mut buf0 = [0x00; 384];
        let mut buf1 = [0x00; 32];
        let inputs = "1c76476f4def4bb94541d57ebba1193381ffa7aa76ada664dd31c16024c43f593034dd2920f673e204fee2811c678745fc819b55d3e9d294e45c9b03a76aef41209dd15ebff5d46c4bd888e51a93cf99a7329636c63514396b4a452003a35bf704bf11ca01483bfa8b34b43561848d28905960114c8ac04049af4b6315a416782bb8324af6cfc93537a2ad1a445cfd0ca2a71acd7ac41fadbf933c2a51be344d120a2a4cf30c1bf9845f20c6fe39e07ea2cce61f0c9bb048165fe5e4de877550111e129f1cf1097710d41c4ac70fcdfa5ba2023c6ff1cbeac322de49d1b6df7c2032c61a830e3c17286de9462bf242fca2883585b93870a73853face6a6bf411198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa";
        let expect = "0000000000000000000000000000000000000000000000000000000000000001";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_pairing(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..32], buf1[..]);

        let mut buf0 = [0x00; 384];
        let mut buf1 = [0x00; 32];
        let inputs = "2eca0c7238bf16e83e7a1e6c5d49540685ff51380f309842a98561558019fc0203d3260361bb8451de5ff5ecd17f010ff22f5c31cdf184e9020b06fa5997db841213d2149b006137fcfb23036606f848d638d576a120ca981b5b1a5f9300b3ee2276cf730cf493cd95d64677bbb75fc42db72513a4c1e387b476d056f80aa75f21ee6226d31426322afcda621464d0611d226783262e21bb3bc86b537e986237096df1f82dff337dd5972e32a8ad43e28a78a96a823ef1cd4debe12b6552ea5f06967a1237ebfeca9aaae0d6d0bab8e28c198c5a339ef8a2407e31cdac516db922160fa257a5fd5b280642ff47b65eca77e626cb685c84fa6d3b6882a283ddd1198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa";
        let expect = "0000000000000000000000000000000000000000000000000000000000000001";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_pairing(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..32], buf1[..]);

        let mut buf0 = [0x00; 384];
        let mut buf1 = [0x00; 32];
        let inputs = "0f25929bcb43d5a57391564615c9e70a992b10eafa4db109709649cf48c50dd216da2f5cb6be7a0aa72c440c53c9bbdfec6c36c7d515536431b3a865468acbba2e89718ad33c8bed92e210e81d1853435399a271913a6520736a4729cf0d51eb01a9e2ffa2e92599b68e44de5bcf354fa2642bd4f26b259daa6f7ce3ed57aeb314a9a87b789a58af499b314e13c3d65bede56c07ea2d418d6874857b70763713178fb49a2d6cd347dc58973ff49613a20757d0fcc22079f9abd10c3baee245901b9e027bd5cfc2cb5db82d4dc9677ac795ec500ecd47deee3b5da006d6d049b811d7511c78158de484232fc68daf8a45cf217d1c2fae693ff5871e8752d73b21198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa";
        let expect = "0000000000000000000000000000000000000000000000000000000000000001";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_pairing(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..32], buf1[..]);

        let mut buf0 = [0x00; 576];
        let mut buf1 = [0x00; 32];
        let inputs = "2f2ea0b3da1e8ef11914acf8b2e1b32d99df51f5f4f206fc6b947eae860eddb6068134ddb33dc888ef446b648d72338684d678d2eb2371c61a50734d78da4b7225f83c8b6ab9de74e7da488ef02645c5a16a6652c3c71a15dc37fe3a5dcb7cb122acdedd6308e3bb230d226d16a105295f523a8a02bfc5e8bd2da135ac4c245d065bbad92e7c4e31bf3757f1fe7362a63fbfee50e7dc68da116e67d600d9bf6806d302580dc0661002994e7cd3a7f224e7ddc27802777486bf80f40e4ca3cfdb186bac5188a98c45e6016873d107f5cd131f3a3e339d0375e58bd6219347b008122ae2b09e539e152ec5364e7e2204b03d11d3caa038bfc7cd499f8176aacbee1f39e4e4afc4bc74790a4a028aff2c3d2538731fb755edefd8cb48d6ea589b5e283f150794b6736f670d6a1033f9b46c6f5204f50813eb85c8dc4b59db1c5d39140d97ee4d2b36d99bc49974d18ecca3e7ad51011956051b464d9e27d46cc25e0764bb98575bd466d32db7b15f582b2d5c452b36aa394b789366e5e3ca5aabd415794ab061441e51d01e94640b7e3084a07e02c78cf3103c542bc5b298669f211b88da1679b0b64a63b7e0e7bfe52aae524f73a55be7fe70c7e9bfc94b4cf0da1213d2149b006137fcfb23036606f848d638d576a120ca981b5b1a5f9300b3ee2276cf730cf493cd95d64677bbb75fc42db72513a4c1e387b476d056f80aa75f21ee6226d31426322afcda621464d0611d226783262e21bb3bc86b537e986237096df1f82dff337dd5972e32a8ad43e28a78a96a823ef1cd4debe12b6552ea5f";
        let expect = "0000000000000000000000000000000000000000000000000000000000000001";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_pairing(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..32], buf1[..]);

        let mut buf0 = [0x00; 576];
        let mut buf1 = [0x00; 32];
        let inputs = "20a754d2071d4d53903e3b31a7e98ad6882d58aec240ef981fdf0a9d22c5926a29c853fcea789887315916bbeb89ca37edb355b4f980c9a12a94f30deeed30211213d2149b006137fcfb23036606f848d638d576a120ca981b5b1a5f9300b3ee2276cf730cf493cd95d64677bbb75fc42db72513a4c1e387b476d056f80aa75f21ee6226d31426322afcda621464d0611d226783262e21bb3bc86b537e986237096df1f82dff337dd5972e32a8ad43e28a78a96a823ef1cd4debe12b6552ea5f1abb4a25eb9379ae96c84fff9f0540abcfc0a0d11aeda02d4f37e4baf74cb0c11073b3ff2cdbb38755f8691ea59e9606696b3ff278acfc098fa8226470d03869217cee0a9ad79a4493b5253e2e4e3a39fc2df38419f230d341f60cb064a0ac290a3d76f140db8418ba512272381446eb73958670f00cf46f1d9e64cba057b53c26f64a8ec70387a13e41430ed3ee4a7db2059cc5fc13c067194bcc0cb49a98552fd72bd9edb657346127da132e5b82ab908f5816c826acb499e22f2412d1a2d70f25929bcb43d5a57391564615c9e70a992b10eafa4db109709649cf48c50dd2198a1f162a73261f112401aa2db79c7dab1533c9935c77290a6ce3b191f2318d198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa";
        let expect = "0000000000000000000000000000000000000000000000000000000000000001";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_pairing(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..32], buf1[..]);

        let mut buf0 = [0x00; 384];
        let mut buf1 = [0x00; 32];
        let inputs = "1c76476f4def4bb94541d57ebba1193381ffa7aa76ada664dd31c16024c43f593034dd2920f673e204fee2811c678745fc819b55d3e9d294e45c9b03a76aef41209dd15ebff5d46c4bd888e51a93cf99a7329636c63514396b4a452003a35bf704bf11ca01483bfa8b34b43561848d28905960114c8ac04049af4b6315a416782bb8324af6cfc93537a2ad1a445cfd0ca2a71acd7ac41fadbf933c2a51be344d120a2a4cf30c1bf9845f20c6fe39e07ea2cce61f0c9bb048165fe5e4de877550111e129f1cf1097710d41c4ac70fcdfa5ba2023c6ff1cbeac322de49d1b6df7c103188585e2364128fe25c70558f1560f4f9350baf3959e603cc91486e110936198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa";
        let expect = "0000000000000000000000000000000000000000000000000000000000000000";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_pairing(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..32], buf1[..]);

        let mut buf0 = [0x00; 192];
        let mut buf1 = [0x00; 32];
        let inputs = "";
        let expect = "0000000000000000000000000000000000000000000000000000000000000001";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_pairing(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..32], buf1[..]);

        let mut buf0 = [0x00; 192];
        let mut buf1 = [0x00; 32];
        let inputs = "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa";
        let expect = "0000000000000000000000000000000000000000000000000000000000000000";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_pairing(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..32], buf1[..]);

        let mut buf0 = [0x00; 384];
        let mut buf1 = [0x00; 32];
        let inputs = "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed275dc4a288d1afb3cbb1ac09187524c7db36395df7be3b99e673b13a075a65ec1d9befcd05a5323e6da4d435f3b617cdb3af83285c2df711ef39c01571827f9d";
        let expect = "0000000000000000000000000000000000000000000000000000000000000001";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_pairing(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..32], buf1[..]);

        let mut buf0 = [0x00; 384];
        let mut buf1 = [0x00; 32];
        let inputs = "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002203e205db4f19b37b60121b83a7333706db86431c6d835849957ed8c3928ad7927dc7234fd11d3e8c36c59277c3e6f149d5cd3cfa9a62aee49f8130962b4b3b9195e8aa5b7827463722b8c153931579d3505566b4edf48d498e185f0509de15204bb53b8977e5f92a0bc372742c4830944a59b4fe6b1c0466e2a6dad122b5d2e030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd31a76dae6d3272396d0cbe61fced2bc532edac647851e3ac53ce1cc9c7e645a83198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa";
        let expect = "0000000000000000000000000000000000000000000000000000000000000001";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_pairing(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..32], buf1[..]);

        let mut buf0 = [0x00; 384];
        let mut buf1 = [0x00; 32];
        let inputs = "105456a333e6d636854f987ea7bb713dfd0ae8371a72aea313ae0c32c0bf10160cf031d41b41557f3e7e3ba0c51bebe5da8e6ecd855ec50fc87efcdeac168bcc0476be093a6d2b4bbf907172049874af11e1b6267606e00804d3ff0037ec57fd3010c68cb50161b7d1d96bb71edfec9880171954e56871abf3d93cc94d745fa114c059d74e5b6c4ec14ae5864ebe23a71781d86c29fb8fb6cce94f70d3de7a2101b33461f39d9e887dbb100f170a2345dde3c07e256d1dfa2b657ba5cd030427000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000021a2c3013d2ea92e13c800cde68ef56a294b883f6ac35d25f587c09b1b3c635f7290158a80cd3d66530f74dc94c94adb88f5cdb481acca997b6e60071f08a115f2f997f3dbd66a7afe07fe7862ce239edba9e05c5afff7f8a1259c9733b2dfbb929d1691530ca701b4a106054688728c9972c8512e9789e9567aae23e302ccd75";
        let expect = "0000000000000000000000000000000000000000000000000000000000000001";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_pairing(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..32], buf1[..]);

        let mut buf0 = [0x00; 1920];
        let mut buf1 = [0x00; 32];
        let inputs = "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed275dc4a288d1afb3cbb1ac09187524c7db36395df7be3b99e673b13a075a65ec1d9befcd05a5323e6da4d435f3b617cdb3af83285c2df711ef39c01571827f9d00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed275dc4a288d1afb3cbb1ac09187524c7db36395df7be3b99e673b13a075a65ec1d9befcd05a5323e6da4d435f3b617cdb3af83285c2df711ef39c01571827f9d00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed275dc4a288d1afb3cbb1ac09187524c7db36395df7be3b99e673b13a075a65ec1d9befcd05a5323e6da4d435f3b617cdb3af83285c2df711ef39c01571827f9d00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed275dc4a288d1afb3cbb1ac09187524c7db36395df7be3b99e673b13a075a65ec1d9befcd05a5323e6da4d435f3b617cdb3af83285c2df711ef39c01571827f9d00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed275dc4a288d1afb3cbb1ac09187524c7db36395df7be3b99e673b13a075a65ec1d9befcd05a5323e6da4d435f3b617cdb3af83285c2df711ef39c01571827f9d";
        let expect = "0000000000000000000000000000000000000000000000000000000000000001";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_pairing(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..32], buf1[..]);

        let mut buf0 = [0x00; 1920];
        let mut buf1 = [0x00; 32];
        let inputs = "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002203e205db4f19b37b60121b83a7333706db86431c6d835849957ed8c3928ad7927dc7234fd11d3e8c36c59277c3e6f149d5cd3cfa9a62aee49f8130962b4b3b9195e8aa5b7827463722b8c153931579d3505566b4edf48d498e185f0509de15204bb53b8977e5f92a0bc372742c4830944a59b4fe6b1c0466e2a6dad122b5d2e030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd31a76dae6d3272396d0cbe61fced2bc532edac647851e3ac53ce1cc9c7e645a83198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002203e205db4f19b37b60121b83a7333706db86431c6d835849957ed8c3928ad7927dc7234fd11d3e8c36c59277c3e6f149d5cd3cfa9a62aee49f8130962b4b3b9195e8aa5b7827463722b8c153931579d3505566b4edf48d498e185f0509de15204bb53b8977e5f92a0bc372742c4830944a59b4fe6b1c0466e2a6dad122b5d2e030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd31a76dae6d3272396d0cbe61fced2bc532edac647851e3ac53ce1cc9c7e645a83198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002203e205db4f19b37b60121b83a7333706db86431c6d835849957ed8c3928ad7927dc7234fd11d3e8c36c59277c3e6f149d5cd3cfa9a62aee49f8130962b4b3b9195e8aa5b7827463722b8c153931579d3505566b4edf48d498e185f0509de15204bb53b8977e5f92a0bc372742c4830944a59b4fe6b1c0466e2a6dad122b5d2e030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd31a76dae6d3272396d0cbe61fced2bc532edac647851e3ac53ce1cc9c7e645a83198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002203e205db4f19b37b60121b83a7333706db86431c6d835849957ed8c3928ad7927dc7234fd11d3e8c36c59277c3e6f149d5cd3cfa9a62aee49f8130962b4b3b9195e8aa5b7827463722b8c153931579d3505566b4edf48d498e185f0509de15204bb53b8977e5f92a0bc372742c4830944a59b4fe6b1c0466e2a6dad122b5d2e030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd31a76dae6d3272396d0cbe61fced2bc532edac647851e3ac53ce1cc9c7e645a83198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002203e205db4f19b37b60121b83a7333706db86431c6d835849957ed8c3928ad7927dc7234fd11d3e8c36c59277c3e6f149d5cd3cfa9a62aee49f8130962b4b3b9195e8aa5b7827463722b8c153931579d3505566b4edf48d498e185f0509de15204bb53b8977e5f92a0bc372742c4830944a59b4fe6b1c0466e2a6dad122b5d2e030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd31a76dae6d3272396d0cbe61fced2bc532edac647851e3ac53ce1cc9c7e645a83198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa";
        let expect = "0000000000000000000000000000000000000000000000000000000000000001";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_pairing(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..32], buf1[..]);

        let mut buf0 = [0x00; 384];
        let mut buf1 = [0x00; 32];
        let inputs = "105456a333e6d636854f987ea7bb713dfd0ae8371a72aea313ae0c32c0bf10160cf031d41b41557f3e7e3ba0c51bebe5da8e6ecd855ec50fc87efcdeac168bcc0476be093a6d2b4bbf907172049874af11e1b6267606e00804d3ff0037ec57fd3010c68cb50161b7d1d96bb71edfec9880171954e56871abf3d93cc94d745fa114c059d74e5b6c4ec14ae5864ebe23a71781d86c29fb8fb6cce94f70d3de7a2101b33461f39d9e887dbb100f170a2345dde3c07e256d1dfa2b657ba5cd030427000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000021a2c3013d2ea92e13c800cde68ef56a294b883f6ac35d25f587c09b1b3c635f7290158a80cd3d66530f74dc94c94adb88f5cdb481acca997b6e60071f08a115f2f997f3dbd66a7afe07fe7862ce239edba9e05c5afff7f8a1259c9733b2dfbb929d1691530ca701b4a106054688728c9972c8512e9789e9567aae23e302ccd75";
        let expect = "0000000000000000000000000000000000000000000000000000000000000001";
        hex2bin(inputs, &mut buf0[..]);
        assert!(alt_bn128_pairing(&buf0, &mut buf1).is_ok());
        hex2bin(expect, &mut buf0[..]);
        assert_eq!(buf0[0..32], buf1[..]);
    }
}
