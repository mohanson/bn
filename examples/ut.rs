mod ut_common;

fn main() {
    ut_common::test_alt_bn128_add();
    ut_common::test_alt_bn128_mul();
    ut_common::test_alt_bn128_pairing();
}
