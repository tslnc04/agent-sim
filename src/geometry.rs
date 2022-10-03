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
        Self { x, y }
    }

    pub fn new_nan() -> Self {
        Self {
            x: T::nan(),
            y: T::nan(),
        }
    }

    pub fn new_zero() -> Self {
        Self {
            x: T::zero(),
            y: T::zero(),
        }
    }

    pub fn new_one() -> Self {
        Self {
            x: T::one(),
            y: T::one(),
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

    /// intersects checks if there is any overlap between the two bounding
    /// boxes, even if one is completely inside the other.
    pub fn intersects(pos_1: Self, dim_1: Self, pos_2: Self, dim_2: Self) -> bool {
        let x_overlap = (pos_1.x + dim_1.x > pos_2.x) && (pos_1.x < pos_2.x + dim_2.x);
        let y_overlap = (pos_1.y + dim_1.y > pos_2.y) && (pos_1.y < pos_2.y + dim_2.y);
        x_overlap && y_overlap
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

// Vector minus scalar subtraction
impl<T: Sub<Output = T> + num::Float> Sub<T> for Vec2D<T> {
    type Output = Self;

    fn sub(self, other: T) -> Self::Output {
        Self {
            x: self.x - other,
            y: self.y - other,
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

/// Rect represents a 2D rectangle given the position of two corners. Intended
/// for use as a bounding box. Internally, the first corner is in the bottom
/// left and the second corner is in the top right. This means the first corner
/// has the smallest x and y values and the second corner has the largest x and
/// y values.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Rect<T: num::Float> {
    pub bl: Vec2D<T>,
    pub tr: Vec2D<T>,
}

impl<T: num::Float> Rect<T> {
    /// Creates a new Rect from two corners, regardless of which corners they
    /// are on the rectangle. The right corners for the internal representation
    /// will be figured out.
    pub fn new(corner1: Vec2D<T>, corner2: Vec2D<T>) -> Self {
        let min_x = corner1.x.min(corner2.x);
        let max_x = corner1.x.max(corner2.x);
        let min_y = corner1.y.min(corner2.y);
        let max_y = corner1.y.max(corner2.y);

        Self {
            bl: Vec2D::new(min_x, min_y),
            tr: Vec2D::new(max_x, max_y),
        }
    }

    /// Creates a new Rect from a center and side lengths
    pub fn new_centered(center: Vec2D<T>, side_lengths: Vec2D<T>) -> Self {
        let half_side_lengths = side_lengths / T::from(2.0).unwrap();
        Self::new(center - half_side_lengths, center + half_side_lengths)
    }

    pub fn center(&self) -> Vec2D<T> {
        (self.bl + self.tr) / T::from(2.0).unwrap()
    }

    /// Checks if the rectangle contains a point
    pub fn contains(&self, point: Vec2D<T>) -> bool {
        point.x >= self.bl.x && point.x <= self.tr.x && point.y >= self.bl.y && point.y <= self.tr.y
    }

    /// Checks if there is overlap between this rectangle and another
    pub fn intersects(&self, other: Self) -> bool {
        // If the right side of one rectangle is to the left of the other
        // rectangle or if the bottom edge of one rectangle is above the top
        // edge of the other, then there isn't overlap. Otherwise, there is.
        !(self.bl.x > other.tr.x
            || other.bl.x > self.tr.x
            || self.bl.y > other.tr.y
            || other.bl.y > self.tr.y)
    }

    /// Finds which quadrant of the rectangle a point is in. This will not fail
    /// even if the point is not in the rectangle, instead pretending the
    /// quadrants extend outward from the rectangle to infinity.
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
    pub fn get_quadrant(&self, point: Vec2D<T>) -> usize {
        let center = self.center();
        let x = if point.x < center.x { 0 } else { 1 };
        let y = if point.y < center.y { 0 } else { 1 };
        2 - 2 * y + x
    }

    pub fn get_width(&self) -> T {
        self.tr.x - self.bl.x
    }

    pub fn get_height(&self) -> T {
        self.tr.y - self.bl.y
    }

    /// Returns a slice of rectangles representing the four quadrants of this
    /// rectangle, with overlapping edges. The quadrants are numbered as
    /// determined by the get_quadrant function, with matching indices in the
    /// resultant slice.
    pub fn quarter(&self) -> [Rect<T>; 4] {
        let center = self.center();
        [
            Rect::new(
                Vec2D::new(self.bl.x, center.y),
                Vec2D::new(center.x, self.tr.y),
            ),
            Rect::new(center, self.tr),
            Rect::new(self.bl, center),
            Rect::new(
                Vec2D::new(center.x, self.bl.y),
                Vec2D::new(self.tr.x, center.y),
            ),
        ]
    }
}
