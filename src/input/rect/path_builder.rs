use crate::output::path::{CubicSegment, SmoothPath};
use crate::types::{Point, Vector};
use crate::utils::EPSILON;

use super::axis::sketch_rect_axis;
use super::corner::{RectCorner, rect_corner_cubics};

pub(super) fn build_rect_path(width: f64, height: f64, radius: f64, smoothing: f64) -> SmoothPath {
    if width <= EPSILON || height <= EPSILON {
        return collapsed_path();
    }
    if radius <= EPSILON {
        return sharp_rect_path(width, height);
    }
    build_smooth_rect_path(width, height, radius, smoothing)
}

// 生成 SketchTool 对齐的 smooth rect 路径。
fn build_smooth_rect_path(width: f64, height: f64, radius: f64, smoothing: f64) -> SmoothPath {
    let horizontal = sketch_rect_axis(radius, smoothing, width);
    let vertical = sketch_rect_axis(radius, smoothing, height);
    let corners = [
        RectCorner::new(
            Point::new(0.0, 0.0),
            Vector::new(0.0, 1.0),
            vertical,
            Vector::new(1.0, 0.0),
            horizontal,
        ),
        RectCorner::new(
            Point::new(width, 0.0),
            Vector::new(-1.0, 0.0),
            horizontal,
            Vector::new(0.0, 1.0),
            vertical,
        ),
        RectCorner::new(
            Point::new(width, height),
            Vector::new(0.0, -1.0),
            vertical,
            Vector::new(-1.0, 0.0),
            horizontal,
        ),
        RectCorner::new(
            Point::new(0.0, height),
            Vector::new(1.0, 0.0),
            horizontal,
            Vector::new(0.0, -1.0),
            vertical,
        ),
    ];

    let cubics = corners
        .iter()
        .map(|corner| rect_corner_cubics(*corner, radius))
        .collect::<Vec<_>>();

    build_path_from_corners(corners, cubics)
}

// 生成退化尺寸对应的单点闭合路径。
fn collapsed_path() -> SmoothPath {
    let mut path = SmoothPath::new();
    path.move_to(Point::new(0.0, 0.0));
    path.close();
    path
}

// 生成未应用圆角时的普通矩形路径。
fn sharp_rect_path(width: f64, height: f64) -> SmoothPath {
    let mut path = SmoothPath::new();
    path.move_to(Point::new(0.0, 0.0));
    path.line_to(Point::new(width, 0.0));
    path.line_to(Point::new(width, height));
    path.line_to(Point::new(0.0, height));
    path.close();
    path
}

// 将矩形角点和 cubic 数据拼成闭合路径。
fn build_path_from_corners(corners: [RectCorner; 4], cubics: Vec<[CubicSegment; 3]>) -> SmoothPath {
    let mut path = SmoothPath::new();
    let mut current = corners[0].end();
    path.move_to(current);

    for index in 1..corners.len() {
        push_line_if_needed(&mut path, &mut current, corners[index].start());
        push_cubics_if_needed(&mut path, &mut current, &cubics[index]);
    }
    push_line_if_needed(&mut path, &mut current, corners[0].start());
    push_cubics_if_needed(&mut path, &mut current, &cubics[0]);
    path.close();
    path
}

// 只在当前点与目标点不重合时追加直线段。
fn push_line_if_needed(path: &mut SmoothPath, current: &mut Point, to: Point) {
    if !points_close(*current, to, EPSILON) {
        path.line_to(to);
    }
    *current = to;
}

// 追加非零 cubic，并保留中间圆弧段以匹配 SketchTool 结构。
fn push_cubics_if_needed(path: &mut SmoothPath, current: &mut Point, cubics: &[CubicSegment]) {
    for (index, cubic) in cubics.iter().enumerate() {
        if index == 1 || !cubic_is_zero(*cubic) {
            path.cubic_to(cubic.ctrl1, cubic.ctrl2, cubic.to);
        }
        *current = cubic.to;
    }
}

// 判断 cubic 段是否退化到所有点重合。
fn cubic_is_zero(cubic: CubicSegment) -> bool {
    points_close(cubic.from, cubic.ctrl1, EPSILON)
        && points_close(cubic.from, cubic.ctrl2, EPSILON)
        && points_close(cubic.from, cubic.to, EPSILON)
}

// 使用给定容差按坐标判断两个点是否足够接近。
fn points_close(a: Point, b: Point, tolerance: f64) -> bool {
    (a.x - b.x).abs() <= tolerance && (a.y - b.y).abs() <= tolerance
}
