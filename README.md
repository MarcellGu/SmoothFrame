# smooth-frame

`smooth-frame` 是一个独立、零依赖的 Rust crate，用来生成 Sketch-like smooth corner / smooth frame 的 cubic Bezier 路径。

它不是只支持矩形的工具。矩形只是便捷封装，底层模型是：

- `SmoothCorner`：任意凸角 primitive
- `SmoothFrame`：闭合凸 polygon / frame
- `SmoothRect`：矩形便捷 API

输出命令可以直接映射到底层图形 API：

- SVG：`M / L / C / Z`
- Canvas：`moveTo / lineTo / bezierCurveTo`
- Skia：`moveTo / lineTo / cubicTo`
- Godot：`Curve2D`，或将 cubic 采样为 polygon

## 安装

```toml
[dependencies]
smooth-frame = "0.1"
```

## 快速开始

```rust
use smooth_frame::SmoothRect;

let path = SmoothRect::new(1000.0, 1000.0)
.with_radius(100.0)
.with_smoothing(0.6)
.to_path();

println!("{}", path.to_svg_path_with_precision(3));
```

## 命令行示例

仓库内置了一个 example 示例，可以直接生成 SVG path：

```bash
cargo run --example demo
```

也可以调整参数，或生成完整 SVG 文件：

```bash
cargo run --example demo -- --width 1000 --height 1000 --radius 250 --smoothing 0.6
cargo run --example demo -- --output smooth.svg
cargo run --example demo -- --output smooth.svg --border 4
```

默认生成 `1000 x 1000`、角半径为 `250` 的 smooth corner SVG。生成 SVG 文件时会按边框宽度向内收缩路径，并将路径平移半个边框宽度，避免
stroke 被裁切。

`SmoothRect` 以本机 `sketchtool` 的矩形输出为准。常规 smooth 区间通常是：

```text
M A,0
L w-A,0
C ...
C ...
C ...
L w,h-A
C ...
C ...
C ...
L A,h
C ...
C ...
C ...
L 0,A
C ...
C ...
C ...
Z
```

也就是 4 条边和 4 个角，每角最多 3 段 cubic，总计最多 12 段 cubic。与 `sketchtool` 一样，矩形便捷 API 会省略零长度边或首尾零长度
cubic；当半径进入近圆或近 capsule 区间时，也会退化为 SketchTool 的普通圆角、圆或 capsule-like 结构。

如果需要始终保留每角 3 段 cubic 的低层 primitive，可以直接使用 `SmoothCorner` 或 `SmoothFrame`。

## 通用角点

```rust
use smooth_frame::{Point, SmoothCorner, Vector};

let cubics = SmoothCorner::new(
Point::new(100.0, 0.0),
Vector::new(- 1.0, 0.0),
Vector::new(0.0, 1.0),
)
.with_radius(24.0)
.with_smoothing(0.6)
.with_limits(80.0, 60.0)
.to_cubic_segments() ?;
# Ok::<_, smooth_frame::SmoothError>(())
```

输入语义：

```text
origin         当前角点
incoming_axis  从角点指向上一条边的方向
outgoing_axis  从角点指向下一条边的方向
radius         核心圆半径
smoothing      Sketch-like smoothing，自动 clamp 到 [0, 1]
incoming_limit incoming 方向最大影响范围
outgoing_limit outgoing 方向最大影响范围
```

约束：

```text
radius >= 0
0 < angle_between(incoming_axis, outgoing_axis) < PI
radius clamp 到当前角可容纳的最大核心半径
```

`radius == 0` 时，frame / rect 输出普通折角路径；`SmoothCorner::to_cubic_segments()` 返回空数组。

## 闭合 frame / polygon

```rust
use smooth_frame::{Point, SmoothFrame};

let path = SmoothFrame::closed([
Point::new(0.0, 0.0),
Point::new(220.0, 30.0),
Point::new(180.0, 170.0),
Point::new(20.0, 140.0),
])
.with_radius(24.0)
.with_smoothing(0.5)
.to_path() ?;
# Ok::<_, smooth_frame::SmoothError>(())
```

v1 支持闭合凸 polygon。凹角目前返回 `SmoothError::ConcaveFrame`，自相交路径返回 `SmoothError::SelfIntersectingFrame`，API
设计保留后续支持 concave corner 的空间。

## 90° 矩形公式

对矩形 90° 角，本库复刻 Sketch-like smooth corner 控制点规律：

```text
r = clamp(radius, 0, min(width, height) / 2)
s = clamp(smoothing, 0, 1)

rawA = (1 + s) * r

Ax = min(rawA, width / 2)
Ay = min(rawA, height / 2)

alpha_x = clamp(Ax / r - 1, 0, 1) * PI / 4
alpha_y = clamp(Ay / r - 1, 0, 1) * PI / 4
```

未饱和时：

```text
A = (1 + smoothing) * radius
```

饱和后：

```text
A = side / 2
effective_smoothing = clamp(A / radius - 1, 0, 1)
```

矩形便捷 API 在近圆或近 capsule 区间继续以 `sketchtool` 为准：当 SketchTool 退化为普通圆角、圆或 capsule-like 结构时，
`SmoothRect` 也做同样退化。

## 单角 cubic 模板

局部坐标中，`u` 沿进入边从角点指向边上的起点，`v` 沿离开边从角点指向边上的终点：

```text
start = (A0, 0)
end   = (0, A1)
```

90° 角使用固定 3 段 cubic：

```text
tan0 = tan(alpha0 / 2)
tangent0 = r - r * tan0
handle0 = (A0 - tangent0) / 3

p1 = (
  r - r * sin(alpha0),
  r - r * cos(alpha0)
)

theta = PI / 2 - alpha0 - alpha1
arcHandle = if theta <= 0 then 0 else (4 / 3) * tan(theta / 4) * r

p2 = (
  r - r * cos(alpha1),
  r - r * sin(alpha1)
)

tan1 = tan(alpha1 / 2)
tangent1 = r - r * tan1
handle1 = (A1 - tangent1) / 3
```

```text
C1:
ctrl1 = (A0 - 2 * handle0, 0)
ctrl2 = (tangent0, 0)
to    = p1

C2:
ctrl1 = (p1.x - arcHandle * cos(alpha0), p1.y + arcHandle * sin(alpha0))
ctrl2 = (p2.x + arcHandle * sin(alpha1), p2.y - arcHandle * cos(alpha1))
to    = p2

C3:
ctrl1 = (0, tangent1)
ctrl2 = (0, tangent1 + handle1)
to    = (0, A1)
```

## 非 90° 泛化

对任意凸角：

```text
phi = angle_between(incoming_axis, outgoing_axis)
0 < phi < PI

base_tangent = r / tan(phi / 2)

rawA = (1 + smoothing) * base_tangent

A0 = min(rawA, incoming_limit)
A1 = min(rawA, outgoing_limit)

alpha0 = clamp(A0 / base_tangent - 1, 0, 1) * phi / 2
alpha1 = clamp(A1 / base_tangent - 1, 0, 1) * phi / 2

middle_arc_angle = phi - alpha0 - alpha1
```

当 `phi = PI / 2` 时，这组公式退化回上面的 Sketch-like 矩形公式。

## 与 Sketch / sketchtool 的关系

矩形便捷 API 的目标是复刻 `sketchtool` 对 Sketch smooth corner 矩形的实际路径输出。

需要特别注意：

- `SmoothRect` 以 `sketchtool` 为准，包括近圆段的普通圆角、圆或 capsule-like fallback。
- `SmoothCorner` / `SmoothFrame` 保留通用 smooth corner primitive，适合非矩形或需要稳定 primitive 的场景。
- 对 `SmoothRect`，本库输出与 `sketchtool` 导出的 SVG path 只应存在浮点级误差。

测试中包含两层约束：

- 单元测试内置 `1000x1000, r=100, smoothing=0.6` 的 Sketch-like 控制点参考。
- 集成测试会在本机存在 `sketchtool` 时，通过 `sketchtool run-script` 在内存中逐个创建 smooth rect，导出 SVG Buffer，立刻删除该
  shape，再继续下一个用例。
- 集成测试覆盖 `1000x1000, smoothing=0.6, radius=0..=500`，多组 `smoothing`，以及多种矩形比例。
- 所有纳入矩阵的用例都会逐条比较 `M/L/C/Z` 控制点，要求只存在浮点级误差；不再为近圆 fallback 放宽结构差异。
- 使用 `-- --nocapture` 运行时，测试会输出一张按尺寸、smoothing、radius 区间和命令结构汇总的对齐结果表。

可通过环境变量控制 SketchTool 路径和强制要求：

```bash
SMOOTH_FRAME_SKETCHTOOL=/Applications/Sketch.app/Contents/MacOS/sketchtool cargo test
SMOOTH_FRAME_REQUIRE_SKETCHTOOL=1 cargo test
```

由于 `sketchtool` SVG 会进行十进制格式化，集成测试使用 `1e-5` 的比较容差。

## 路径输出

```rust
let path = smooth_frame::SmoothRect::new(1000.0, 500.0)
.with_radius(250.0)
.with_smoothing(0.6)
.to_path();

let commands = path.commands();
let cubics = path.cubic_segments();
let svg = path.to_svg_path();
let compact_svg = path.to_svg_path_with_precision(3);
```

`to_svg_path_with_precision` 会裁掉多余尾随零，例如 `1000.000` 会输出为 `1000`。
