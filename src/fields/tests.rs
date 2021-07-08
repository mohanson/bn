use super::FieldElement;

fn can_invert<F: FieldElement>() {
    let mut a = F::one();

    for _ in 0..10000 {
        assert_eq!(a * a.inverse().unwrap(), F::one());

        a = a + F::one();
    }

    a = -F::one();
    for _ in 0..10000 {
        assert_eq!(a * a.inverse().unwrap(), F::one());

        a = a - F::one();
    }

    assert_eq!(F::zero().inverse(), None);
}
