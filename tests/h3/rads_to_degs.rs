use super::h3api;
use float_eq::assert_float_eq;

macro_rules! test {
    ($name:ident, $angle:literal) => {
        #[test]
        fn $name() {
            let result = $angle.to_degrees();
            let reference = h3api::rads_to_degs($angle);

            assert_float_eq!(result, reference, abs <= f64::EPSILON);
        }
    };
}

test!(positive, 0.8526719057477519_f64);
test!(negative, -0.8526719057477519_f64);
test!(large_positive, 7.833988913707753_f64);
test!(large_negative, -7.833988913707753_f64);
