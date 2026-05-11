use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::math::{clamp, EPSILON};

/// 二维点。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    /// X 坐标。
    pub x: f64,
    /// Y 坐标。
    pub y: f64,
}

impl Point {
    /// 创建一个二维点。
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// 判断坐标是否都是有限数值。
    #[must_use]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

impl Add<Vector> for Point {
    type Output = Point;

    fn add(self, rhs: Vector) -> Self::Output {
        Point::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub<Point> for Point {
    type Output = Vector;

    fn sub(self, rhs: Point) -> Self::Output {
        Vector::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Sub<Vector> for Point {
    type Output = Point;

    fn sub(self, rhs: Vector) -> Self::Output {
        Point::new(self.x - rhs.x, self.y - rhs.y)
    }
}

/// 二维向量。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vector {
    /// X 分量。
    pub x: f64,
    /// Y 分量。
    pub y: f64,
}

impl Vector {
    /// 创建一个二维向量。
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// 判断分量是否都是有限数值。
    #[must_use]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    /// 返回向量长度。
    #[must_use]
    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    /// 返回向量长度的平方。
    #[must_use]
    pub const fn length_squared(self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    /// 返回两个向量的点积。
    #[must_use]
    pub const fn dot(self, other: Vector) -> f64 {
        self.x * other.x + self.y * other.y
    }

    /// 返回两个向量的二维叉积。
    #[must_use]
    pub const fn cross(self, other: Vector) -> f64 {
        self.x * other.y - self.y * other.x
    }

    /// 返回单位向量；零长度或非有限向量返回 `None`。
    #[must_use]
    pub fn normalized(self) -> Option<Vector> {
        let length = self.length();
        if !length.is_finite() || length <= EPSILON {
            return None;
        }
        Some(self / length)
    }

    /// 返回两个向量之间的夹角，单位为弧度。
    #[must_use]
    pub fn angle_between(self, other: Vector) -> Option<f64> {
        let a = self.normalized()?;
        let b = other.normalized()?;
        Some(clamp(a.dot(b), -1.0, 1.0).acos())
    }
}

impl Add for Vector {
    type Output = Vector;

    fn add(self, rhs: Vector) -> Self::Output {
        Vector::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Vector {
    type Output = Vector;

    fn sub(self, rhs: Vector) -> Self::Output {
        Vector::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f64> for Vector {
    type Output = Vector;

    fn mul(self, rhs: f64) -> Self::Output {
        Vector::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<Vector> for f64 {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Self::Output {
        rhs * self
    }
}

impl Div<f64> for Vector {
    type Output = Vector;

    fn div(self, rhs: f64) -> Self::Output {
        Vector::new(self.x / rhs, self.y / rhs)
    }
}

impl Neg for Vector {
    type Output = Vector;

    fn neg(self) -> Self::Output {
        Vector::new(-self.x, -self.y)
    }
}
