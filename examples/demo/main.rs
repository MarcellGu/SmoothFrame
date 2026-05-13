mod cli;
mod svg;

use std::env;
use std::fs;
use std::process;

use smooth_frame::SmoothRect;

use cli::{drawable_dimension, effective_radius, parse_args};
use svg::{format_number, render_svg};

// 解析命令行参数并输出路径数据或完整 SVG。
fn main() {
    let config = match parse_args(env::args().skip(1)) {
        Ok(Some(config)) => config,
        Ok(None) => return,
        Err(message) => {
            eprintln!("参数错误：{message}");
            eprintln!("运行 `cargo run --example demo -- --help` 查看用法。");
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
