use crate::output::path::{CubicSegment, SmoothPath};
use crate::process::Processor;
use crate::process::corner::{SmoothCornerGeometry, SmoothCornerProcessor, SmoothCornerRequest};
use crate::types::Point;
use crate::utils::EPSILON;

type CornerPath = (SmoothCornerGeometry, Vec<CubicSegment>);

// 生成闭合凸 polygon 的 smooth frame 路径。
pub(super) fn build_smooth_frame_path(points: &[Point], radius: f64, smoothing: f64) -> SmoothPath {
    let corners = collect_corner_paths(points, radius, smoothing);

    if corners
        .iter()
        .all(|(geometry, _)| geometry.radius <= EPSILON)
    {
        return sharp_path(points);
    }

    build_path_from_corners(&corners)
}

// 生成不带圆角的原始闭合路径。
fn sharp_path(points: &[Point]) -> SmoothPath {
    let mut path = SmoothPath::new();
    path.move_to(points[0]);
    for point in points.iter().copied().skip(1) {
        path.line_to(point);
    }
    path.close();
    path
}

// 为每个 frame 顶点计算 smooth corner 几何和 cubic 段。
fn collect_corner_paths(points: &[Point], radius: f64, smoothing: f64) -> Vec<CornerPath> {
    let mut corners = Vec::with_capacity(points.len());
    for index in 0..points.len() {
        let prev = points[(index + points.len() - 1) % points.len()];
        let origin = points[index];
        let next = points[(index + 1) % points.len()];
        let incoming = prev - origin;
        let outgoing = next - origin;
        let incoming_length = incoming.length();
        let outgoing_length = outgoing.length();
        let request = SmoothCornerRequest {
            origin,
            incoming_axis: incoming
                .normalized()
                .expect("输入层已保证 frame 不含退化边"),
            outgoing_axis: outgoing
                .normalized()
                .expect("输入层已保证 frame 不含退化边"),
            radius,
            smoothing,
            incoming_limit: incoming_length / 2.0,
            outgoing_limit: outgoing_length / 2.0,
            angle: incoming
                .angle_between(outgoing)
                .expect("输入层已保证 frame 不含退化角"),
        };
        let processed = SmoothCornerProcessor::new(request).process();
        corners.push((processed.geometry, processed.cubics));
    }
    corners
}

// 将所有 corner 的起止点和 cubic 段拼接成闭合路径。
fn build_path_from_corners(corners: &[CornerPath]) -> SmoothPath {
    let mut path = SmoothPath::new();
    path.move_to(corners[0].0.end);

    for (geometry, cubics) in corners.iter().skip(1) {
        path.line_to(geometry.start);
        push_cubics(&mut path, cubics);
    }

    path.line_to(corners[0].0.start);
    push_cubics(&mut path, &corners[0].1);
    path.close();
    path
}

// 将一组 cubic 段追加到路径命令序列中。
fn push_cubics(path: &mut SmoothPath, cubics: &[CubicSegment]) {
    for cubic in cubics {
        path.cubic_to(cubic.ctrl1, cubic.ctrl2, cubic.to);
    }
}
