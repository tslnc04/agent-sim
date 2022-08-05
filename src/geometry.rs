use num;
use std::ops::{Add, Div, Mul, Sub};

pub fn mag(x: (f64, f64)) -> f64 {
    (x.0 * x.0 + x.1 * x.1).sqrt()
}

pub fn dist(x: (f64, f64), y: (f64, f64)) -> f64 {
    mag((x.0 - y.0, x.1 - y.1))
}

pub fn normalize(x: (f64, f64)) -> (f64, f64) {
    let x_mag = mag(x);
    (x.0 / x_mag, x.1 / x_mag)
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vec2D<T: num::Float> {
    pub x: T,
    pub y: T,
}

impl<T: num::Float> Vec2D<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x: x, y: y }
    }

    // TODO(tslnc04): convert mag, dist, and normalize to use operator
    // overloading rather than the deprecated functions

    pub fn mag(&self) -> T {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn dist(&self, other: Self) -> T {
        self.sub(other).mag()
    }

    pub fn normalize(&self) -> Self {
        self.div(self.mag())
    }

    /// Deprecated: use operator overloading instead
    pub fn add(&self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }

    /// Deprecated: use operator overloading instead
    pub fn sub(&self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }

    /// Deprecated: use operator overloading instead
    pub fn div(&self, other: T) -> Self {
        Self::new(self.x / other, self.y / other)
    }
}

impl<T: Add<Output = T> + num::Float> Add for Vec2D<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T: Sub<Output = T> + num::Float> Sub for Vec2D<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

// TODO(tslnc04): figure out how to do scalar by vector multiplication
impl<T: Mul<Output = T> + num::Float> Mul for Vec2D<T> {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

impl<T: Div<Output = T> + num::Float> Div for Vec2D<T> {
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        Self {
            x: self.x / other.x,
            y: self.y / other.y,
        }
    }
}
