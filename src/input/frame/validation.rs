use crate::errors::SmoothError;
use crate::types::Point;
use crate::utils::EPSILON;

// 校验 frame 输入是否满足当前算法支持的闭合凸多边形条件。
pub(super) fn validate_frame(points: &[Point]) -> Result<(), SmoothError> {
    if points.len() < 3 {
        return Err(SmoothError::TooFewPoints);
    }
    if points.iter().any(|point| !point.is_finite()) {
        return Err(SmoothError::NonFiniteInput);
    }

    validate_area_and_edges(points)?;
    if has_self_intersection(points) {
        return Err(SmoothError::SelfIntersectingFrame);
    }
    validate_convex_turns(points)
}

// 校验 polygon 面积和边长是否都非退化。
fn validate_area_and_edges(points: &[Point]) -> Result<(), SmoothError> {
    let mut area = 0.0;
    for index in 0..points.len() {
        let a = points[index];
        let b = points[(index + 1) % points.len()];
        if (b - a).length() <= EPSILON {
            return Err(SmoothError::DegenerateFrame);
        }
        area += a.x * b.y - b.x * a.y;
    }
    if area.abs() <= EPSILON {
        return Err(SmoothError::DegenerateFrame);
    }
    Ok(())
}

// 校验每个转角方向一致且不存在共线退化角。
fn validate_convex_turns(points: &[Point]) -> Result<(), SmoothError> {
    let mut turn_sign = 0.0_f64;
    for index in 0..points.len() {
        let prev = points[(index + points.len() - 1) % points.len()];
        let origin = points[index];
        let next = points[(index + 1) % points.len()];
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

// 检查非相邻边之间是否存在交点。
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

// 判断两条边在闭合 polygon 中是否共享端点。
fn are_adjacent_edges(first: usize, second: usize, count: usize) -> bool {
    first == second || (first + 1) % count == second || (second + 1) % count == first
}

// 使用方向测试判断两条线段是否相交。
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

// 计算点 c 相对有向边 ab 的方向值。
fn orientation(a: Point, b: Point, c: Point) -> f64 {
    (b - a).cross(c - a)
}

// 判断点是否落在线段的包围盒范围内。
fn point_on_segment(point: Point, start: Point, end: Point) -> bool {
    point.x >= start.x.min(end.x) - EPSILON
        && point.x <= start.x.max(end.x) + EPSILON
        && point.y >= start.y.min(end.y) - EPSILON
        && point.y <= start.y.max(end.y) + EPSILON
}
