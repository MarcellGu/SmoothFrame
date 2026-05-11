use std::f64::consts::PI;

use crate::error::SmoothError;
use crate::geometry::{Point, Vector};
use crate::math::{clamp01, EPSILON};
use crate::path::CubicSegment;

/// 单个 smooth corner 解析后的参数，便于测试或调试 Sketch 对齐。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SmoothCornerGeometry {
    /// 原始角点。
    pub origin: Point,
    /// 从角点指向上一条边的单位方向。
    pub incoming_axis: Vector,
    /// 从角点指向下一条边的单位方向。
    pub outgoing_axis: Vector,
    /// clamp 后实际使用的核心圆半径。
    pub radius: f64,
    /// clamp 到 `[0, 1]` 后的平滑系数。
    pub smoothing: f64,
    /// incoming 方向的最大影响范围。
    pub incoming_limit: f64,
    /// outgoing 方向的最大影响范围。
    pub outgoing_limit: f64,
    /// incoming 与 outgoing 方向之间的夹角，单位为弧度。
    pub angle: f64,
    /// 核心圆在未平滑时的切点距离。
    pub base_tangent: f64,
    /// incoming 方向最终影响范围。
    pub incoming_influence: f64,
    /// outgoing 方向最终影响范围。
    pub outgoing_influence: f64,
    /// incoming 侧过渡角，单位为弧度。
    pub alpha0: f64,
    /// outgoing 侧过渡角，单位为弧度。
    pub alpha1: f64,
    /// 中间圆弧段角度，单位为弧度。
    pub middle_arc_angle: f64,
    /// 角点 smooth cubic 的起点。
    pub start: Point,
    /// 角点 smooth cubic 的终点。
    pub end: Point,
}

/// 任意凸角的 Sketch-like smooth corner。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SmoothCorner {
    origin: Point,
    incoming_axis: Vector,
    outgoing_axis: Vector,
    radius: f64,
    smoothing: f64,
    incoming_limit: f64,
    outgoing_limit: f64,
}

impl SmoothCorner {
    /// 创建一个任意凸角的 smooth corner。
    ///
    /// `incoming_axis` 从角点指向上一条边，`outgoing_axis` 从角点指向下一条边。
    #[must_use]
    pub fn new(origin: Point, incoming_axis: Vector, outgoing_axis: Vector) -> Self {
        Self {
            origin,
            incoming_axis,
            outgoing_axis,
            radius: 0.0,
            smoothing: 0.0,
            incoming_limit: f64::INFINITY,
            outgoing_limit: f64::INFINITY,
        }
    }

    /// 设置核心圆半径。
    #[must_use]
    pub fn with_radius(mut self, radius: f64) -> Self {
        self.radius = radius;
        self
    }

    /// 设置 Sketch-like smoothing，计算时会 clamp 到 `[0, 1]`。
    #[must_use]
    pub fn with_smoothing(mut self, smoothing: f64) -> Self {
        self.smoothing = smoothing;
        self
    }

    /// 设置 incoming / outgoing 两侧可占用的最大长度。
    #[must_use]
    pub fn with_limits(mut self, incoming_limit: f64, outgoing_limit: f64) -> Self {
        self.incoming_limit = incoming_limit;
        self.outgoing_limit = outgoing_limit;
        self
    }

    /// 解析 smooth corner 的几何参数。
    pub fn geometry(&self) -> Result<SmoothCornerGeometry, SmoothError> {
        if !self.origin.is_finite()
            || !self.incoming_axis.is_finite()
            || !self.outgoing_axis.is_finite()
            || !self.radius.is_finite()
            || !self.smoothing.is_finite()
            || self.incoming_limit.is_nan()
            || self.outgoing_limit.is_nan()
        {
            return Err(SmoothError::NonFiniteInput);
        }
        if self.radius < 0.0 || self.incoming_limit < 0.0 || self.outgoing_limit < 0.0 {
            return Err(SmoothError::NegativeInput);
        }

        let incoming_axis = self
            .incoming_axis
            .normalized()
            .ok_or(SmoothError::DegenerateAxis)?;
        let outgoing_axis = self
            .outgoing_axis
            .normalized()
            .ok_or(SmoothError::DegenerateAxis)?;
        let angle = incoming_axis
            .angle_between(outgoing_axis)
            .ok_or(SmoothError::DegenerateAxis)?;

        if angle <= EPSILON || PI - angle <= EPSILON {
            return Err(SmoothError::InvalidAngle);
        }

        let smoothing = clamp01(self.smoothing);
        let max_limit = self.incoming_limit.min(self.outgoing_limit);
        let max_radius = max_limit * (angle / 2.0).tan();
        let radius = self.radius.min(max_radius);

        if radius <= EPSILON {
            return Ok(SmoothCornerGeometry {
                origin: self.origin,
                incoming_axis,
                outgoing_axis,
                radius: 0.0,
                smoothing,
                incoming_limit: self.incoming_limit,
                outgoing_limit: self.outgoing_limit,
                angle,
                base_tangent: 0.0,
                incoming_influence: 0.0,
                outgoing_influence: 0.0,
                alpha0: 0.0,
                alpha1: 0.0,
                middle_arc_angle: angle,
                start: self.origin,
                end: self.origin,
            });
        }

        let base_tangent = radius / (angle / 2.0).tan();
        let raw_influence = (1.0 + smoothing) * base_tangent;
        let incoming_influence = raw_influence.min(self.incoming_limit);
        let outgoing_influence = raw_influence.min(self.outgoing_limit);

        let alpha0 = clamp01(incoming_influence / base_tangent - 1.0) * angle / 2.0;
        let alpha1 = clamp01(outgoing_influence / base_tangent - 1.0) * angle / 2.0;
        let middle_arc_angle = (angle - alpha0 - alpha1).max(0.0);

        Ok(SmoothCornerGeometry {
            origin: self.origin,
            incoming_axis,
            outgoing_axis,
            radius,
            smoothing,
            incoming_limit: self.incoming_limit,
            outgoing_limit: self.outgoing_limit,
            angle,
            base_tangent,
            incoming_influence,
            outgoing_influence,
            alpha0,
            alpha1,
            middle_arc_angle,
            start: self.origin + incoming_axis * incoming_influence,
            end: self.origin + outgoing_axis * outgoing_influence,
        })
    }

    /// 生成最多 3 段 cubic Bezier。
    ///
    /// 半径为 0 或被限制压缩到 0 时返回空数组。
    pub fn to_cubics(&self) -> Result<Vec<CubicSegment>, SmoothError> {
        let geometry = self.geometry()?;
        if geometry.radius <= EPSILON {
            return Ok(Vec::new());
        }

        let center = geometry.origin
            + (geometry.incoming_axis + geometry.outgoing_axis)
                .normalized()
                .ok_or(SmoothError::InvalidAngle)?
                * (geometry.radius / (geometry.angle / 2.0).sin());
        let incoming_tangent = geometry.origin + geometry.incoming_axis * geometry.base_tangent;
        let e0 = (incoming_tangent - center) / geometry.radius;
        let e1 = -geometry.incoming_axis;

        let p1 = circle_point(center, geometry.radius, e0, e1, geometry.alpha0);
        let p2 = circle_point(
            center,
            geometry.radius,
            e0,
            e1,
            geometry.angle - geometry.alpha1,
        );

        let tangent0 = geometry.base_tangent - geometry.radius * (geometry.alpha0 / 2.0).tan();
        let handle0 = (geometry.incoming_influence - tangent0) / 3.0;

        let tangent1 = geometry.base_tangent - geometry.radius * (geometry.alpha1 / 2.0).tan();
        let handle1 = (geometry.outgoing_influence - tangent1) / 3.0;

        let arc_handle = if geometry.middle_arc_angle <= EPSILON {
            0.0
        } else {
            (4.0 / 3.0) * (geometry.middle_arc_angle / 4.0).tan() * geometry.radius
        };

        let arc_tangent0 = circle_tangent(e0, e1, geometry.alpha0);
        let arc_tangent1 = circle_tangent(e0, e1, geometry.angle - geometry.alpha1);

        let c1 = CubicSegment {
            from: geometry.start,
            ctrl1: geometry.origin
                + geometry.incoming_axis * (geometry.incoming_influence - 2.0 * handle0),
            ctrl2: geometry.origin + geometry.incoming_axis * tangent0,
            to: p1,
        };
        let c2 = CubicSegment {
            from: p1,
            ctrl1: p1 + arc_tangent0 * arc_handle,
            ctrl2: p2 - arc_tangent1 * arc_handle,
            to: p2,
        };
        let c3 = CubicSegment {
            from: p2,
            ctrl1: geometry.origin + geometry.outgoing_axis * tangent1,
            ctrl2: geometry.origin + geometry.outgoing_axis * (tangent1 + handle1),
            to: geometry.end,
        };

        Ok(vec![c1, c2, c3])
    }
}

fn circle_point(center: Point, radius: f64, e0: Vector, e1: Vector, angle: f64) -> Point {
    center + e0 * (radius * angle.cos()) + e1 * (radius * angle.sin())
}

fn circle_tangent(e0: Vector, e1: Vector, angle: f64) -> Vector {
    -e0 * angle.sin() + e1 * angle.cos()
}
