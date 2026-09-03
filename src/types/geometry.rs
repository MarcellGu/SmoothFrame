use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::utils::{EPSILON, clamp};

/// 二维点。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    /// X 坐标。
    pub x: f64,
    /// Y 坐标。
    pub y: f64,
}

/// 二维向量。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vector {
    /// X 分量。
    pub x: f64,
    /// Y 分量。
    pub y: f64,
}

impl Point {
    /// 创建一个点。
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

/// 点加向量，得到平移后的点
impl Add<Vector> for Point {
    type Output = Point;

    // 执行点加向量的平移运算。
    fn add(self, rhs: Vector) -> Self::Output {
        Point::new(self.x + rhs.x, self.y + rhs.y)
    }
}

/// 两个点相减，得到从右侧点指向左侧点的向量。
impl Sub<Point> for Point {
    type Output = Vector;

    // 计算两个点之间的位移向量。
    fn sub(self, rhs: Point) -> Self::Output {
        Vector::new(self.x - rhs.x, self.y - rhs.y)
    }
}

/// 点减向量，得到反向平移后的点。
impl Sub<Vector> for Point {
    type Output = Point;

    // 执行点减向量的反向平移运算。
    fn sub(self, rhs: Vector) -> Self::Output {
        Point::new(self.x - rhs.x, self.y - rhs.y)
    }
}

/// 两个向量相加，得到叠加后的合成向量。
impl Add for Vector {
    type Output = Vector;

    // 按分量相加两个向量。
    fn add(self, rhs: Vector) -> Self::Output {
        Vector::new(self.x + rhs.x, self.y + rhs.y)
    }
}

/// 两个向量相减，得到左侧向量相对右侧向量的差向量
impl Sub for Vector {
    type Output = Vector;

    // 按分量相减两个向量。
    fn sub(self, rhs: Vector) -> Self::Output {
        Vector::new(self.x - rhs.x, self.y - rhs.y)
    }
}

/// 向量乘以标量，得到按该倍数缩放后的向量。
impl Mul<f64> for Vector {
    type Output = Vector;

    // 将向量按标量倍数缩放。
    fn mul(self, rhs: f64) -> Self::Output {
        Vector::new(self.x * rhs, self.y * rhs)
    }
}

/// 标量乘以向量，得到按该倍数缩放后的向量。
impl Mul<Vector> for f64 {
    type Output = Vector;

    // 支持标量在左侧时的向量缩放写法。
    fn mul(self, rhs: Vector) -> Self::Output {
        rhs * self
    }
}

/// 向量除以标量，得到按该标量倒数缩放后的向量。
impl Div<f64> for Vector {
    type Output = Vector;

    // 将向量按标量倒数缩放。
    fn div(self, rhs: f64) -> Self::Output {
        Vector::new(self.x / rhs, self.y / rhs)
    }
}

/// 对向量取负，得到方向相反、长度相同的向量。
impl Neg for Vector {
    type Output = Vector;

    // 翻转向量方向。
    fn neg(self) -> Self::Output {
        Vector::new(-self.x, -self.y)
    }
}
