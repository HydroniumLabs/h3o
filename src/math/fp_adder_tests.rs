use super::*;

#[test]
fn small_increments() {
    let values = vec![1e-10_f64; 1_000_000];
    let naive: f64 = values.iter().copied().sum();
    let expected = 1e-4;

    let mut adder = FloatAdder::default();
    for value in values {
        adder += value;
    }
    let result = f64::from(adder);

    let naive_error = (expected - naive).abs();
    let adder_error = (expected - result).abs();

    assert!(adder_error < naive_error);
}
