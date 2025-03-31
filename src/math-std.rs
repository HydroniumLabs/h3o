#[inline]
pub const fn abs(x: f64) -> f64 {
    x.abs()
}

#[inline]
pub fn sin(x: f64) -> f64 {
    x.sin()
}

#[inline]
pub fn cos(x: f64) -> f64 {
    x.cos()
}

#[inline]
pub fn tan(x: f64) -> f64 {
    x.tan()
}

#[inline]
pub fn asin(x: f64) -> f64 {
    x.asin()
}

#[inline]
pub fn acos(x: f64) -> f64 {
    x.acos()
}

#[inline]
pub fn atan(x: f64) -> f64 {
    x.atan()
}

#[inline]
pub fn atan2(y: f64, x: f64) -> f64 {
    y.atan2(x)
}

#[inline]
pub fn hypot(x: f64, y: f64) -> f64 {
    x.hypot(y)
}

#[inline]
pub fn sqrt(x: f64) -> f64 {
    x.sqrt()
}

#[inline]
pub fn round(x: f64) -> f64 {
    x.round()
}

#[inline]
pub fn mul_add(a: f64, b: f64, c: f64) -> f64 {
    a.mul_add(b, c)
}
