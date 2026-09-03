use std::f64::consts::PI;

use crate::output::path::CubicSegment;
use crate::types::{Point, Vector};
use crate::utils::EPSILON;

use super::axis::RectAxis;

#[derive(Debug, Clone, Copy)]
pub(super) struct RectCorner {
    pub(super) origin: Point,
    pub(super) incoming_axis: Vector,
    pub(super) incoming: RectAxis,
    pub(super) outgoing_axis: Vector,
    pub(super) outgoing: RectAxis,
}

impl RectCorner {
    // 组装矩形一个角的坐标系和两侧影响范围。
    pub(super) fn new(
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

    // 返回当前角在 incoming 边上的 cubic 起点。
    pub(super) fn start(self) -> Point {
        self.origin + self.incoming_axis * self.incoming.influence
    }

    // 返回当前角在 outgoing 边上的 cubic 终点。
    pub(super) fn end(self) -> Point {
        self.origin + self.outgoing_axis * self.outgoing.influence
    }
}

// 为矩形的一个角生成三段 Sketch-like cubic。
pub(super) fn rect_corner_cubics(corner: RectCorner, radius: f64) -> [CubicSegment; 3] {
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
