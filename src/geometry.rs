use num;
use std::ops::{Add, AddAssign, Div, Mul, Sub};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vec2D<T: num::Float> {
    pub x: T,
    pub y: T,
}

impl<T: num::Float> Vec2D<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x: x, y: y }
    }

    pub fn new_nan() -> Self {
        Self {
            x: T::nan(),
            y: T::nan(),
        }
    }

    pub fn new_random<D, R>(x_distro: D, y_distro: D, rng: &mut R) -> Self
    where
        D: rand::distributions::Distribution<T>,
        R: rand::Rng,
    {
        Self {
            x: x_distro.sample(rng),
            y: y_distro.sample(rng),
        }
    }

    /// Returns whether any of the components are NaN
    pub fn is_nan(&self) -> bool {
        self.x.is_nan() || self.y.is_nan()
    }

    pub fn mag(&self) -> T {
        self.dot(self).sqrt()
    }

    pub fn dist(&self, other: Self) -> T {
        (*self - other).mag()
    }

    pub fn normalize(&self) -> Self {
        self.div(self.mag())
    }

    pub fn dot(&self, other: &Self) -> T {
        self.x * other.x + self.y * other.y
    }

    pub fn clamp_mag(&self, max: T) -> Self {
        if self.mag() > max {
            self.normalize() * max
        } else {
            *self
        }
    }
}

// Vector addition
impl<T: Add<Output = T> + num::Float> Add for Vec2D<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

// Vector addition and assignment
impl<T: Add<Output = T> + num::Float> AddAssign for Vec2D<T> {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

// Vector subtraction
impl<T: Sub<Output = T> + num::Float> Sub for Vec2D<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

// Computes the Hadamard product of two vectors
impl<T: Mul<Output = T> + num::Float> Mul for Vec2D<T> {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

// Vector by scalar multiplication
impl<T: Mul<Output = T> + num::Float> Mul<T> for Vec2D<T> {
    type Output = Self;

    fn mul(self, other: T) -> Self::Output {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

// Hadamard division of two vectors
impl<T: Div<Output = T> + num::Float> Div for Vec2D<T> {
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        Self {
            x: self.x / other.x,
            y: self.y / other.y,
        }
    }
}

// Vector by scalar division
impl<T: Div<Output = T> + num::Float> Div<T> for Vec2D<T> {
    type Output = Self;

    fn div(self, other: T) -> Self::Output {
        Self {
            x: self.x / other,
            y: self.y / other,
        }
    }
}
