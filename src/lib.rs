//! Sketch-like smooth corner / smooth frame 的 cubic Bezier 几何库。
//!
//! 低层类型 [`SmoothCorner`] 支持任意凸角；[`SmoothFrame`] 负责闭合凸多边形；
//! [`SmoothRect`] 只是矩形便捷封装。

#![warn(missing_docs)]

mod corner;
mod error;
mod frame;
mod geometry;
mod math;
mod path;
mod rect;

pub use corner::{SmoothCorner, SmoothCornerGeometry};
pub use error::SmoothError;
pub use frame::SmoothFrame;
pub use geometry::{Point, Vector};
pub use path::{CubicSegment, PathCommand, SmoothPath};
pub use rect::SmoothRect;
