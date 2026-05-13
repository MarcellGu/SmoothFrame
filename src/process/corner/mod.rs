mod arc;
mod cubics;
mod geometry;

use crate::output::path::CubicSegment;
use crate::process::Processor;
use crate::types::{Point, Vector};

use cubics::build_cubics;
use geometry::resolve_geometry;

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SmoothCornerRequest {
    pub(crate) origin: Point,
    pub(crate) incoming_axis: Vector,
    pub(crate) outgoing_axis: Vector,
    pub(crate) radius: f64,
    pub(crate) smoothing: f64,
    pub(crate) incoming_limit: f64,
    pub(crate) outgoing_limit: f64,
    pub(crate) angle: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProcessedSmoothCorner {
    pub(crate) geometry: SmoothCornerGeometry,
    pub(crate) cubics: Vec<CubicSegment>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SmoothCornerProcessor {
    request: SmoothCornerRequest,
}

impl SmoothCornerProcessor {
    pub(crate) fn new(request: SmoothCornerRequest) -> Self {
        Self { request }
    }
}

impl Processor for SmoothCornerProcessor {
    type Output = ProcessedSmoothCorner;

    fn process(&self) -> Self::Output {
        let geometry = resolve_geometry(self.request);
        let cubics = build_cubics(geometry);
        ProcessedSmoothCorner { geometry, cubics }
    }
}
