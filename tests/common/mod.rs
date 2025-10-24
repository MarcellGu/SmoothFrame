use squircle_frame::{PathCommand, Point, SmoothRect};
use std::env;
use std::path::{Path, PathBuf};

const SKETCHTOOL_TOLERANCE: f64 = 1.0e-5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SvgCommand {
    MoveTo(Point),
    LineTo(Point),
    CubicTo {
        ctrl1: Point,
        ctrl2: Point,
        to: Point,
    },
    Close,
}

#[derive(Debug)]
pub struct SketchtoolCase {
    name: String,
    width: f64,
    height: f64,
    radius: f64,
    smoothing: f64,
    path: String,
}

#[derive(Debug)]
pub struct CaseResult {
    suite: String,
    width: f64,
    height: f64,
    smoothing: f64,
    radius: f64,
    command_shape: String,
    max_diff: f64,
}

#[derive(Debug)]
struct SummaryRow {
    suite: String,
    size: String,
    smoothing: String,
    radii: Vec<f64>,
    command_shape: String,
    max_diff: f64,
}

impl From<PathCommand> for SvgCommand {
    fn from(command: PathCommand) -> Self {
        match command {
            PathCommand::MoveTo(point) => SvgCommand::MoveTo(point),
            PathCommand::LineTo(point) => SvgCommand::LineTo(point),
            PathCommand::CubicTo { ctrl1, ctrl2, to } => SvgCommand::CubicTo { ctrl1, ctrl2, to },
            PathCommand::Close => SvgCommand::Close,
        }
    }
}

pub fn find_sketchtool() -> Option<PathBuf> {
    if let Some(path) = env::var_os("SQUIRCLE_FRAME_SKETCHTOOL").map(PathBuf::from) {
        return path.exists().then_some(path);
    }

    let bundled = PathBuf::from("/Applications/Sketch.app/Contents/MacOS/sketchtool");
    if bundled.exists() {
        return Some(bundled);
    }

    env::var_os("PATH")?
        .to_string_lossy()
        .split(':')
        .map(|path| Path::new(path).join("sketchtool"))
        .find(|path| path.exists())
}

pub fn sketchtool_script() -> String {
    r#"
var api=require('sketch');

function attrValue(svg, tagName, attrName) {
  var tagStart = svg.indexOf('<' + tagName + ' ');
  if (tagStart < 0) return null;
  var attrStart = svg.indexOf(' ' + attrName + '="', tagStart);
  if (attrStart < 0) return null;
  attrStart += attrName.length + 3;
  var attrEnd = svg.indexOf('"', attrStart);
  return svg.slice(attrStart, attrEnd);
}

function extractPathData(svg) {
  var path = attrValue(svg, 'path', 'd');
  if (path) return path;

  var points = attrValue(svg, 'polygon', 'points');
  if (points) {
    var nums = points.trim().split(/[ ,]+/).filter(Boolean);
    var d = '';
    for (var i = 0; i < nums.length; i += 2) {
      d += (i === 0 ? 'M' : ' L') + nums[i] + ',' + nums[i + 1];
    }
    return d + ' Z';
  }

  return 'NO_PATH';
}

function emit(name, width, height, radius, smoothing) {
  var shape=new api.ShapePath({
    name:name,
    frame:new api.Rectangle(0,0,width,height)
  });
  shape.style.fills=[{color:'#000000FF'}];
  shape.style.borders=[];
  shape.style.corners={
    style:api.Style.CornerStyle.Smooth,
    radii:[radius,radius,radius,radius],
    smoothing:smoothing
  };
  page.layers.push(shape);
  var svg=String(api.export(shape,{formats:['svg'], output:null}));
  var d=extractPathData(svg);
  console.log(['SFCASE', name, width, height, radius, smoothing, d].join('\t'));
  shape.remove();
}

var doc=new api.Document();
var page=doc.selectedPage;

for (var radius=0; radius<=500; radius++) {
  emit('square_r_'+radius, 1000, 1000, radius, 0.6);
}

var smoothings=[0,0.3,0.6,0.8,1.0];
var smoothingRadii=[0,1,100,250,400,500];
for (var si=0; si<smoothings.length; si++) {
  for (var ri=0; ri<smoothingRadii.length; ri++) {
    emit(
      'smoothing_'+smoothings[si]+'_r_'+smoothingRadii[ri],
      1000,
      1000,
      smoothingRadii[ri],
      smoothings[si]
    );
  }
}

var rects=[
  [1000,500,250],
  [500,1000,250],
  [1200,300,150],
  [300,1200,150],
  [1024,768,384]
];
for (var di=0; di<rects.length; di++) {
  var item=rects[di];
  for (var rr=0; rr<=item[2]; rr++) {
    emit('rect_'+item[0]+'x'+item[1]+'_r_'+rr, item[0], item[1], rr, 0.6);
  }
}

try { doc.close(); } catch(e) {}
"#
    .to_owned()
}

pub fn parse_sketchtool_cases(stdout: &str) -> Vec<SketchtoolCase> {
    stdout
        .lines()
        .filter_map(|line| {
            let line = clean_sketch_console_line(line);
            let line = line.get(line.find("SFCASE")?..)?;
            let line = line.replace("\\t", "\t");
            let mut parts = line.splitn(7, '\t');
            if parts.next()? != "SFCASE" {
                return None;
            }

            Some(SketchtoolCase {
                name: parts.next()?.to_owned(),
                width: parts.next()?.parse().expect("width 解析失败"),
                height: parts.next()?.parse().expect("height 解析失败"),
                radius: parts.next()?.parse().expect("radius 解析失败"),
                smoothing: parts.next()?.parse().expect("smoothing 解析失败"),
                path: parts.next()?.to_owned(),
            })
        })
        .collect()
}

pub fn assert_case_matches(case: SketchtoolCase) -> CaseResult {
    let sketch_commands = parse_svg_path(&case.path);
    let ours = SmoothRect::new(case.width, case.height)
        .with_radius(case.radius)
        .with_smoothing(case.smoothing)
        .to_path();
    let our_commands = ours
        .commands()
        .iter()
        .copied()
        .map(SvgCommand::from)
        .collect::<Vec<_>>();

    assert_eq!(
        sketch_commands.len(),
        our_commands.len(),
        "case={} width={} height={} radius={} smoothing={} sketch={:?} ours={:?}",
        case.name,
        case.width,
        case.height,
        case.radius,
        case.smoothing,
        sketch_commands,
        our_commands
    );

    let mut max_diff: f64 = 0.0;
    for (index, (actual, expected)) in sketch_commands.iter().zip(our_commands.iter()).enumerate() {
        max_diff = max_diff.max(assert_svg_command_close(
            *actual, *expected, &case.name, index,
        ));
    }

    CaseResult {
        suite: case_suite(&case.name),
        width: case.width,
        height: case.height,
        smoothing: case.smoothing,
        radius: case.radius,
        command_shape: command_shape(&sketch_commands),
        max_diff,
    }
}

pub fn print_alignment_table(results: &[CaseResult]) {
    let mut rows: Vec<SummaryRow> = Vec::new();

    for result in results {
        let size = format_number(result.width, 0) + "x" + &format_number(result.height, 0);
        let smoothing = format_number(result.smoothing, 3);
        if let Some(row) = rows.iter_mut().find(|row| {
            row.suite == result.suite
                && row.size == size
                && row.smoothing == smoothing
                && row.command_shape == result.command_shape
        }) {
            row.radii.push(result.radius);
            row.max_diff = row.max_diff.max(result.max_diff);
        } else {
            rows.push(SummaryRow {
                suite: result.suite.clone(),
                size,
                smoothing,
                radii: vec![result.radius],
                command_shape: result.command_shape.clone(),
                max_diff: result.max_diff,
            });
        }
    }

    rows.sort_by(|a, b| {
        a.suite
            .cmp(&b.suite)
            .then(a.size.cmp(&b.size))
            .then(a.smoothing.cmp(&b.smoothing))
            .then(
                a.radii
                    .first()
                    .partial_cmp(&b.radii.first())
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
            .then(a.command_shape.cmp(&b.command_shape))
    });

    println!();
    println!("SketchTool 对齐结果表");
    println!("| 用例组 | 尺寸 | smoothing | radius | 命令结构 | 用例数 | 最大误差 | 结果 |");
    println!("|---|---:|---:|---:|---:|---:|---:|---|");
    for row in rows {
        println!(
            "| {} | {} | {} | {} | {} | {} | {:.9} | 通过 |",
            row.suite,
            row.size,
            row.smoothing,
            format_radii(&row.radii),
            row.command_shape,
            row.radii.len(),
            row.max_diff
        );
    }
}

fn clean_sketch_console_line(line: &str) -> &str {
    let line = line.trim();
    if line.len() >= 2 && line.starts_with('\'') && line.ends_with('\'') {
        &line[1..line.len() - 1]
    } else {
        line
    }
}

fn case_suite(name: &str) -> String {
    if name.starts_with("square_r_") {
        "square-radius-sweep".to_owned()
    } else if name.starts_with("smoothing_") {
        "smoothing-matrix".to_owned()
    } else if name.starts_with("rect_") {
        "rect-ratio-sweep".to_owned()
    } else {
        "other".to_owned()
    }
}

fn command_shape(commands: &[SvgCommand]) -> String {
    let mut moves = 0;
    let mut lines = 0;
    let mut cubics = 0;
    let mut closes = 0;
    for command in commands {
        match command {
            SvgCommand::MoveTo(_) => moves += 1,
            SvgCommand::LineTo(_) => lines += 1,
            SvgCommand::CubicTo { .. } => cubics += 1,
            SvgCommand::Close => closes += 1,
        }
    }
    format!("M{moves} L{lines} C{cubics} Z{closes}")
}

fn format_radii(radii: &[f64]) -> String {
    let mut radii = radii.to_vec();
    radii.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    radii.dedup_by(|a, b| (*a - *b).abs() <= f64::EPSILON);

    let mut ranges = Vec::new();
    let mut index = 0;
    while index < radii.len() {
        let start = radii[index] as i64;
        let mut end = start;
        index += 1;
        while index < radii.len() && radii[index] as i64 == end + 1 {
            end += 1;
            index += 1;
        }

        if start == end {
            ranges.push(start.to_string());
        } else {
            ranges.push(format!("{start}-{end}"));
        }
    }

    ranges.join(",")
}

fn parse_svg_path(path: &str) -> Vec<SvgCommand> {
    let tokens = tokenize_svg_path(path);
    let mut cursor = 0;
    let mut current_command = None;
    let mut commands = Vec::new();

    while cursor < tokens.len() {
        if let Token::Command(command) = tokens[cursor] {
            current_command = Some(command);
            cursor += 1;
        }

        match current_command.expect("SVG path 缺少命令") {
            'M' => {
                let point = read_point(&tokens, &mut cursor);
                commands.push(SvgCommand::MoveTo(point));
                current_command = Some('L');
            }
            'L' => {
                let point = read_point(&tokens, &mut cursor);
                commands.push(SvgCommand::LineTo(point));
            }
            'C' => {
                let ctrl1 = read_point(&tokens, &mut cursor);
                let ctrl2 = read_point(&tokens, &mut cursor);
                let to = read_point(&tokens, &mut cursor);
                commands.push(SvgCommand::CubicTo { ctrl1, ctrl2, to });
            }
            'Z' => {
                commands.push(SvgCommand::Close);
                current_command = None;
            }
            command => panic!("暂不支持的 SVG 命令：{command}"),
        }
    }

    commands
}

#[derive(Debug, Clone, Copy)]
enum Token {
    Command(char),
    Number(f64),
}

fn tokenize_svg_path(path: &str) -> Vec<Token> {
    let chars = path.chars().collect::<Vec<_>>();
    let mut cursor = 0;
    let mut tokens = Vec::new();

    while cursor < chars.len() {
        let ch = chars[cursor];
        if ch.is_ascii_whitespace() || ch == ',' {
            cursor += 1;
            continue;
        }
        if matches!(ch, 'M' | 'L' | 'C' | 'Z') {
            tokens.push(Token::Command(ch));
            cursor += 1;
            continue;
        }

        let start = cursor;
        cursor += 1;
        while cursor < chars.len() {
            let ch = chars[cursor];
            let prev = chars[cursor - 1];
            if ch.is_ascii_digit() || ch == '.' || ch == 'e' || ch == 'E' {
                cursor += 1;
            } else if (ch == '-' || ch == '+') && (prev == 'e' || prev == 'E') {
                cursor += 1;
            } else {
                break;
            }
        }

        let number = chars[start..cursor]
            .iter()
            .collect::<String>()
            .parse::<f64>()
            .expect("SVG path 数字解析失败");
        tokens.push(Token::Number(number));
    }

    tokens
}

fn read_point(tokens: &[Token], cursor: &mut usize) -> Point {
    Point::new(read_number(tokens, cursor), read_number(tokens, cursor))
}

fn read_number(tokens: &[Token], cursor: &mut usize) -> f64 {
    let number = match tokens.get(*cursor) {
        Some(Token::Number(number)) => *number,
        other => panic!("期望 SVG 数字，实际为：{other:?}"),
    };
    *cursor += 1;
    number
}

fn assert_svg_command_close(
    actual: SvgCommand,
    expected: SvgCommand,
    case_name: &str,
    index: usize,
) -> f64 {
    match (actual, expected) {
        (SvgCommand::MoveTo(actual), SvgCommand::MoveTo(expected))
        | (SvgCommand::LineTo(actual), SvgCommand::LineTo(expected)) => {
            assert_point_close(actual, expected, case_name, index)
        }
        (
            SvgCommand::CubicTo {
                ctrl1: actual_ctrl1,
                ctrl2: actual_ctrl2,
                to: actual_to,
            },
            SvgCommand::CubicTo {
                ctrl1: expected_ctrl1,
                ctrl2: expected_ctrl2,
                to: expected_to,
            },
        ) => {
            let ctrl1_diff = assert_point_close(actual_ctrl1, expected_ctrl1, case_name, index);
            let ctrl2_diff = assert_point_close(actual_ctrl2, expected_ctrl2, case_name, index);
            let to_diff = assert_point_close(actual_to, expected_to, case_name, index);
            ctrl1_diff.max(ctrl2_diff).max(to_diff)
        }
        (SvgCommand::Close, SvgCommand::Close) => 0.0,
        _ => panic!(
            "case={case_name} 第 {index} 条 SVG 命令类型不匹配：actual={actual:?}, expected={expected:?}"
        ),
    }
}

fn assert_point_close(actual: Point, expected: Point, case_name: &str, index: usize) -> f64 {
    let x_diff = assert_number_close(actual.x, expected.x, case_name, index);
    let y_diff = assert_number_close(actual.y, expected.y, case_name, index);
    x_diff.max(y_diff)
}

fn assert_number_close(actual: f64, expected: f64, case_name: &str, index: usize) -> f64 {
    let diff = (actual - expected).abs();
    assert!(
        diff <= SKETCHTOOL_TOLERANCE,
        "case={case_name} 第 {index} 条命令数值不匹配：actual={actual}, expected={expected}, diff={}",
        diff
    );
    diff
}

fn format_number(value: f64, precision: usize) -> String {
    let mut text = format!("{value:.precision$}");
    if text.contains('.') {
        while text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
    }
    text
}
