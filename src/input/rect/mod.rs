mod axis;
mod corner;
mod path_builder;

use crate::output::path::SmoothPath;
use crate::utils::clamp01;

use path_builder::build_rect_path;

/// 矩形便捷封装。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SmoothRect {
    width: f64,
    height: f64,
    radius: f64,
    smoothing: f64,
}

impl SmoothRect {
    /// 创建矩形便捷封装。
    ///
    /// 宽高为非有限数、负数或零时会退化为单点闭合路径。
    #[must_use]
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            radius: 0.0,
            smoothing: 0.0,
        }
    }

    /// 设置矩形角半径。
    ///
    /// 负数或非有限数会按 0 处理，过大的半径会 clamp 到短边的一半。
    #[must_use]
    pub fn with_radius(mut self, radius: f64) -> Self {
        self.radius = radius;
        self
    }

    /// 设置 Sketch-like smoothing。
    ///
    /// 有限数会 clamp 到 `[0, 1]`，非有限数会按 0 处理。
    #[must_use]
    pub fn with_smoothing(mut self, smoothing: f64) -> Self {
        self.smoothing = smoothing;
        self
    }

    /// 生成矩形 smooth corner 路径。
    #[must_use]
    pub fn to_path(&self) -> SmoothPath {
        let width = sanitize_dimension(self.width);
        let height = sanitize_dimension(self.height);
        let requested_radius = sanitize_dimension(self.radius);
        let radius = requested_radius.min(width.min(height) / 2.0);
        let smoothing = if self.smoothing.is_finite() {
            clamp01(self.smoothing)
        } else {
            0.0
        };

        build_rect_path(width, height, radius, smoothing)
    }
}

// 将负数或非有限尺寸按 0 处理。
fn sanitize_dimension(value: f64) -> f64 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        0.0
    }
}
