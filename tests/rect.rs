mod support;

use smooth_frame::{PathCommand, Point, SmoothRect};

use support::assert_point_close;

// 验证半径为 0 时输出普通矩形且不生成 cubic。
#[test]
fn radius_zero_outputs_plain_rect() {
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

// 验证 1000x1000 r100 s0.6 的参考控制点保持稳定。
#[test]
fn sketchtool_reference_1000_r100_s06() {
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

// 验证大半径饱和时 SketchTool 的零长度边省略结构保持一致。
#[test]
fn saturated_radius_400_matches_sketchtool_zero_length_edge_omission() {
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

// 验证短轴达到最大半径时退化结构仍保持连续。
#[test]
fn short_axis_saturation_remains_continuous() {
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

// 验证正方形最大半径时退化为 SketchTool 的四段 cubic 圆。
#[test]
fn max_radius_matches_sketchtool_four_cubic_circle() {
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

// 验证近最大半径时提前进入普通圆角 fallback 结构。
#[test]
fn near_max_radius_matches_sketchtool_round_rect_fallback() {
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

// 验证 0 到 500 半径扫描下的命令结构和有限数输出。
#[test]
fn radius_sweep_0_to_500_matches_sketchtool_structure() {
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
