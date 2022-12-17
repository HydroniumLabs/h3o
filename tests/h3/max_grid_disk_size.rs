use super::h3api;

macro_rules! test {
    ($name:ident, $k:literal) => {
        #[test]
        fn $name() {
            let result = h3o::max_grid_disk_size($k);
            let reference = h3api::max_grid_disk_size($k);

            assert_eq!(result, reference);
        }
    };
}

test!(zero, 0);
test!(one, 1);
test!(many, 42);
