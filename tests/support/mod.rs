use smooth_frame::Point;

pub const TOLERANCE: f64 = 1.0e-9;

// 断言两个点在默认测试容差内相等。
#[allow(dead_code)]
pub fn assert_point_close(actual: Point, expected: Point) {
    assert_close(actual.x, expected.x);
    assert_close(actual.y, expected.y);
}

// 断言两个浮点数在默认测试容差内相等。
pub fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= TOLERANCE,
        "actual={actual:?}, expected={expected:?}, diff={:?}",
        (actual - expected).abs()
    );
}
