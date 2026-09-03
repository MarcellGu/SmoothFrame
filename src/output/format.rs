use crate::output::path::PathCommand;
use crate::utils::{MAX_FORMAT_PRECISION, bounded_precision, format_point};

/// 将通用路径命令转换成具体后端输出的扩展契约。
///
/// 库内置 SVG path data formatter；Godot、Canvas、函数调用列表等格式可以在库外实现
/// 这个 trait，并通过 [`crate::SmoothPath::export_with`] 复用同一份路径中间表示。
pub trait PathFormatter {
    /// formatter 生成的目标输出类型。
    type Output;

    /// 将路径命令格式化为目标输出。
    fn format(&self, commands: &[PathCommand]) -> Self::Output;
}

/// SVG path data 输出格式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SvgPathFormat {
    precision: usize,
}

impl SvgPathFormat {
    /// 创建 SVG path data formatter。
    ///
    /// 小数位数最多保留 12 位，超过该上限时会自动 clamp。
    #[must_use]
    pub fn new(precision: usize) -> Self {
        Self {
            precision: bounded_precision(precision),
        }
    }

    /// SVG path data 输出支持的最大小数位数。
    pub const MAX_PRECISION: usize = MAX_FORMAT_PRECISION;

    /// 返回坐标小数位数。
    #[must_use]
    pub fn precision(&self) -> usize {
        self.precision
    }
}

impl Default for SvgPathFormat {
    fn default() -> Self {
        Self::new(6)
    }
}

impl PathFormatter for SvgPathFormat {
    type Output = String;

    fn format(&self, commands: &[PathCommand]) -> Self::Output {
        let mut parts = Vec::with_capacity(commands.len());
        for command in commands {
            match *command {
                PathCommand::MoveTo(point) => {
                    parts.push(format!("M {}", format_point(point, self.precision)));
                }
                PathCommand::LineTo(point) => {
                    parts.push(format!("L {}", format_point(point, self.precision)));
                }
                PathCommand::CubicTo { ctrl1, ctrl2, to } => {
                    parts.push(format!(
                        "C {} {} {}",
                        format_point(ctrl1, self.precision),
                        format_point(ctrl2, self.precision),
                        format_point(to, self.precision)
                    ));
                }
                PathCommand::Close => parts.push("Z".to_owned()),
            }
        }
        parts.join(" ")
    }
}
