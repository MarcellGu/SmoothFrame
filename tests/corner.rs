mod support;

use smooth_frame::{Point, SmoothCorner, Vector};
use std::f64::consts::PI;

use support::assert_close;

// 验证 smoothing 对影响范围和过渡角的基础公式保持稳定。
#[test]
fn smoothing_influence_rule_matches_sketch_formula() {
    for smoothing in [0.0, 0.3, 0.6, 0.8, 1.0] {
        let geometry = SmoothCorner::new(
            Point::new(0.0, 0.0),
            Vector::new(1.0, 0.0),
            Vector::new(0.0, 1.0),
        )
        .with_radius(100.0)
        .with_smoothing(smoothing)
        .with_limits(500.0, 500.0)
        .geometry()
        .unwrap();

        assert_close(geometry.incoming_influence, (1.0 + smoothing) * 100.0);
        assert_close(geometry.outgoing_influence, (1.0 + smoothing) * 100.0);
        assert_close(geometry.alpha0, smoothing * PI / 4.0);
        assert_close(geometry.alpha1, smoothing * PI / 4.0);
    }
}
