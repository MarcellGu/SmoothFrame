use smooth_frame::{Point, SmoothError, SmoothFrame};

// 验证非矩形凸四边形按每角三段 cubic 输出。
#[test]
fn convex_polygon_frame_is_supported() {
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

// 验证三角形 frame 按三个角生成九段 cubic。
#[test]
fn convex_triangle_frame_is_supported() {
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

// 验证交替内外半径的凹星形轮廓返回凹多边形错误。
#[test]
fn concave_star_frame_returns_error() {
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

// 验证经典五角星路径返回自相交错误。
#[test]
fn self_intersecting_star_frame_returns_error() {
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

// 验证普通凹多边形返回凹多边形错误。
#[test]
fn concave_frame_returns_error() {
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
