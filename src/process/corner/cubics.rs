use crate::output::path::CubicSegment;
use crate::types::{Point, Vector};
use crate::utils::EPSILON;

use super::SmoothCornerGeometry;
use super::arc::{circle_point, circle_tangent};

// 根据解析后的 smooth corner 几何生成 cubic 段。
pub(super) fn build_cubics(geometry: SmoothCornerGeometry) -> Vec<CubicSegment> {
    if geometry.radius <= EPSILON {
        return Vec::new();
    }

    let basis = ArcBasis::from_geometry(geometry);
    let p1 = circle_point(
        basis.center,
        geometry.radius,
        basis.e0,
        basis.e1,
        geometry.alpha0,
    );
    let p2 = circle_point(
        basis.center,
        geometry.radius,
        basis.e0,
        basis.e1,
        geometry.angle - geometry.alpha1,
    );
    let handles = CornerHandles::from_geometry(geometry);

    vec![
        first_cubic(geometry, p1, handles.incoming),
        middle_cubic(geometry, basis, p1, p2, handles.arc),
        last_cubic(geometry, p2, handles.outgoing),
    ]
}

#[derive(Debug, Clone, Copy)]
struct ArcBasis {
    center: Point,
    e0: Vector,
    e1: Vector,
}

impl ArcBasis {
    // 根据 smooth corner 几何建立局部圆弧坐标系。
    fn from_geometry(geometry: SmoothCornerGeometry) -> Self {
        let center = geometry.origin
            + (geometry.incoming_axis + geometry.outgoing_axis)
                .normalized()
                .expect("输入层已保证 smooth corner 角度有效")
                * (geometry.radius / (geometry.angle / 2.0).sin());
        let incoming_tangent = geometry.origin + geometry.incoming_axis * geometry.base_tangent;
        Self {
            center,
            e0: (incoming_tangent - center) / geometry.radius,
            e1: -geometry.incoming_axis,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct CornerHandles {
    incoming: f64,
    outgoing: f64,
    arc: f64,
}

impl CornerHandles {
    // 根据 smooth corner 几何计算三段 cubic 的控制柄长度。
    fn from_geometry(geometry: SmoothCornerGeometry) -> Self {
        let tangent0 = geometry.base_tangent - geometry.radius * (geometry.alpha0 / 2.0).tan();
        let tangent1 = geometry.base_tangent - geometry.radius * (geometry.alpha1 / 2.0).tan();
        let arc = if geometry.middle_arc_angle <= EPSILON {
            0.0
        } else {
            (4.0 / 3.0) * (geometry.middle_arc_angle / 4.0).tan() * geometry.radius
        };

        Self {
            incoming: (geometry.incoming_influence - tangent0) / 3.0,
            outgoing: (geometry.outgoing_influence - tangent1) / 3.0,
            arc,
        }
    }
}

// 构造 incoming 侧的过渡 cubic。
fn first_cubic(geometry: SmoothCornerGeometry, p1: Point, incoming_handle: f64) -> CubicSegment {
    let tangent0 = geometry.base_tangent - geometry.radius * (geometry.alpha0 / 2.0).tan();
    CubicSegment {
        from: geometry.start,
        ctrl1: geometry.origin
            + geometry.incoming_axis * (geometry.incoming_influence - 2.0 * incoming_handle),
        ctrl2: geometry.origin + geometry.incoming_axis * tangent0,
        to: p1,
    }
}

// 构造中间圆弧 cubic。
fn middle_cubic(
    geometry: SmoothCornerGeometry,
    basis: ArcBasis,
    p1: Point,
    p2: Point,
    arc_handle: f64,
) -> CubicSegment {
    let arc_tangent0 = circle_tangent(basis.e0, basis.e1, geometry.alpha0);
    let arc_tangent1 = circle_tangent(basis.e0, basis.e1, geometry.angle - geometry.alpha1);
    CubicSegment {
        from: p1,
        ctrl1: p1 + arc_tangent0 * arc_handle,
        ctrl2: p2 - arc_tangent1 * arc_handle,
        to: p2,
    }
}

// 构造 outgoing 侧的过渡 cubic。
fn last_cubic(geometry: SmoothCornerGeometry, p2: Point, outgoing_handle: f64) -> CubicSegment {
    let tangent1 = geometry.base_tangent - geometry.radius * (geometry.alpha1 / 2.0).tan();
    CubicSegment {
        from: p2,
        ctrl1: geometry.origin + geometry.outgoing_axis * tangent1,
        ctrl2: geometry.origin + geometry.outgoing_axis * (tangent1 + outgoing_handle),
        to: geometry.end,
    }
}
