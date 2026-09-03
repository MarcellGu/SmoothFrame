use std::f64::consts::PI;

use crate::utils::clamp01;

const SKETCH_FALLBACK_EFFECTIVE_SMOOTHING: f64 = 0.005;

#[derive(Debug, Clone, Copy)]
pub(super) struct RectAxis {
    pub(super) influence: f64,
    pub(super) alpha: f64,
}

// 按 SketchTool 的规则计算单个轴向上的影响范围和过渡角。
pub(super) fn sketch_rect_axis(radius: f64, smoothing: f64, side: f64) -> RectAxis {
    let raw_influence = (1.0 + smoothing) * radius;
    let limit = side / 2.0;
    let saturated_influence = raw_influence.min(limit);
    let effective_smoothing = clamp01(saturated_influence / radius - 1.0);

    if raw_influence >= limit && effective_smoothing < SKETCH_FALLBACK_EFFECTIVE_SMOOTHING {
        return RectAxis {
            influence: radius.min(limit),
            alpha: 0.0,
        };
    }

    RectAxis {
        influence: saturated_influence,
        alpha: effective_smoothing * PI / 4.0,
    }
}
