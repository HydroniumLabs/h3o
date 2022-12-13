use super::*;
use float_eq::assert_float_eq;

#[test]
fn magnitude() {
    let v = Vec2d::new(3.0, 4.0);
    let expected = 5.0;

    let result = v.magnitude();

    assert_float_eq!(result, expected, abs <= f64::EPSILON);
}

#[test]
fn intersection() {
    let line1 = (Vec2d::new(2.0, 2.0), Vec2d::new(6.0, 6.0));
    let line2 = (Vec2d::new(0.0, 4.0), Vec2d::new(10.0, 4.0));
    let expected = Vec2d::new(4.0, 4.0);

    let result = Vec2d::intersection(line1, line2);

    assert_float_eq!(
        result.x,
        expected.x,
        abs <= f64::EPSILON,
        "x as expected"
    );
    assert_float_eq!(
        result.y,
        expected.y,
        abs <= f64::EPSILON,
        "y as expected"
    );
}
