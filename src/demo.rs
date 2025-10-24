use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

use squircle_frame::SmoothRect;

#[derive(Debug, Clone)]
struct Config {
    width: f64,
    height: f64,
    radius: Option<f64>,
    smoothing: f64,
    precision: usize,
    border_width: f64,
    svg: bool,
    output: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            width: 1000.0,
            height: 1000.0,
            radius: Some(225.0),
            smoothing: 0.6,
            precision: 3,
            border_width: 2.0,
            svg: false,
            output: None,
        }
    }
}

fn main() {
    let config = match parse_args(env::args().skip(1)) {
        Ok(Some(config)) => config,
        Ok(None) => return,
        Err(message) => {
            eprintln!("参数错误：{message}");
            eprintln!("运行 `cargo run --bin demo -- --help` 查看用法。");
            process::exit(2);
        }
    };

    let drawable_width = drawable_dimension(config.width, config.border_width);
    let drawable_height = drawable_dimension(config.height, config.border_width);
    let radius = effective_radius(&config);
    let path = SmoothRect::new(drawable_width, drawable_height)
        .with_radius(radius)
        .with_smoothing(config.smoothing)
        .to_path();
    let path_data = path.to_svg_path_with_precision(config.precision);

    if let Some(output) = config.output.as_ref() {
        let svg = render_svg(&config, &path_data);
        if let Err(error) = fs::write(output, svg) {
            eprintln!("写入 SVG 文件失败：{}：{error}", output.display());
            process::exit(1);
        }
        println!("已生成 SVG 文件：{}", output.display());
    } else if config.svg {
        println!("{}", render_svg(&config, &path_data));
    } else {
        println!("SVG 宽度：{}", format_number(config.width));
        println!("SVG 高度：{}", format_number(config.height));
        println!("路径宽度：{}", format_number(drawable_width));
        println!("路径高度：{}", format_number(drawable_height));
        println!("半径：{}", format_number(radius));
        println!("平滑：{}", format_number(config.smoothing));
        println!("边框：{}", format_number(config.border_width));
        println!("命令数：{}", path.commands().len());
        println!("SVG path：{path_data}");
    }
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Option<Config>, String> {
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

fn effective_radius(config: &Config) -> f64 {
    config.radius.unwrap_or(250.0)
}

fn drawable_dimension(svg_dimension: f64, border_width: f64) -> f64 {
    svg_dimension - border_width
}

fn parse_f64_arg(name: &str, value: Option<String>) -> Result<f64, String> {
    parse_f64_value(
        name,
        value
            .as_deref()
            .ok_or_else(|| format!("缺少 {name} 的值"))?,
    )
}

fn parse_usize_arg(name: &str, value: Option<String>) -> Result<usize, String> {
    parse_usize_value(
        name,
        value
            .as_deref()
            .ok_or_else(|| format!("缺少 {name} 的值"))?,
    )
}

fn parse_path_arg(name: &str, value: Option<String>) -> Result<PathBuf, String> {
    value
        .map(PathBuf::from)
        .ok_or_else(|| format!("缺少 {name} 的值"))
}

fn parse_f64_value(name: &str, value: &str) -> Result<f64, String> {
    value
        .parse()
        .map_err(|_| format!("{name} 的值 `{value}` 不是有效数字"))
}

fn parse_usize_value(name: &str, value: &str) -> Result<usize, String> {
    value
        .parse()
        .map_err(|_| format!("{name} 的值 `{value}` 不是有效整数"))
}

fn value_after_equals(arg: &str) -> &str {
    arg.split_once('=').map_or("", |(_, value)| value)
}

fn render_svg(config: &Config, path_data: &str) -> String {
    let offset = config.border_width / 2.0;

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {canvas_width} {canvas_height}" width="{canvas_width}" height="{canvas_height}">
  <path d="{path_data}" transform="translate({offset} {offset})" fill="#6EE7B7" stroke="#0F172A" stroke-width="{border_width}"/>
</svg>"##,
        canvas_width = format_number(config.width),
        canvas_height = format_number(config.height),
        offset = format_number(offset),
        border_width = format_number(config.border_width),
    )
}

fn format_number(value: f64) -> String {
    let formatted = format!("{value:.3}");
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_owned()
}

fn print_help() {
    println!(
        r#"demo

生成 Sketch-like smooth corner 矩形的 SVG path 或完整 SVG。

用法：
  cargo run --bin demo
  cargo run --bin demo -- --width 1000 --height 1000 --radius 250 --smoothing 0.6
  cargo run --bin demo -- --svg > squircle.svg
  cargo run --bin demo -- --output squircle.svg

参数：
  --width <数字>       矩形宽度，默认 1000
  --height <数字>      矩形高度，默认 1000
  --radius <数字>      核心圆半径，默认 250
  --smoothing <数字>   平滑系数，范围 0..1，默认 0.6
  --precision <整数>   SVG path 小数位数，默认 3
  --border <数字>      SVG 边框宽度，默认 2
  --svg               输出完整 SVG，而不是只输出 path
  -o, --output <路径>  生成完整 SVG 文件
  -h, --help          显示帮助
"#
    );
}
