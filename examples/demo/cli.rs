use std::path::PathBuf;

#[derive(Debug, Clone)]
pub(crate) struct Config {
    pub(crate) width: f64,
    pub(crate) height: f64,
    pub(crate) radius: Option<f64>,
    pub(crate) smoothing: f64,
    pub(crate) precision: usize,
    pub(crate) border_width: f64,
    pub(crate) svg: bool,
    pub(crate) output: Option<PathBuf>,
}

impl Default for Config {
    // 提供 demo 的默认尺寸、半径和输出选项。
    fn default() -> Self {
        Self {
            width: 1000.0,
            height: 1000.0,
            radius: Some(225.0),
            smoothing: 0.6,
            precision: 3,
            border_width: 0.0,
            svg: false,
            output: None,
        }
    }
}

// 将命令行参数解析成 demo 配置。
pub(crate) fn parse_args(args: impl Iterator<Item = String>) -> Result<Option<Config>, String> {
    let mut config = Config::default();
    let mut args = args.peekable();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                return Ok(None);
            }
            "--svg" => config.svg = true,
            "-o" | "--output" => {
                config.output = Some(parse_path_arg("--output", args.next())?);
            }
            "--width" => config.width = parse_f64_arg("--width", args.next())?,
            "--height" => config.height = parse_f64_arg("--height", args.next())?,
            "--radius" => config.radius = Some(parse_f64_arg("--radius", args.next())?),
            "--smoothing" => config.smoothing = parse_f64_arg("--smoothing", args.next())?,
            "--precision" => config.precision = parse_usize_arg("--precision", args.next())?,
            "--border" | "--border-width" => {
                config.border_width = parse_f64_arg("--border-width", args.next())?;
            }
            _ if arg.starts_with("--width=") => {
                config.width = parse_f64_value("--width", value_after_equals(&arg))?;
            }
            _ if arg.starts_with("--height=") => {
                config.height = parse_f64_value("--height", value_after_equals(&arg))?;
            }
            _ if arg.starts_with("--radius=") => {
                config.radius = Some(parse_f64_value("--radius", value_after_equals(&arg))?);
            }
            _ if arg.starts_with("--smoothing=") => {
                config.smoothing = parse_f64_value("--smoothing", value_after_equals(&arg))?;
            }
            _ if arg.starts_with("--precision=") => {
                config.precision = parse_usize_value("--precision", value_after_equals(&arg))?;
            }
            _ if arg.starts_with("--border=") => {
                config.border_width = parse_f64_value("--border", value_after_equals(&arg))?;
            }
            _ if arg.starts_with("--border-width=") => {
                config.border_width = parse_f64_value("--border-width", value_after_equals(&arg))?;
            }
            _ if arg.starts_with("--output=") => {
                config.output = Some(PathBuf::from(value_after_equals(&arg)));
            }
            _ => return Err(format!("未知参数 `{arg}`")),
        }
    }

    validate_config(&config)?;
    Ok(Some(config))
}

// 校验 demo 输入是否处于可渲染范围。
fn validate_config(config: &Config) -> Result<(), String> {
    if !config.width.is_finite() || config.width <= 0.0 {
        return Err("--width 必须是大于 0 的有限数字".to_owned());
    }
    if !config.height.is_finite() || config.height <= 0.0 {
        return Err("--height 必须是大于 0 的有限数字".to_owned());
    }
    if let Some(radius) = config.radius {
        if !radius.is_finite() || radius < 0.0 {
            return Err("--radius 必须是大于或等于 0 的有限数字".to_owned());
        }
    }
    if !config.smoothing.is_finite() || !(0.0..=1.0).contains(&config.smoothing) {
        return Err("--smoothing 必须在 0 到 1 之间".to_owned());
    }
    if !config.border_width.is_finite() || config.border_width < 0.0 {
        return Err("--border-width 必须是大于或等于 0 的有限数字".to_owned());
    }
    if config.border_width >= config.width.min(config.height) {
        return Err("--border-width 必须小于 SVG 宽度和高度".to_owned());
    }
    if config.precision > 12 {
        return Err("--precision 不能大于 12".to_owned());
    }
    Ok(())
}

// 返回最终用于几何计算的圆角半径。
pub(crate) fn effective_radius(config: &Config) -> f64 {
    config.radius.unwrap_or(250.0)
}

// 根据边框宽度计算实际路径尺寸。
pub(crate) fn drawable_dimension(svg_dimension: f64, border_width: f64) -> f64 {
    svg_dimension - border_width
}

// 解析带参数名的浮点数参数。
fn parse_f64_arg(name: &str, value: Option<String>) -> Result<f64, String> {
    parse_f64_value(
        name,
        value
            .as_deref()
            .ok_or_else(|| format!("缺少 {name} 的值"))?,
    )
}

// 解析带参数名的无符号整数参数。
fn parse_usize_arg(name: &str, value: Option<String>) -> Result<usize, String> {
    parse_usize_value(
        name,
        value
            .as_deref()
            .ok_or_else(|| format!("缺少 {name} 的值"))?,
    )
}

// 解析文件路径参数。
fn parse_path_arg(name: &str, value: Option<String>) -> Result<PathBuf, String> {
    value
        .map(PathBuf::from)
        .ok_or_else(|| format!("缺少 {name} 的值"))
}

// 将字符串值解析为浮点数。
fn parse_f64_value(name: &str, value: &str) -> Result<f64, String> {
    value
        .parse()
        .map_err(|_| format!("{name} 的值 `{value}` 不是有效数字"))
}

// 将字符串值解析为无符号整数。
fn parse_usize_value(name: &str, value: &str) -> Result<usize, String> {
    value
        .parse()
        .map_err(|_| format!("{name} 的值 `{value}` 不是有效整数"))
}

// 取出 `--key=value` 形式参数中的值。
fn value_after_equals(arg: &str) -> &str {
    arg.split_once('=').map_or("", |(_, value)| value)
}

// 打印 demo 的命令行帮助信息。
fn print_help() {
    println!(
        r#"demo

生成 Sketch-like smooth corner 矩形的 SVG path 或完整 SVG。

用法：
  cargo run --example demo
  cargo run --example demo -- --width 1000 --height 1000 --radius 250 --smoothing 0.6
  cargo run --example demo -- --svg > smooth.svg
  cargo run --example demo -- --output smooth.svg

参数：
  --width <数字>       矩形宽度，默认 1000
  --height <数字>      矩形高度，默认 1000
  --radius <数字>      核心圆半径，默认 250
  --smoothing <数字>   平滑系数，范围 0..1，默认 0.6
  --precision <整数>   SVG path 小数位数，默认 3
  --border <数字>      SVG 边框宽度，默认 0
  --svg               输出完整 SVG，而不是只输出 path
  -o, --output <路径>  生成完整 SVG 文件
  -h, --help          显示帮助
"#
    );
}
