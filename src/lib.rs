//! Sketch-like smooth corner / squircle frame 的 cubic Bezier 几何库。
//!
//! 低层类型 [`SmoothCorner`] 支持任意凸角；[`SmoothFrame`] 负责闭合凸多边形；
//! [`SmoothRect`] 只是矩形便捷封装。

use std::error::Error;
use std::f64::consts::PI;
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

const EPSILON: f64 = 1.0e-12;
const SKETCH_FALLBACK_EFFECTIVE_SMOOTHING: f64 = 0.005;

/// 二维点。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

impl Add<Vector> for Point {
    type Output = Point;

    fn add(self, rhs: Vector) -> Self::Output {
        Point::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub<Point> for Point {
    type Output = Vector;

    fn sub(self, rhs: Point) -> Self::Output {
        Vector::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Sub<Vector> for Point {
    type Output = Point;

    fn sub(self, rhs: Vector) -> Self::Output {
        Point::new(self.x - rhs.x, self.y - rhs.y)
    }
}

/// 二维向量。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
}

impl Vector {
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    #[must_use]
    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    #[must_use]
    pub const fn length_squared(self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    #[must_use]
    pub const fn dot(self, other: Vector) -> f64 {
        self.x * other.x + self.y * other.y
    }

    #[must_use]
    pub const fn cross(self, other: Vector) -> f64 {
        self.x * other.y - self.y * other.x
    }

    #[must_use]
    pub fn normalized(self) -> Option<Vector> {
        let length = self.length();
        if !length.is_finite() || length <= EPSILON {
            return None;
        }
        Some(self / length)
    }

    #[must_use]
    pub fn angle_between(self, other: Vector) -> Option<f64> {
        let a = self.normalized()?;
        let b = other.normalized()?;
        Some(clamp(a.dot(b), -1.0, 1.0).acos())
    }
}

impl Add for Vector {
    type Output = Vector;

    fn add(self, rhs: Vector) -> Self::Output {
        Vector::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Vector {
    type Output = Vector;

    fn sub(self, rhs: Vector) -> Self::Output {
        Vector::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f64> for Vector {
    type Output = Vector;

    fn mul(self, rhs: f64) -> Self::Output {
        Vector::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<Vector> for f64 {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Self::Output {
        rhs * self
    }
}

impl Div<f64> for Vector {
    type Output = Vector;

    fn div(self, rhs: f64) -> Self::Output {
        Vector::new(self.x / rhs, self.y / rhs)
    }
}

impl Neg for Vector {
    type Output = Vector;

    fn neg(self) -> Self::Output {
        Vector::new(-self.x, -self.y)
    }
}

/// 一段 cubic Bezier，包含起点，便于直接映射到底层渲染 API。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CubicSegment {
    pub from: Point,
    pub ctrl1: Point,
    pub ctrl2: Point,
    pub to: Point,
}

/// 可直接映射到 SVG Canvas Skia 等 API 的路径命令。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathCommand {
    MoveTo(Point),
    LineTo(Point),
    CubicTo {
        ctrl1: Point,
        ctrl2: Point,
        to: Point,
    },
    Close,
}

/// 平滑路径。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SmoothPath {
    commands: Vec<PathCommand>,
}

impl SmoothPath {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn commands(&self) -> &[PathCommand] {
        &self.commands
    }

    #[must_use]
    pub fn cubics(&self) -> Vec<CubicSegment> {
        let mut cubics = Vec::new();
        let mut current = None;
        let mut subpath_start = None;

        for command in &self.commands {
            match *command {
                PathCommand::MoveTo(point) => {
                    current = Some(point);
                    subpath_start = Some(point);
                }
                PathCommand::LineTo(point) => {
                    current = Some(point);
                }
                PathCommand::CubicTo { ctrl1, ctrl2, to } => {
                    if let Some(from) = current {
                        cubics.push(CubicSegment {
                            from,
                            ctrl1,
                            ctrl2,
                            to,
                        });
                    }
                    current = Some(to);
                }
                PathCommand::Close => {
                    current = subpath_start;
                }
            }
        }

        cubics
    }

    pub fn move_to(&mut self, point: Point) {
        self.commands.push(PathCommand::MoveTo(point));
    }

    pub fn line_to(&mut self, point: Point) {
        self.commands.push(PathCommand::LineTo(point));
    }

    pub fn cubic_to(&mut self, ctrl1: Point, ctrl2: Point, to: Point) {
        self.commands
            .push(PathCommand::CubicTo { ctrl1, ctrl2, to });
    }

    pub fn close(&mut self) {
        self.commands.push(PathCommand::Close);
    }

    #[must_use]
    pub fn to_svg_path(&self) -> String {
        self.to_svg_path_with_precision(6)
    }

    #[must_use]
    pub fn to_svg_path_with_precision(&self, precision: usize) -> String {
        let mut parts = Vec::with_capacity(self.commands.len());
        for command in &self.commands {
            match *command {
                PathCommand::MoveTo(point) => {
                    parts.push(format!("M {}", format_point(point, precision)));
                }
                PathCommand::LineTo(point) => {
                    parts.push(format!("L {}", format_point(point, precision)));
                }
                PathCommand::CubicTo { ctrl1, ctrl2, to } => {
                    parts.push(format!(
                        "C {} {} {}",
                        format_point(ctrl1, precision),
                        format_point(ctrl2, precision),
                        format_point(to, precision)
                    ));
                }
                PathCommand::Close => parts.push("Z".to_owned()),
            }
        }
        parts.join(" ")
    }
}

/// 几何计算错误。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmoothError {
    NonFiniteInput,
    NegativeInput,
    DegenerateAxis,
    InvalidAngle,
    TooFewPoints,
    DegenerateFrame,
    ConcaveFrame,
    SelfIntersectingFrame,
}

impl fmt::Display for SmoothError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SmoothError::NonFiniteInput => write!(f, "输入包含非有限数值"),
            SmoothError::NegativeInput => write!(f, "长度、半径或限制不能为负数"),
            SmoothError::DegenerateAxis => write!(f, "角点方向向量退化"),
            SmoothError::InvalidAngle => write!(f, "角点角度必须满足 0 < phi < PI"),
            SmoothError::TooFewPoints => write!(f, "闭合 frame 至少需要 3 个点"),
            SmoothError::DegenerateFrame => write!(f, "frame 包含退化边或面积为零"),
            SmoothError::ConcaveFrame => write!(f, "当前版本仅支持凸 frame"),
            SmoothError::SelfIntersectingFrame => write!(f, "当前版本不支持自相交 frame"),
        }
    }
}

impl Error for SmoothError {}

/// 单个 smooth corner 解析后的参数，便于测试或调试 Sketch 对齐。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SmoothCornerGeometry {
    pub origin: Point,
    pub incoming_axis: Vector,
    pub outgoing_axis: Vector,
    pub radius: f64,
    pub smoothing: f64,
    pub incoming_limit: f64,
    pub outgoing_limit: f64,
    pub angle: f64,
    pub base_tangent: f64,
    pub incoming_influence: f64,
    pub outgoing_influence: f64,
    pub alpha0: f64,
    pub alpha1: f64,
    pub middle_arc_angle: f64,
    pub start: Point,
    pub end: Point,
}

/// 任意凸角的 Sketch-like smooth corner。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SmoothCorner {
    origin: Point,
    incoming_axis: Vector,
    outgoing_axis: Vector,
    radius: f64,
    smoothing: f64,
    incoming_limit: f64,
    outgoing_limit: f64,
}

impl SmoothCorner {
    #[must_use]
    pub fn new(origin: Point, incoming_axis: Vector, outgoing_axis: Vector) -> Self {
        Self {
            origin,
            incoming_axis,
            outgoing_axis,
            radius: 0.0,
            smoothing: 0.0,
            incoming_limit: f64::INFINITY,
            outgoing_limit: f64::INFINITY,
        }
    }

    #[must_use]
    pub fn with_radius(mut self, radius: f64) -> Self {
        self.radius = radius;
        self
    }

    #[must_use]
    pub fn with_smoothing(mut self, smoothing: f64) -> Self {
        self.smoothing = smoothing;
        self
    }

    #[must_use]
    pub fn with_limits(mut self, incoming_limit: f64, outgoing_limit: f64) -> Self {
        self.incoming_limit = incoming_limit;
        self.outgoing_limit = outgoing_limit;
        self
    }

    pub fn geometry(&self) -> Result<SmoothCornerGeometry, SmoothError> {
        if !self.origin.is_finite()
            || !self.incoming_axis.is_finite()
            || !self.outgoing_axis.is_finite()
            || !self.radius.is_finite()
            || !self.smoothing.is_finite()
            || self.incoming_limit.is_nan()
            || self.outgoing_limit.is_nan()
        {
            return Err(SmoothError::NonFiniteInput);
        }
        if self.radius < 0.0 || self.incoming_limit < 0.0 || self.outgoing_limit < 0.0 {
            return Err(SmoothError::NegativeInput);
        }

        let incoming_axis = self
            .incoming_axis
            .normalized()
            .ok_or(SmoothError::DegenerateAxis)?;
        let outgoing_axis = self
            .outgoing_axis
            .normalized()
            .ok_or(SmoothError::DegenerateAxis)?;
        let angle = incoming_axis
            .angle_between(outgoing_axis)
            .ok_or(SmoothError::DegenerateAxis)?;

        if angle <= EPSILON || PI - angle <= EPSILON {
            return Err(SmoothError::InvalidAngle);
        }

        let smoothing = clamp01(self.smoothing);
        let max_limit = self.incoming_limit.min(self.outgoing_limit);
        let max_radius = max_limit * (angle / 2.0).tan();
        let radius = self.radius.min(max_radius);

        if radius <= EPSILON {
            return Ok(SmoothCornerGeometry {
                origin: self.origin,
                incoming_axis,
                outgoing_axis,
                radius: 0.0,
                smoothing,
                incoming_limit: self.incoming_limit,
                outgoing_limit: self.outgoing_limit,
                angle,
                base_tangent: 0.0,
                incoming_influence: 0.0,
                outgoing_influence: 0.0,
                alpha0: 0.0,
                alpha1: 0.0,
                middle_arc_angle: angle,
                start: self.origin,
                end: self.origin,
            });
        }

        let base_tangent = radius / (angle / 2.0).tan();
        let raw_influence = (1.0 + smoothing) * base_tangent;
        let incoming_influence = raw_influence.min(self.incoming_limit);
        let outgoing_influence = raw_influence.min(self.outgoing_limit);

        let alpha0 = clamp01(incoming_influence / base_tangent - 1.0) * angle / 2.0;
        let alpha1 = clamp01(outgoing_influence / base_tangent - 1.0) * angle / 2.0;
        let middle_arc_angle = (angle - alpha0 - alpha1).max(0.0);

        Ok(SmoothCornerGeometry {
            origin: self.origin,
            incoming_axis,
            outgoing_axis,
            radius,
            smoothing,
            incoming_limit: self.incoming_limit,
            outgoing_limit: self.outgoing_limit,
            angle,
            base_tangent,
            incoming_influence,
            outgoing_influence,
            alpha0,
            alpha1,
            middle_arc_angle,
            start: self.origin + incoming_axis * incoming_influence,
            end: self.origin + outgoing_axis * outgoing_influence,
        })
    }

    pub fn to_cubics(&self) -> Result<Vec<CubicSegment>, SmoothError> {
        let geometry = self.geometry()?;
        if geometry.radius <= EPSILON {
            return Ok(Vec::new());
        }

        let center = geometry.origin
            + (geometry.incoming_axis + geometry.outgoing_axis)
                .normalized()
                .ok_or(SmoothError::InvalidAngle)?
                * (geometry.radius / (geometry.angle / 2.0).sin());
        let incoming_tangent = geometry.origin + geometry.incoming_axis * geometry.base_tangent;
        let e0 = (incoming_tangent - center) / geometry.radius;
        let e1 = -geometry.incoming_axis;

        let p1 = circle_point(center, geometry.radius, e0, e1, geometry.alpha0);
        let p2 = circle_point(
            center,
            geometry.radius,
            e0,
            e1,
            geometry.angle - geometry.alpha1,
        );

        let tangent0 = geometry.base_tangent - geometry.radius * (geometry.alpha0 / 2.0).tan();
        let handle0 = (geometry.incoming_influence - tangent0) / 3.0;

        let tangent1 = geometry.base_tangent - geometry.radius * (geometry.alpha1 / 2.0).tan();
        let handle1 = (geometry.outgoing_influence - tangent1) / 3.0;

        let arc_handle = if geometry.middle_arc_angle <= EPSILON {
            0.0
        } else {
            (4.0 / 3.0) * (geometry.middle_arc_angle / 4.0).tan() * geometry.radius
        };

        let arc_tangent0 = circle_tangent(e0, e1, geometry.alpha0);
        let arc_tangent1 = circle_tangent(e0, e1, geometry.angle - geometry.alpha1);

        let c1 = CubicSegment {
            from: geometry.start,
            ctrl1: geometry.origin
                + geometry.incoming_axis * (geometry.incoming_influence - 2.0 * handle0),
            ctrl2: geometry.origin + geometry.incoming_axis * tangent0,
            to: p1,
        };
        let c2 = CubicSegment {
            from: p1,
            ctrl1: p1 + arc_tangent0 * arc_handle,
            ctrl2: p2 - arc_tangent1 * arc_handle,
            to: p2,
        };
        let c3 = CubicSegment {
            from: p2,
            ctrl1: geometry.origin + geometry.outgoing_axis * tangent1,
            ctrl2: geometry.origin + geometry.outgoing_axis * (tangent1 + handle1),
            to: geometry.end,
        };

        Ok(vec![c1, c2, c3])
    }
}

/// 闭合凸 polygon / frame。
#[derive(Debug, Clone, PartialEq)]
pub struct SmoothFrame {
    points: Vec<Point>,
    radius: f64,
    smoothing: f64,
}

impl SmoothFrame {
    #[must_use]
    pub fn closed(points: impl IntoIterator<Item = Point>) -> Self {
        Self {
            points: points.into_iter().collect(),
            radius: 0.0,
            smoothing: 0.0,
        }
    }

    #[must_use]
    pub fn with_radius(mut self, radius: f64) -> Self {
        self.radius = radius;
        self
    }

    #[must_use]
    pub fn with_smoothing(mut self, smoothing: f64) -> Self {
        self.smoothing = smoothing;
        self
    }

    #[must_use]
    pub fn points(&self) -> &[Point] {
        &self.points
    }

    pub fn to_path(&self) -> Result<SmoothPath, SmoothError> {
        self.validate_frame()?;

        if self.radius < 0.0 {
            return Err(SmoothError::NegativeInput);
        }
        if !self.radius.is_finite() || !self.smoothing.is_finite() {
            return Err(SmoothError::NonFiniteInput);
        }

        if self.radius <= EPSILON {
            return Ok(self.sharp_path());
        }

        let mut corners = Vec::with_capacity(self.points.len());
        for index in 0..self.points.len() {
            let prev = self.points[(index + self.points.len() - 1) % self.points.len()];
            let origin = self.points[index];
            let next = self.points[(index + 1) % self.points.len()];
            let incoming = prev - origin;
            let outgoing = next - origin;
            let incoming_length = incoming.length();
            let outgoing_length = outgoing.length();
            let corner = SmoothCorner::new(origin, incoming, outgoing)
                .with_radius(self.radius)
                .with_smoothing(self.smoothing)
                .with_limits(incoming_length / 2.0, outgoing_length / 2.0);
            corners.push((corner.geometry()?, corner.to_cubics()?));
        }

        if corners
            .iter()
            .all(|(geometry, _)| geometry.radius <= EPSILON)
        {
            return Ok(self.sharp_path());
        }

        let mut path = SmoothPath::new();
        path.move_to(corners[0].0.end);

        for (geometry, cubics) in corners.iter().skip(1) {
            path.line_to(geometry.start);
            push_cubics(&mut path, cubics);
        }

        path.line_to(corners[0].0.start);
        push_cubics(&mut path, &corners[0].1);
        path.close();

        Ok(path)
    }

    fn sharp_path(&self) -> SmoothPath {
        let mut path = SmoothPath::new();
        path.move_to(self.points[0]);
        for point in self.points.iter().copied().skip(1) {
            path.line_to(point);
        }
        path.close();
        path
    }

    fn validate_frame(&self) -> Result<(), SmoothError> {
        if self.points.len() < 3 {
            return Err(SmoothError::TooFewPoints);
        }
        if self.points.iter().any(|point| !point.is_finite()) {
            return Err(SmoothError::NonFiniteInput);
        }

        let mut area = 0.0;
        for index in 0..self.points.len() {
            let a = self.points[index];
            let b = self.points[(index + 1) % self.points.len()];
            if (b - a).length() <= EPSILON {
                return Err(SmoothError::DegenerateFrame);
            }
            area += a.x * b.y - b.x * a.y;
        }
        if area.abs() <= EPSILON {
            return Err(SmoothError::DegenerateFrame);
        }
        if has_self_intersection(&self.points) {
            return Err(SmoothError::SelfIntersectingFrame);
        }

        let mut turn_sign = 0.0_f64;
        for index in 0..self.points.len() {
            let prev = self.points[(index + self.points.len() - 1) % self.points.len()];
            let origin = self.points[index];
            let next = self.points[(index + 1) % self.points.len()];
            let incoming = (prev - origin)
                .normalized()
                .ok_or(SmoothError::DegenerateFrame)?;
            let outgoing = (next - origin)
                .normalized()
                .ok_or(SmoothError::DegenerateFrame)?;
            let cross = incoming.cross(outgoing);
            if cross.abs() <= EPSILON {
                return Err(SmoothError::DegenerateFrame);
            }
            let sign = cross.signum();
            if turn_sign == 0.0 {
                turn_sign = sign;
            } else if sign != turn_sign {
                return Err(SmoothError::ConcaveFrame);
            }
        }

        Ok(())
    }
}

/// 矩形便捷封装。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SmoothRect {
    width: f64,
    height: f64,
    radius: f64,
    smoothing: f64,
}

impl SmoothRect {
    #[must_use]
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            radius: 0.0,
            smoothing: 0.0,
        }
    }

    #[must_use]
    pub fn with_radius(mut self, radius: f64) -> Self {
        self.radius = radius;
        self
    }

    #[must_use]
    pub fn with_smoothing(mut self, smoothing: f64) -> Self {
        self.smoothing = smoothing;
        self
    }

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

fn push_cubics(path: &mut SmoothPath, cubics: &[CubicSegment]) {
    for cubic in cubics {
        path.cubic_to(cubic.ctrl1, cubic.ctrl2, cubic.to);
    }
}

fn circle_point(center: Point, radius: f64, e0: Vector, e1: Vector, angle: f64) -> Point {
    center + e0 * (radius * angle.cos()) + e1 * (radius * angle.sin())
}

fn circle_tangent(e0: Vector, e1: Vector, angle: f64) -> Vector {
    -e0 * angle.sin() + e1 * angle.cos()
}

fn sanitize_dimension(value: f64) -> f64 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        0.0
    }
}

fn has_self_intersection(points: &[Point]) -> bool {
    for first in 0..points.len() {
        let first_start = points[first];
        let first_end = points[(first + 1) % points.len()];

        for second in (first + 1)..points.len() {
            if are_adjacent_edges(first, second, points.len()) {
                continue;
            }

            let second_start = points[second];
            let second_end = points[(second + 1) % points.len()];
            if segments_intersect(first_start, first_end, second_start, second_end) {
                return true;
            }
        }
    }

    false
}

fn are_adjacent_edges(first: usize, second: usize, count: usize) -> bool {
    first == second || (first + 1) % count == second || (second + 1) % count == first
}

fn segments_intersect(a: Point, b: Point, c: Point, d: Point) -> bool {
    let ab_c = orientation(a, b, c);
    let ab_d = orientation(a, b, d);
    let cd_a = orientation(c, d, a);
    let cd_b = orientation(c, d, b);

    if ab_c.abs() <= EPSILON && point_on_segment(c, a, b) {
        return true;
    }
    if ab_d.abs() <= EPSILON && point_on_segment(d, a, b) {
        return true;
    }
    if cd_a.abs() <= EPSILON && point_on_segment(a, c, d) {
        return true;
    }
    if cd_b.abs() <= EPSILON && point_on_segment(b, c, d) {
        return true;
    }

    ab_c.signum() != ab_d.signum() && cd_a.signum() != cd_b.signum()
}

fn orientation(a: Point, b: Point, c: Point) -> f64 {
    (b - a).cross(c - a)
}

fn point_on_segment(point: Point, start: Point, end: Point) -> bool {
    point.x >= start.x.min(end.x) - EPSILON
        && point.x <= start.x.max(end.x) + EPSILON
        && point.y >= start.y.min(end.y) - EPSILON
        && point.y <= start.y.max(end.y) + EPSILON
}

fn points_close(a: Point, b: Point, tolerance: f64) -> bool {
    (a.x - b.x).abs() <= tolerance && (a.y - b.y).abs() <= tolerance
}

fn clamp01(value: f64) -> f64 {
    clamp(value, 0.0, 1.0)
}

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

fn format_point(point: Point, precision: usize) -> String {
    format!(
        "{},{}",
        format_number(point.x, precision),
        format_number(point.y, precision)
    )
}

fn format_number(value: f64, precision: usize) -> String {
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
