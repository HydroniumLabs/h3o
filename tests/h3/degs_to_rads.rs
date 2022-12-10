use super::h3api;
use float_eq::assert_float_eq;

macro_rules! test {
    ($name:ident, $angle:literal) => {
        #[test]
        fn $name() {
            let result = $angle.to_radians();
            let reference = h3api::degs_to_rads($angle);

            assert_float_eq!(result, reference, abs <= f64::EPSILON);
        }
    };
}

test!(positive, 48.854501508844095_f64);
test!(negative, -48.854501508844095_f64);
test!(large_positive, 448.8545015088441_f64);
test!(large_negative, -448.8545015088441_f64);
