use crate::utils::EPSILON;

use super::{SmoothCornerGeometry, SmoothCornerRequest};

// 解析 smooth corner 的所有几何参数。
pub(super) fn resolve_geometry(request: SmoothCornerRequest) -> SmoothCornerGeometry {
    let max_limit = request.incoming_limit.min(request.outgoing_limit);
    let max_radius = max_limit * (request.angle / 2.0).tan();
    let radius = request.radius.min(max_radius);

    if radius <= EPSILON {
        return zero_radius_geometry(request);
    }

    nonzero_radius_geometry(request, radius)
}

// 生成半径被压缩为 0 时的几何结果。
fn zero_radius_geometry(request: SmoothCornerRequest) -> SmoothCornerGeometry {
    SmoothCornerGeometry {
        origin: request.origin,
        incoming_axis: request.incoming_axis,
        outgoing_axis: request.outgoing_axis,
        radius: 0.0,
        smoothing: request.smoothing,
        incoming_limit: request.incoming_limit,
        outgoing_limit: request.outgoing_limit,
        angle: request.angle,
        base_tangent: 0.0,
        incoming_influence: 0.0,
        outgoing_influence: 0.0,
        alpha0: 0.0,
        alpha1: 0.0,
        middle_arc_angle: request.angle,
        start: request.origin,
        end: request.origin,
    }
}

// 生成半径非零时的完整 smooth corner 几何结果。
fn nonzero_radius_geometry(request: SmoothCornerRequest, radius: f64) -> SmoothCornerGeometry {
    let base_tangent = radius / (request.angle / 2.0).tan();
    let raw_influence = (1.0 + request.smoothing) * base_tangent;
    let incoming_influence = raw_influence.min(request.incoming_limit);
    let outgoing_influence = raw_influence.min(request.outgoing_limit);

    let alpha0 = (incoming_influence / base_tangent - 1.0) * request.angle / 2.0;
    let alpha1 = (outgoing_influence / base_tangent - 1.0) * request.angle / 2.0;
    let middle_arc_angle = (request.angle - alpha0 - alpha1).max(0.0);

    SmoothCornerGeometry {
        origin: request.origin,
        incoming_axis: request.incoming_axis,
        outgoing_axis: request.outgoing_axis,
        radius,
        smoothing: request.smoothing,
        incoming_limit: request.incoming_limit,
        outgoing_limit: request.outgoing_limit,
        angle: request.angle,
        base_tangent,
        incoming_influence,
        outgoing_influence,
        alpha0,
        alpha1,
        middle_arc_angle,
        start: request.origin + request.incoming_axis * incoming_influence,
        end: request.origin + request.outgoing_axis * outgoing_influence,
    }
}
