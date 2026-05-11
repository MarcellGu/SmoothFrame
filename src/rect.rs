use std::f64::consts::PI;

use crate::geometry::{Point, Vector};
use crate::math::{clamp01, EPSILON};
use crate::path::{CubicSegment, SmoothPath};

const SKETCH_FALLBACK_EFFECTIVE_SMOOTHING: f64 = 0.005;

/// 矩形便捷封装。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SmoothRect {
    width: f64,
    height: f64,
    radius: f64,
    smoothing: f64,
}

impl SmoothRect {
    /// 创建矩形便捷封装。
    ///
    /// 宽高为非有限数、负数或零时会退化为单点闭合路径。
    #[must_use]
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            radius: 0.0,
            smoothing: 0.0,
        }
    }

    /// 设置矩形角半径。
    ///
    /// 负数或非有限数会按 0 处理，过大的半径会 clamp 到短边的一半。
    #[must_use]
    pub fn with_radius(mut self, radius: f64) -> Self {
        self.radius = radius;
        self
    }

    /// 设置 Sketch-like smoothing。
    ///
    /// 有限数会 clamp 到 `[0, 1]`，非有限数会按 0 处理。
    #[must_use]
    pub fn with_smoothing(mut self, smoothing: f64) -> Self {
        self.smoothing = smoothing;
        self
    }

    /// 生成矩形 smooth corner 路径。
    #[must_use]
    pub fn to_path(&self) -> SmoothPath {
        let width = sanitize_dimension(self.width);
        let height = sanitize_dimension(self.height);
        let requested_radius = sanitize_dimension(self.radius);
        let smoothing = if self.smoothing.is_finite() {
            clamp01(self.smoothing)
        } else {
            0.0
        };

        if width <= EPSILON || height <= EPSILON {
            let mut path = SmoothPath::new();
            path.move_to(Point::new(0.0, 0.0));
            path.close();
            return path;
        }

        let radius = requested_radius.min(width.min(height) / 2.0);
        if radius <= EPSILON {
            let mut path = SmoothPath::new();
            path.move_to(Point::new(0.0, 0.0));
            path.line_to(Point::new(width, 0.0));
            path.line_to(Point::new(width, height));
            path.line_to(Point::new(0.0, height));
            path.close();
            return path;
        }

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
}

#[derive(Debug, Clone, Copy)]
struct RectAxis {
    influence: f64,
    alpha: f64,
}

#[derive(Debug, Clone, Copy)]
struct RectCorner {
    origin: Point,
    incoming_axis: Vector,
    incoming: RectAxis,
    outgoing_axis: Vector,
    outgoing: RectAxis,
}

impl RectCorner {
    fn new(
        origin: Point,
        incoming_axis: Vector,
        incoming: RectAxis,
        outgoing_axis: Vector,
        outgoing: RectAxis,
    ) -> Self {
        Self {
            origin,
            incoming_axis,
            incoming,
            outgoing_axis,
            outgoing,
        }
    }

    fn start(self) -> Point {
        self.origin + self.incoming_axis * self.incoming.influence
    }

    fn end(self) -> Point {
        self.origin + self.outgoing_axis * self.outgoing.influence
    }
}

fn sketch_rect_axis(radius: f64, smoothing: f64, side: f64) -> RectAxis {
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

fn rect_corner_cubics(corner: RectCorner, radius: f64) -> [CubicSegment; 3] {
    let alpha0 = corner.incoming.alpha;
    let alpha1 = corner.outgoing.alpha;
    let influence0 = corner.incoming.influence;
    let influence1 = corner.outgoing.influence;

    let tangent0 = radius - radius * (alpha0 / 2.0).tan();
    let handle0 = (influence0 - tangent0) / 3.0;
    let tangent1 = radius - radius * (alpha1 / 2.0).tan();
    let handle1 = (influence1 - tangent1) / 3.0;

    let p1 = corner.origin
        + corner.incoming_axis * (radius - radius * alpha0.sin())
        + corner.outgoing_axis * (radius - radius * alpha0.cos());
    let p2 = corner.origin
        + corner.incoming_axis * (radius - radius * alpha1.cos())
        + corner.outgoing_axis * (radius - radius * alpha1.sin());

    let middle_arc_angle = (PI / 2.0 - alpha0 - alpha1).max(0.0);
    let arc_handle = if middle_arc_angle <= EPSILON {
        0.0
    } else {
        (4.0 / 3.0) * (middle_arc_angle / 4.0).tan() * radius
    };

    [
        CubicSegment {
            from: corner.start(),
            ctrl1: corner.origin + corner.incoming_axis * (influence0 - 2.0 * handle0),
            ctrl2: corner.origin + corner.incoming_axis * tangent0,
            to: p1,
        },
        CubicSegment {
            from: p1,
            ctrl1: p1
                + corner.incoming_axis * (-arc_handle * alpha0.cos())
                + corner.outgoing_axis * (arc_handle * alpha0.sin()),
            ctrl2: p2
                + corner.incoming_axis * (arc_handle * alpha1.sin())
                + corner.outgoing_axis * (-arc_handle * alpha1.cos()),
            to: p2,
        },
        CubicSegment {
            from: p2,
            ctrl1: corner.origin + corner.outgoing_axis * tangent1,
            ctrl2: corner.origin + corner.outgoing_axis * (tangent1 + handle1),
            to: corner.end(),
        },
    ]
}

fn push_line_if_needed(path: &mut SmoothPath, current: &mut Point, to: Point) {
    if !points_close(*current, to, EPSILON) {
        path.line_to(to);
    }
    *current = to;
}

fn push_cubics_if_needed(path: &mut SmoothPath, current: &mut Point, cubics: &[CubicSegment]) {
    for (index, cubic) in cubics.iter().enumerate() {
        if index == 1 || !cubic_is_zero(*cubic) {
            path.cubic_to(cubic.ctrl1, cubic.ctrl2, cubic.to);
        }
        *current = cubic.to;
    }
}

fn cubic_is_zero(cubic: CubicSegment) -> bool {
    points_close(cubic.from, cubic.ctrl1, EPSILON)
        && points_close(cubic.from, cubic.ctrl2, EPSILON)
        && points_close(cubic.from, cubic.to, EPSILON)
}

fn points_close(a: Point, b: Point, tolerance: f64) -> bool {
    (a.x - b.x).abs() <= tolerance && (a.y - b.y).abs() <= tolerance
}

fn sanitize_dimension(value: f64) -> f64 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        0.0
    }
}
