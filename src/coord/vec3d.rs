/// 3D floating-point vector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3d {
    /// `x` component.
    pub x: f64,
    /// `y` component.
    pub y: f64,
    /// `z` component.
    pub z: f64,
}

impl Vec3d {
    /// Initializes a new 3D vector with the specified component values.
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Computes the square of the distance between two 3D coordinates.
    pub fn distance(&self, other: &Self) -> f64 {
        let x_diff = self.x - other.x;
        let y_diff = self.y - other.y;
        let z_diff = self.z - other.z;

        x_diff.mul_add(x_diff, y_diff.mul_add(y_diff, z_diff * z_diff))
    }
}

#[cfg(test)]
#[path = "./vec3d_tests.rs"]
mod tests;
