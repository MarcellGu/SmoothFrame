![SmoothFrame](demo.svg)

# SmoothFrame

SmoothFrame 是一个零依赖的 Rust 几何库，用来生成接近 Sketch「Smooth Corners」效果的 cubic Bezier **路径**。

它只负责算路径，不绑定任何渲染后端。你可以把输出命令直接接到 SVG、Canvas、Skia、Godot 或自己的绘图管线里。

## 安装

```toml
[dependencies]
smooth-frame = "0.2"
```

项目使用 Rust 2024 edition，最低 Rust 版本为 `1.85`。

## 快速开始

最常用的入口是 `SmoothRect`：

```rust
use smooth_frame::SmoothRect;

fn main() {
    let path = SmoothRect::new(240.0, 120.0)
        .with_radius(32.0)
        .with_smoothing(0.6)
        .to_path();

    println!("{}", path.to_svg_path_with_precision(2));
}
```

`radius` 是核心圆半径，`smoothing` 是 Sketch 风格的平滑系数，会被限制在 `0..=1`。

生成出来的 `SmoothPath` 可以：

- 用 `to_svg_path()` 或 `to_svg_path_with_precision()` 输出 SVG path data。
- 用 `commands()` 读取 `MoveTo / LineTo / CubicTo / Close` 命令。
- 用 `cubics()` 提取所有 cubic Bezier 段，方便测试、采样或接入其他 API。
- 用 `export_with()` 接入自定义 `PathFormatter`，输出 Godot、Canvas 或函数调用等格式。

## 运行示例

仓库里带了一个 demo，可以直接生成路径或完整 SVG：

```bash
cargo run --example demo
cargo run --example demo -- --svg > smooth.svg
cargo run --example demo -- --output smooth.svg
```

也可以调整尺寸、半径、平滑系数和边框：

```bash
cargo run --example demo -- \
  --width 1000 \
  --height 600 \
  --radius 160 \
  --smoothing 0.6 \
  --border 4 \
  --output smooth.svg
```

查看所有参数：

```bash
cargo run --example demo -- --help
```

## API 一览

### `SmoothRect`

矩形便捷 API，也是最接近 SketchTool 导出结果的入口。

```rust
use smooth_frame::SmoothRect;

fn main() {
    let path = SmoothRect::new(1000.0, 500.0)
        .with_radius(120.0)
        .with_smoothing(0.6)
        .to_path();

    let svg_path = path.to_svg_path_with_precision(3);
}
```

普通区间下，一个矩形最多会生成 4 条边和 12 段 cubic。接近圆形或胶囊形时，SketchTool 会退化成普通圆角、圆或 capsule-like 结构；
`SmoothRect` 会跟随这种行为。

如果你需要每个角始终保留 3 段 cubic 的 primitive，请使用 `SmoothCorner` 或 `SmoothFrame`。

### `SmoothFrame`

用于闭合凸多边形。

```rust
use smooth_frame::{Point, SmoothError, SmoothFrame};

fn main() -> Result<(), SmoothError> {
    let path = SmoothFrame::closed([
        Point::new(0.0, 0.0),
        Point::new(220.0, 30.0),
        Point::new(180.0, 170.0),
        Point::new(20.0, 140.0),
    ])
        .with_radius(24.0)
        .with_smoothing(0.5)
        .to_path()?;

    println!("{}", path.to_svg_path());
    Ok(())
}
```

`SmoothFrame` 会检查输入点：

- 少于 3 个点会返回 `TooFewPoints`。
- 退化边、共线点或面积为 0 会返回 `DegenerateFrame`。
- 凹多边形会返回 `ConcaveFrame`。
- 自相交路径会返回 `SelfIntersectingFrame`。

### `SmoothCorner`

用于单个凸角。它是更底层的 primitive，适合你自己管理边、角和拼接顺序的场景。

```rust
use smooth_frame::{Point, SmoothCorner, SmoothError, Vector};

fn main() -> Result<(), SmoothError> {
    let cubics = SmoothCorner::new(
        Point::new(100.0, 0.0),
        Vector::new(-1.0, 0.0),
        Vector::new(0.0, 1.0),
    )
        .with_radius(24.0)
        .with_smoothing(0.6)
        .with_limits(80.0, 60.0)
        .to_cubics()?;

    println!("cubic 段数：{}", cubics.len());
    Ok(())
}
```

输入语义：

- `origin`：当前角点。
- `incoming_axis`：从角点指向上一条边的方向。
- `outgoing_axis`：从角点指向下一条边的方向。
- `with_radius()`：核心圆半径。
- `with_smoothing()`：Sketch-like smoothing，计算时会 clamp 到 `0..=1`。
- `with_limits()`：incoming / outgoing 两侧允许占用的最大长度。

## 接到渲染 API

输出层以 `SmoothPath` 作为稳定中间表示，SVG 只是内置 formatter。需要接到其他后端时，可以直接读取路径命令：

```rust
use smooth_frame::{PathCommand, SmoothRect};

fn main() {
    let path = SmoothRect::new(240.0, 120.0)
        .with_radius(32.0)
        .with_smoothing(0.6)
        .to_path();

    for command in path.commands() {
        match *command {
            PathCommand::MoveTo(p) => {
                // SVG: M x,y
                // Canvas: moveTo(x, y)
            }
            PathCommand::LineTo(p) => {
                // SVG: L x,y
                // Canvas: lineTo(x, y)
            }
            PathCommand::CubicTo { ctrl1, ctrl2, to } => {
                // SVG: C c1x,c1y c2x,c2y x,y
                // Canvas: bezierCurveTo(c1x, c1y, c2x, c2y, x, y)
            }
            PathCommand::Close => {
                // SVG: Z
                // Canvas: closePath()
            }
        }
    }
}
```

`CubicSegment` 里包含 `from / ctrl1 / ctrl2 / to`，如果你的后端更喜欢一段一段处理 Bezier，可以直接使用 `path.cubics()`。

也可以实现 `PathFormatter`，把输出格式作为独立扩展点：

```rust
use smooth_frame::{PathCommand, PathFormatter, SmoothRect};

struct FunctionCallFormat;

impl PathFormatter for FunctionCallFormat {
    type Output = Vec<String>;

    fn format(&self, commands: &[PathCommand]) -> Self::Output {
        commands
            .iter()
            .map(|command| match *command {
                PathCommand::MoveTo(p) => format!("move_to({}, {})", p.x, p.y),
                PathCommand::LineTo(p) => format!("line_to({}, {})", p.x, p.y),
                PathCommand::CubicTo { ctrl1, ctrl2, to } => format!(
                    "cubic_to({}, {}, {}, {}, {}, {})",
                    ctrl1.x, ctrl1.y, ctrl2.x, ctrl2.y, to.x, to.y
                ),
                PathCommand::Close => "close()".to_owned(),
            })
            .collect()
    }
}

fn main() {
    let calls = SmoothRect::new(240.0, 120.0)
        .with_radius(32.0)
        .with_smoothing(0.6)
        .to_path()
        .export_with(&FunctionCallFormat);
}
```

## 和 SketchTool 的关系

`SmoothRect` 的目标是对齐 SketchTool 对 smooth corner 矩形的实际 SVG 导出，而不是只实现一个近似公式。

测试里包含两类约束：

- 内置几何单元测试，覆盖常见矩形、最大半径、胶囊形、圆形退化、凸多边形、错误输入等情况。
- 如果本机安装了 SketchTool，集成测试会批量创建 Sketch smooth rect、导出 SVG、逐条比较 `M / L / C / Z` 控制点。

默认情况下，找不到 SketchTool 时会跳过集成测试。如果本机能找到 SketchTool，`cargo test` 会跑完整对齐矩阵，这一步可能比较慢。只想验证
SketchTool 对齐时可以单独运行：

```bash
cargo test --test sketchtool -- --nocapture
```

重复运行 SketchTool 集成测试前，建议先手动退出 Sketch。SketchTool 会调用本机 Sketch 进行导出，Sketch 已打开或上一次测试后仍停留在后台时，可能复用旧状态或占用导出环境，导致重复测试结果不稳定。

需要指定 SketchTool 路径或强制要求本机必须存在 SketchTool 时：

```bash
SMOOTH_FRAME_SKETCHTOOL=/Applications/Sketch.app/Contents/MacOS/sketchtool cargo test --test sketchtool
SMOOTH_FRAME_REQUIRE_SKETCHTOOL=1 cargo test --test sketchtool
```

## 开发

```bash
cargo test --test rect
cargo test --test corner
cargo test --test frame
cargo test --test path
cargo test --test sketchtool -- --nocapture
cargo run --example demo -- --help
cargo fmt
```

日常改矩形逻辑时先跑 `cargo test --test rect` 就够快；需要确认 Sketch 导出兼容性时再跑
`cargo test --test sketchtool -- --nocapture`。

这个 crate 没有运行时依赖。公开 API 统一从 crate 根导出，所以使用方只需要写：

```rust
use smooth_frame::{Point, SmoothFrame, SmoothRect};
```

源码按输入层、处理层、输出层拆分在 `src/` 下：

- `lib.rs`：crate 入口和公开类型 re-export。
- `input/`：公开输入入口，负责参数校验、输入清洗和格式归一化。
- `input/corner.rs`：`SmoothCorner` 输入封装，校验单角方向、半径和平滑系数。
- `input/rect/`：`SmoothRect` 矩形便捷输入，处理尺寸归一化、矩形角适配和路径拼接。
- `input/frame/`：`SmoothFrame` 闭合凸多边形输入，负责点集校验、角点适配和路径拼接。
- `process/`：核心处理层，定义 crate 私有 `Processor` trait，只包含 smooth corner 纯几何计算。
- `process/corner/`：单个 smooth corner 的几何解析、圆弧坐标和 cubic 生成。
- `output/format.rs`：公开 `PathFormatter` 扩展契约和内置 `SvgPathFormat`。
- `output/path.rs`：`SmoothPath`、`PathCommand`、`CubicSegment` 和 formatter 委托入口。
- `types/geometry.rs`：`Point`、`Vector` 和基础几何运算。
- `errors/mod.rs`：`SmoothError`。
- `utils/mod.rs`：内部数值工具、格式化工具和容差常量。

测试按领域拆分在 `tests/` 下：

- `rect.rs`：矩形 smooth corner 结构和 Sketch-like 退化行为。
- `corner.rs`：单角 primitive 的几何公式。
- `frame.rs`：凸多边形 frame 和错误输入。
- `path.rs`：路径输出格式。
- `sketchtool.rs`：SketchTool 集成对齐矩阵。
- `support/`：测试断言和 SketchTool 解析辅助。

示例 demo 拆分在 `examples/demo/` 下：

- `main.rs`：串联参数解析、路径生成和输出。
- `cli.rs`：命令行参数解析与输入校验。
- `svg.rs`：完整 SVG 渲染和数字格式化。

## 许可证

Apache-2.0
