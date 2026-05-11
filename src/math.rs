pub(crate) const EPSILON: f64 = 1.0e-12;

pub(crate) fn clamp01(value: f64) -> f64 {
    clamp(value, 0.0, 1.0)
}

pub(crate) fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}
