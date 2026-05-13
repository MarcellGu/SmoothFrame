use smooth_frame::{PathCommand, PathFormatter, SmoothRect, SvgPathFormat};

// 验证 SVG path 输出不会带无意义的尾随零。
#[test]
fn svg_path_is_stable_without_trailing_zeroes() {
    let svg = SmoothRect::new(1000.0, 500.0)
        .with_radius(0.0)
        .to_path()
        .to_svg_path_with_precision(3);

    assert_eq!(svg, "M 0,0 L 1000,0 L 1000,500 L 0,500 Z");
    assert!(!svg.contains(".000"));
}

// 验证公开 SVG 精度会限制到 12 位，极大输入不会触发异常分配。
#[test]
fn svg_path_precision_is_capped() {
    let format = SvgPathFormat::new(usize::MAX);
    assert_eq!(format.precision(), SvgPathFormat::MAX_PRECISION);

    let path = SmoothRect::new(10.0 / 3.0, 5.0 / 3.0)
        .with_radius(0.0)
        .to_path();

    assert_eq!(
        path.to_svg_path_with_precision(usize::MAX),
        path.to_svg_path_with_precision(SvgPathFormat::MAX_PRECISION)
    );
}

struct FunctionCallFormat;

impl PathFormatter for FunctionCallFormat {
    type Output = Vec<String>;

    fn format(&self, commands: &[PathCommand]) -> Self::Output {
        commands
            .iter()
            .map(|command| match *command {
                PathCommand::MoveTo(point) => format!("move_to({}, {})", point.x, point.y),
                PathCommand::LineTo(point) => format!("line_to({}, {})", point.x, point.y),
                PathCommand::CubicTo { ctrl1, ctrl2, to } => format!(
                    "cubic_to({}, {}, {}, {}, {}, {})",
                    ctrl1.x, ctrl1.y, ctrl2.x, ctrl2.y, to.x, to.y
                ),
                PathCommand::Close => "close()".to_owned(),
            })
            .collect()
    }
}

// 验证使用方可以通过 formatter trait 扩展输出格式。
#[test]
fn custom_formatter_can_export_function_calls() {
    let calls = SmoothRect::new(10.0, 5.0)
        .with_radius(0.0)
        .to_path()
        .export_with(&FunctionCallFormat);

    assert_eq!(
        calls,
        [
            "move_to(0, 0)",
            "line_to(10, 0)",
            "line_to(10, 5)",
            "line_to(0, 5)",
            "close()"
        ]
    );
}
