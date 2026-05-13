use crate::types::Point;

pub(crate) const MAX_FORMAT_PRECISION: usize = 12;
pub(crate) const EPSILON: f64 = 1.0e-12;

/// 按指定精度格式化浮点数，并去掉无意义的尾零。
pub fn format_number(value: f64, precision: usize) -> String {
    let precision = bounded_precision(precision);
    let zero_epsilon = 10.0_f64.powi(-(precision as i32 + 1));
    let value = if value.abs() < zero_epsilon {
        0.0
    } else {
        value
    };
    let mut text = format!("{value:.precision$}");
    if text.contains('.') {
        while text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
    }
    if text == "-0" { "0".to_owned() } else { text }
}

/// 按 SVG path 需要的 `x,y` 形式格式化点。
pub fn format_point(point: Point, precision: usize) -> String {
    format!(
        "{},{}",
        format_number(point.x, precision),
        format_number(point.y, precision)
    )
}

// 限制格式化精度，避免公开 API 收到极大 usize 时分配异常大的字符串。
pub(crate) fn bounded_precision(precision: usize) -> usize {
    precision.min(MAX_FORMAT_PRECISION)
}

// 将数值限制到 0 到 1 的闭区间。
pub(crate) fn clamp01(value: f64) -> f64 {
    clamp(value, 0.0, 1.0)
}

// 将数值限制到指定闭区间。
pub(crate) fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}
