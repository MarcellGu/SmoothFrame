use squircle_frame::{
    PathCommand, Point, SmoothCorner, SmoothError, SmoothFrame, SmoothRect, Vector,
};
use std::f64::consts::PI;

const TOLERANCE: f64 = 1.0e-9;

#[test]
fn radius_zero_outputs_plain_rect() {
    // 半径为 0 时输出普通矩形，不生成 cubic。
    let path = SmoothRect::new(1000.0, 500.0)
        .with_radius(0.0)
        .with_smoothing(0.6)
        .to_path();

    assert_eq!(path.cubics().len(), 0);
    assert_eq!(
        path.commands(),
        &[
            PathCommand::MoveTo(Point::new(0.0, 0.0)),
            PathCommand::LineTo(Point::new(1000.0, 0.0)),
            PathCommand::LineTo(Point::new(1000.0, 500.0)),
            PathCommand::LineTo(Point::new(0.0, 500.0)),
            PathCommand::Close,
        ]
    );
}

#[test]
fn sketchtool_reference_1000_r100_s06() {
    // 这些常量来自 Sketch-like 90 度控制点规律；用来约束 sketchtool 导出结果
    // 与本库结果只存在浮点级误差。
    let path = SmoothRect::new(1000.0, 1000.0)
        .with_radius(100.0)
        .with_smoothing(0.6)
        .to_path();
    let cubics = path.cubics();

    assert_eq!(cubics.len(), 12);
    assert_point_close(cubics[0].from, Point::new(840.0, 0.0));
    assert_point_close(cubics[0].ctrl1, Point::new(896.005250605341, 0.0));
    assert_point_close(cubics[0].ctrl2, Point::new(924.007875908012, 0.0));
    assert_point_close(cubics[0].to, Point::new(945.399049973955, 10.899347581163));

    assert_point_close(
        cubics[1].ctrl1,
        Point::new(964.215259261833, 20.486685076350),
    );
    assert_point_close(
        cubics[1].ctrl2,
        Point::new(979.513314923650, 35.784740738167),
    );
    assert_point_close(cubics[1].to, Point::new(989.100652418837, 54.600950026045));

    assert_point_close(cubics[2].ctrl1, Point::new(1000.0, 75.992124091988));
    assert_point_close(cubics[2].ctrl2, Point::new(1000.0, 103.994749394659));
    assert_point_close(cubics[2].to, Point::new(1000.0, 160.0));
}

#[test]
fn saturated_radius_400_matches_sketchtool_zero_length_edge_omission() {
    // r=400 且 smoothing=0.6 时 influence 被边长一半压缩，SketchTool 会省略零长度边。
    let path = SmoothRect::new(1000.0, 1000.0)
        .with_radius(400.0)
        .with_smoothing(0.6)
        .to_path();

    assert_eq!(path.cubics().len(), 12);
    assert_eq!(path.commands().len(), 14);
    assert_eq!(
        path.commands()[0],
        PathCommand::MoveTo(Point::new(500.0, 0.0))
    );
}

#[test]
fn short_axis_saturation_remains_continuous() {
    // 1000x500 的短轴在最大半径处退化为 SketchTool 的 capsule-like 结构。
    let path = SmoothRect::new(1000.0, 500.0)
        .with_radius(250.0)
        .with_smoothing(0.6)
        .to_path();

    assert_eq!(path.cubics().len(), 8);
    assert_eq!(path.commands().len(), 12);
    assert_eq!(
        path.commands()[0],
        PathCommand::MoveTo(Point::new(400.0, 0.0))
    );
    assert!(path.cubics().iter().all(|c| c.from.is_finite()
        && c.ctrl1.is_finite()
        && c.ctrl2.is_finite()
        && c.to.is_finite()));
}

#[test]
fn max_radius_matches_sketchtool_four_cubic_circle() {
    // 最大半径时以 SketchTool 为准，退化为 4-cubic circle。
    let path = SmoothRect::new(1000.0, 1000.0)
        .with_radius(500.0)
        .with_smoothing(0.6)
        .to_path();

    assert_eq!(path.cubics().len(), 4);
    assert_eq!(path.commands().len(), 6);
    assert_eq!(
        path.commands()[0],
        PathCommand::MoveTo(Point::new(500.0, 0.0))
    );
}

#[test]
fn near_max_radius_matches_sketchtool_round_rect_fallback() {
    // SketchTool 在近圆区会提前退化为普通圆角结构。
    let path = SmoothRect::new(1000.0, 1000.0)
        .with_radius(498.0)
        .with_smoothing(0.6)
        .to_path();

    assert_eq!(path.cubics().len(), 4);
    assert_eq!(path.commands().len(), 10);
    assert_eq!(
        path.commands()[0],
        PathCommand::MoveTo(Point::new(498.0, 0.0))
    );
}

#[test]
fn smoothing_influence_rule_matches_sketch_formula() {
    // 未饱和时 A=(1+s)*r，alpha=s*PI/4。
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

#[test]
fn convex_polygon_frame_is_supported() {
    // 非矩形凸四边形同样按每角 3 段 cubic 输出。
    let path = SmoothFrame::closed([
        Point::new(0.0, 0.0),
        Point::new(220.0, 30.0),
        Point::new(180.0, 170.0),
        Point::new(20.0, 140.0),
    ])
    .with_radius(24.0)
    .with_smoothing(0.5)
    .to_path()
    .unwrap();

    assert_eq!(path.cubics().len(), 12);
}

#[test]
fn convex_triangle_frame_is_supported() {
    // 三角形是最小闭合凸 frame，每角 3 段 cubic，总计 9 段。
    let path = SmoothFrame::closed([
        Point::new(40.0, 0.0),
        Point::new(180.0, 30.0),
        Point::new(80.0, 150.0),
    ])
    .with_radius(18.0)
    .with_smoothing(0.6)
    .to_path()
    .unwrap();

    assert_eq!(path.cubics().len(), 9);
    assert_eq!(path.commands().len(), 14);
    assert!(path.cubics().iter().all(|c| c.from.is_finite()
        && c.ctrl1.is_finite()
        && c.ctrl2.is_finite()
        && c.to.is_finite()));
}

#[test]
fn concave_star_frame_returns_error() {
    // 交替内外半径形成的星形轮廓包含凹角，v1 明确拒绝。
    let result = SmoothFrame::closed([
        Point::new(100.0, 0.0),
        Point::new(124.0, 68.0),
        Point::new(196.0, 72.0),
        Point::new(138.0, 114.0),
        Point::new(158.0, 184.0),
        Point::new(100.0, 142.0),
        Point::new(42.0, 184.0),
        Point::new(62.0, 114.0),
        Point::new(4.0, 72.0),
        Point::new(76.0, 68.0),
    ])
    .with_radius(12.0)
    .with_smoothing(0.6)
    .to_path();

    assert_eq!(result.unwrap_err(), SmoothError::ConcaveFrame);
}

#[test]
fn self_intersecting_star_frame_returns_error() {
    // 经典五角星路径是自相交 polygon，不属于 v1 支持的闭合凸 frame。
    let result = SmoothFrame::closed([
        Point::new(100.0, 0.0),
        Point::new(158.0, 184.0),
        Point::new(4.0, 72.0),
        Point::new(196.0, 72.0),
        Point::new(42.0, 184.0),
    ])
    .with_radius(12.0)
    .with_smoothing(0.6)
    .to_path();

    assert_eq!(result.unwrap_err(), SmoothError::SelfIntersectingFrame);
}

#[test]
fn concave_frame_returns_error() {
    // v1 对凹角返回错误，API 仍保留后续扩展空间。
    let result = SmoothFrame::closed([
        Point::new(0.0, 0.0),
        Point::new(100.0, 0.0),
        Point::new(40.0, 40.0),
        Point::new(100.0, 100.0),
        Point::new(0.0, 100.0),
    ])
    .with_radius(12.0)
    .with_smoothing(0.6)
    .to_path();

    assert_eq!(result.unwrap_err(), SmoothError::ConcaveFrame);
}

#[test]
fn radius_sweep_0_to_500_matches_sketchtool_structure() {
    // 逐一扫描 0..=500，覆盖 SketchTool 的 smooth、零长度边省略和近圆 fallback 区间。
    for radius in 0..=500 {
        let path = SmoothRect::new(1000.0, 1000.0)
            .with_radius(radius as f64)
            .with_smoothing(0.6)
            .to_path();

        if radius == 0 {
            assert_eq!(path.cubics().len(), 0, "radius={radius}");
            assert_eq!(path.commands().len(), 5, "radius={radius}");
        } else if radius <= 312 {
            assert_eq!(path.cubics().len(), 12, "radius={radius}");
            assert_eq!(path.commands().len(), 18, "radius={radius}");
        } else if radius <= 497 {
            assert_eq!(path.cubics().len(), 12, "radius={radius}");
            assert_eq!(path.commands().len(), 14, "radius={radius}");
        } else if radius <= 499 {
            assert_eq!(path.cubics().len(), 4, "radius={radius}");
            assert_eq!(path.commands().len(), 10, "radius={radius}");
        } else {
            assert_eq!(path.cubics().len(), 4, "radius={radius}");
            assert_eq!(path.commands().len(), 6, "radius={radius}");
        }

        assert!(
            path.cubics().iter().all(|c| c.from.is_finite()
                && c.ctrl1.is_finite()
                && c.ctrl2.is_finite()
                && c.to.is_finite()),
            "radius={radius}"
        );
    }
}

#[test]
fn svg_path_is_stable_without_trailing_zeroes() {
    let svg = SmoothRect::new(1000.0, 500.0)
        .with_radius(0.0)
        .to_path()
        .to_svg_path_with_precision(3);

    assert_eq!(svg, "M 0,0 L 1000,0 L 1000,500 L 0,500 Z");
    assert!(!svg.contains(".000"));
}

fn assert_point_close(actual: Point, expected: Point) {
    assert_close(actual.x, expected.x);
    assert_close(actual.y, expected.y);
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= TOLERANCE,
        "actual={actual:?}, expected={expected:?}, diff={:?}",
        (actual - expected).abs()
    );
}
