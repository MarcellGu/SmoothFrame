use crate::types::{Point, Vector};

// 根据局部圆坐标系和角度计算圆弧上的点。
pub(super) fn circle_point(
    center: Point,
    radius: f64,
    e0: Vector,
    e1: Vector,
    angle: f64,
) -> Point {
    center + e0 * (radius * angle.cos()) + e1 * (radius * angle.sin())
}

// 根据局部圆坐标系和角度计算圆弧切线方向。
pub(super) fn circle_tangent(e0: Vector, e1: Vector, angle: f64) -> Vector {
    -e0 * angle.sin() + e1 * angle.cos()
}
