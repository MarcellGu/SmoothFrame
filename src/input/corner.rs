use std::f64::consts::PI;

use crate::errors::SmoothError;
use crate::output::path::CubicSegment;
use crate::process::Processor;
use crate::process::corner::{SmoothCornerGeometry, SmoothCornerProcessor, SmoothCornerRequest};
use crate::types::{Point, Vector};
use crate::utils::{EPSILON, clamp01};

/// 任意凸角的 Sketch-like smooth corner 输入入口。
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

    /// 设置 Sketch-like smoothing，生成前会 clamp 到 `[0, 1]`。
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
        Ok(SmoothCornerProcessor::new(self.to_request()?)
            .process()
            .geometry)
    }

    /// 生成最多 3 段 cubic Bezier。
    ///
    /// 半径为 0 或被限制压缩到 0 时返回空数组。
    pub fn to_cubics(&self) -> Result<Vec<CubicSegment>, SmoothError> {
        Ok(SmoothCornerProcessor::new(self.to_request()?)
            .process()
            .cubics)
    }

    // 校验输入并转换成处理层使用的规范化请求。
    fn to_request(&self) -> Result<SmoothCornerRequest, SmoothError> {
        validate_corner_inputs(self)?;

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

        Ok(SmoothCornerRequest {
            origin: self.origin,
            incoming_axis,
            outgoing_axis,
            radius: self.radius,
            smoothing: clamp01(self.smoothing),
            incoming_limit: self.incoming_limit,
            outgoing_limit: self.outgoing_limit,
            angle,
        })
    }
}

// 校验 smooth corner 输入是否为有限且非负。
fn validate_corner_inputs(corner: &SmoothCorner) -> Result<(), SmoothError> {
    if !corner.origin.is_finite()
        || !corner.incoming_axis.is_finite()
        || !corner.outgoing_axis.is_finite()
        || !corner.radius.is_finite()
        || !corner.smoothing.is_finite()
        || corner.incoming_limit.is_nan()
        || corner.outgoing_limit.is_nan()
    {
        return Err(SmoothError::NonFiniteInput);
    }
    if corner.radius < 0.0 || corner.incoming_limit < 0.0 || corner.outgoing_limit < 0.0 {
        return Err(SmoothError::NegativeInput);
    }
    Ok(())
}
