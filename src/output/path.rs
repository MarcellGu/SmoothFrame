use crate::types::Point;

use super::format::{PathFormatter, SvgPathFormat};

/// 一段 cubic Bezier，包含起点，便于直接映射到底层渲染 API。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CubicSegment {
    /// cubic 起点。
    pub from: Point,
    /// 第一个控制点。
    pub ctrl1: Point,
    /// 第二个控制点。
    pub ctrl2: Point,
    /// cubic 终点。
    pub to: Point,
}

/// 可直接映射到 SVG Canvas Skia 等 API 的路径命令。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathCommand {
    /// 移动当前点。
    MoveTo(Point),
    /// 从当前点绘制直线。
    LineTo(Point),
    /// 从当前点绘制 cubic Bezier。
    CubicTo {
        /// 第一个控制点。
        ctrl1: Point,
        /// 第二个控制点。
        ctrl2: Point,
        /// cubic 终点。
        to: Point,
    },
    /// 闭合当前子路径。
    Close,
}

/// 平滑路径。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SmoothPath {
    commands: Vec<PathCommand>,
}

impl SmoothPath {
    /// 创建空路径。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 返回底层路径命令。
    #[must_use]
    pub fn commands(&self) -> &[PathCommand] {
        &self.commands
    }

    /// 提取路径中的所有 cubic 段。
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

    /// 追加 `MoveTo` 命令。
    pub fn move_to(&mut self, point: Point) {
        self.commands.push(PathCommand::MoveTo(point));
    }

    /// 追加 `LineTo` 命令。
    pub fn line_to(&mut self, point: Point) {
        self.commands.push(PathCommand::LineTo(point));
    }

    /// 追加 `CubicTo` 命令。
    pub fn cubic_to(&mut self, ctrl1: Point, ctrl2: Point, to: Point) {
        self.commands
            .push(PathCommand::CubicTo { ctrl1, ctrl2, to });
    }

    /// 追加闭合路径命令。
    pub fn close(&mut self) {
        self.commands.push(PathCommand::Close);
    }

    /// 使用指定 formatter 导出路径。
    ///
    /// 这是输出层的主要扩展点：调用方可以实现 [`PathFormatter`]，把同一组
    /// [`PathCommand`] 转换成 SVG、Godot、Canvas 或自定义函数调用格式。
    pub fn export_with<F>(&self, formatter: &F) -> F::Output
    where
        F: PathFormatter + ?Sized,
    {
        formatter.format(self.commands())
    }

    /// 以默认 6 位小数输出 SVG path data。
    #[must_use]
    pub fn to_svg_path(&self) -> String {
        self.export_with(&SvgPathFormat::default())
    }

    /// 按指定小数位数输出 SVG path data。
    #[must_use]
    pub fn to_svg_path_with_precision(&self, precision: usize) -> String {
        self.export_with(&SvgPathFormat::new(precision))
    }
}
