mod path_builder;
mod validation;

use crate::errors::SmoothError;
use crate::output::path::SmoothPath;
use crate::types::Point;
use crate::utils::clamp01;

use path_builder::build_smooth_frame_path;
use validation::validate_frame;

/// 闭合凸 polygon / frame。
#[derive(Debug, Clone, PartialEq)]
pub struct SmoothFrame {
    points: Vec<Point>,
    radius: f64,
    smoothing: f64,
}

impl SmoothFrame {
    /// 创建闭合 frame。
    ///
    /// 当前版本要求输入点组成非退化凸多边形。
    #[must_use]
    pub fn closed(points: impl IntoIterator<Item = Point>) -> Self {
        Self {
            points: points.into_iter().collect(),
            radius: 0.0,
            smoothing: 0.0,
        }
    }

    /// 设置每个角的核心圆半径。
    #[must_use]
    pub fn with_radius(mut self, radius: f64) -> Self {
        self.radius = radius;
        self
    }

    /// 设置每个角的 Sketch-like smoothing，生成前会 clamp 到 `[0, 1]`。
    #[must_use]
    pub fn with_smoothing(mut self, smoothing: f64) -> Self {
        self.smoothing = smoothing;
        self
    }

    /// 返回 frame 输入点。
    #[must_use]
    pub fn points(&self) -> &[Point] {
        &self.points
    }

    /// 生成闭合 smooth frame 路径。
    pub fn to_path(&self) -> Result<SmoothPath, SmoothError> {
        validate_frame(&self.points)?;

        if self.radius < 0.0 {
            return Err(SmoothError::NegativeInput);
        }
        if !self.radius.is_finite() || !self.smoothing.is_finite() {
            return Err(SmoothError::NonFiniteInput);
        }

        Ok(build_smooth_frame_path(
            &self.points,
            self.radius,
            clamp01(self.smoothing),
        ))
    }
}
