use super::h3api;
use h3o::BaseCell;

#[test]
fn value() {
    let result = BaseCell::count();
    let reference = h3api::res0_cell_count();

    assert_eq!(result, reference);
}
