use crate::corner::SmoothCorner;
use crate::error::SmoothError;
use crate::geometry::Point;
use crate::math::EPSILON;
use crate::path::{CubicSegment, SmoothPath};

/// 闭合凸 polygon / frame。
#[derive(Debug, Clone, PartialEq)]
pub struct SmoothFrame {
    points: Vec<Point>,
    radius: f64,
    smoothing: f64,
}

impl SmoothFrame {
    /// 创建闭合 frame。
    ///
    /// 当前版本要求输入点组成非退化凸多边形。
    #[must_use]
    pub fn closed(points: impl IntoIterator<Item = Point>) -> Self {
        Self {
            points: points.into_iter().collect(),
            radius: 0.0,
            smoothing: 0.0,
        }
    }

    /// 设置每个角的核心圆半径。
    #[must_use]
    pub fn with_radius(mut self, radius: f64) -> Self {
        self.radius = radius;
        self
    }

    /// 设置每个角的 Sketch-like smoothing，计算时会 clamp 到 `[0, 1]`。
    #[must_use]
    pub fn with_smoothing(mut self, smoothing: f64) -> Self {
        self.smoothing = smoothing;
        self
    }

    /// 返回 frame 输入点。
    #[must_use]
    pub fn points(&self) -> &[Point] {
        &self.points
    }

    /// 生成闭合 smooth frame 路径。
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

fn push_cubics(path: &mut SmoothPath, cubics: &[CubicSegment]) {
    for cubic in cubics {
        path.cubic_to(cubic.ctrl1, cubic.ctrl2, cubic.to);
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
