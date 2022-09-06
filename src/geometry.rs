use num;
use std::ops::{Add, AddAssign, Div, Mul, Sub};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vec2D<T: num::Float> {
    pub x: T,
    pub y: T,
}

// TODO(tslnc04): allow for integer vectors
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

    pub fn new_random<Dx, Dy, R>(x_distro: Dx, y_distro: Dy, rng: &mut R) -> Self
    where
        Dx: rand::distributions::Distribution<T>,
        Dy: rand::distributions::Distribution<T>,
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

    pub fn is_in_bounds(&self, pos: Self, dim: Self) -> bool {
        self.x >= pos.x && self.x <= pos.x + dim.x && self.y >= pos.y && self.y <= pos.y + dim.y
    }

    /// Finds which quadrant of the rectangular bounds the vector is in.
    /// Quadrants are numbered by the following system:
    /// +---+---+
    /// | 0 | 1 |
    /// +---+---+
    /// | 2 | 3 |
    /// +---+---+
    /// (0,0) => 2
    /// (0,1) => 0
    /// (1,0) => 3
    /// (1,1) => 1
    pub fn get_bounds_quadrant(&self, pos: Self, dim: Self) -> usize {
        let x = if self.x < pos.x + dim.x / T::from(2.0).unwrap() {
            0
        } else {
            1
        };
        let y = if self.y < pos.y + dim.y / T::from(2.0).unwrap() {
            0
        } else {
            1
        };
        2 - 2 * y + x
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
