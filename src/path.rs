use crate::geometry::Point;

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

    /// 以默认 6 位小数输出 SVG path data。
    #[must_use]
    pub fn to_svg_path(&self) -> String {
        self.to_svg_path_with_precision(6)
    }

    /// 按指定小数位数输出 SVG path data。
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
