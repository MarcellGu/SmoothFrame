//! Sketch-like smooth corner / smooth frame 的 cubic Bezier 几何库。
//!
//! 低层类型 [`SmoothCorner`] 支持任意凸角；[`SmoothFrame`] 负责闭合凸多边形；
//! [`SmoothRect`] 只是矩形便捷封装。

#![warn(missing_docs)]

mod errors;
mod input;
mod output;
mod process;
mod types;
mod utils;

pub use errors::SmoothError;
pub use input::corner::SmoothCorner;
pub use input::frame::SmoothFrame;
pub use input::rect::SmoothRect;
pub use output::format::{PathFormatter, SvgPathFormat};
pub use output::path::{CubicSegment, PathCommand, SmoothPath};
pub use process::corner::SmoothCornerGeometry;
pub use types::geometry::{Point, Vector};
