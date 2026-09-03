use super::cli::Config;

// 将路径数据包成一个可预览的完整 SVG。
pub(crate) fn render_svg(config: &Config, path_data: &str) -> String {
    let offset = config.border_width / 2.0;
    let stroke_attrs = if config.border_width > 0.0 {
        format!(
            r##" stroke="#0F172A" stroke-opacity="0.12" stroke-width="{}""##,
            format_number(config.border_width)
        )
    } else {
        String::new()
    };

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {canvas_width} {canvas_height}" width="{canvas_width}" height="{canvas_height}">
  <defs>
    <linearGradient id="smooth-frame-fill" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#0EA5E9"/>
      <stop offset="48%" stop-color="#2DD4BF"/>
      <stop offset="100%" stop-color="#FDE68A"/>
    </linearGradient>
  </defs>
  <path d="{path_data}" transform="translate({offset} {offset})" fill="url(#smooth-frame-fill)"{stroke_attrs}/>
</svg>"##,
        canvas_width = format_number(config.width),
        canvas_height = format_number(config.height),
        offset = format_number(offset),
    )
}

// 以最多三位小数格式化 demo 输出数字。
pub(crate) fn format_number(value: f64) -> String {
    let formatted = format!("{value:.3}");
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_owned()
}
