# AGENTS.md

此文件为 AI 编码助手（Claude Code 等）在此代码库中工作时提供指导。

## 项目概述

RawLib 是一个用 Rust 构建的高性能 RAW 图像处理工具。它基于 LibRaw C++ 库，提供两大核心能力：

1. **缩略图提取** — 从相机 RAW 文件中提取内嵌 JPEG/位图缩略图，无需完整解码
2. **EXIF 元数据提取** — 读取拍摄参数（相机型号、快门、光圈、ISO、焦距、GPS 坐标等）

支持批量处理、递归目录扫描、多线程并行、进度条显示和多种输出选项。同时提供可集成的 Rust 库 API（`rawlib` crate）。

## 项目结构

```
rawlib/
├── src/
│   ├── lib.rs           # 库入口，公共 API 导出
│   ├── main.rs          # CLI 二进制入口（clap 参数解析 + EXIF 模式分发）
│   ├── ffi.rs           # LibRaw C API 的 unsafe FFI 绑定（extern "C" 块 + 常量）
│   ├── raw_processor.rs # LibRaw 的高级安全封装（RawProcessor、ThumbnailData、ImageFormat）
│   ├── parallel.rs      # 库级并行处理（ParallelProcessor、ProcessResult、ProcessingStats）
│   ├── exif.rs          # EXIF 元数据提取（ExifData、ExifError、GpsInfo、并行提取）
│   ├── cli.rs           # CLI 配置解析、文件收集、输出策略、ProcessStats 统计
│   ├── processor.rs     # CLI 批处理器（串行/并行调度、覆盖策略、进度条）
│   ├── error.rs         # 应用级错误类型（Error 枚举 + Result 别名）
│   └── utils.rs         # 工具函数（find_available_filename 重命名等）
├── examples/
│   ├── usage.rs              # 库 API 基本用法示例
│   └── parallel_processing.rs # 并行处理示例
├── libraw/
│   ├── msvc/      # Windows MSVC 预编译静态库 + 头文件
│   └── gnu/       # Windows MinGW / Unix 预编译静态库 + 头文件
├── build.rs       # 平台检测 + LibRaw 链接配置
└── Cargo.toml     # 项目元数据与依赖声明
```

### 依赖说明

| 依赖 | 用途 |
|------|------|
| `libc` | FFI 基础类型（c_int、c_char 等） |
| `clap` 4.5 (derive) | CLI 参数解析 |
| `indicatif` | 终端进度条 |
| `walkdir` | 递归目录遍历 |
| `thiserror` | 错误类型派生宏 |
| `log` + `env_logger` | 日志系统（通过 `RUST_LOG` 环境变量控制） |
| `rayon` | 数据并行（多线程处理） |
| `num_cpus` | 自动检测 CPU 核心数 |
| `kamadak-exif`（别名 `exif`） | EXIF 数据解析 |
| `cc`（构建依赖） | C/C++ 编译支持 |
| `tempfile`（开发依赖） | 测试中创建临时目录 |

## LibRaw 集成细节

### 构建脚本（build.rs）

`build.rs` 在编译期自动检测目标平台并配置链接，优先级链为：

**Linux / macOS：**
1. 通过 `pkg-config --exists libraw` 检测系统动态库 → 链接 `libraw.so/dylib`
2. 检查 `/usr/lib64/libraw.so` → 链接系统动态库
3. 回退到 `libraw/gnu/lib/` 打包的静态库（`libraw.a`）
4. 所有路径均需链接 `libstdc++`

**Windows MSVC：**
- 直接使用 `libraw/msvc/lib/libraw_static.lib` 静态链接

**Windows GNU（MinGW）：**
- 使用 `libraw/gnu/lib/libraw.a` 静态链接（需 `-fPIC` 编译）
- 回退到动态链接（`libraw.dll`）
- 额外链接 `libstdc++`

### 关键注意

- GNU 静态库必须用 `-fPIC` 编译，否则 x86_64 平台会出现重定位错误
- `ffi.rs` 使用 `#[link]` 属性指定库名，`build.rs` 提供搜索路径
- 头文件位于 `libraw/{msvc,gnu}/libraw/libraw.h`，修改后需重新构建

## 常用开发命令

### 构建

```bash
cargo build              # 开发构建（未优化）
cargo build --release    # 发布构建（LTO + codegen-units=1）
cargo build --examples   # 构建示例
cargo check              # 快速语法检查（不生成二进制）
```

### 测试

```bash
cargo test                          # 运行全部测试
cargo test test_name                # 运行指定测试
cargo test -- --nocapture           # 显示测试中的 println 输出
cargo test -- --ignored             # 运行被 #[ignore] 标记的测试
```

### 运行

```bash
cargo run -- --help                 # 查看 CLI 帮助
cargo run -- photo.cr2 -o thumb.jpg # 单文件提取
cargo run -- ./photos/ -r --progress # 递归批量处理，显示进度条
cargo run -- photo.nef --exif       # 仅提取 EXIF 信息
cargo run -- photo.nef --exif --json # JSON 格式输出 EXIF
RUST_LOG=debug cargo run -- photo.cr2 -v  # 开启调试日志
cargo run --example usage           # 运行库 API 示例
```

### 代码质量

```bash
cargo fmt               # 格式化代码
cargo fmt -- --check    # 检查格式（不修改）
cargo clippy            # 运行 Clippy 静态检查
cargo doc --open        # 生成并打开 API 文档
```

## CLI 完整参数

### 基本用法

```
rawlib <INPUT> [OPTIONS]
```

| 参数 | 简写 | 说明 | 默认值 |
|------|------|------|--------|
| `INPUT` | — | 输入文件或目录（必填，支持多个） | — |
| `--output <PATH>` | `-o` | 输出文件或目录 | 输入同目录，`.jpg` 扩展名 |
| `--overwrite <POLICY>` | — | 覆盖策略：`skip` / `overwrite` / `rename` | `skip` |
| `--format <FMT>` | `-f` | 输出格式：`auto` / `jpg` / `jpeg` / `bmp` | `auto` |
| `--recursive` | `-r` | 递归扫描子目录 | 否 |
| `--jobs <N>` | `-j` | 并行线程数 | CPU 核心数 |
| `--progress` | — | 显示进度条 | 否 |
| `--verbose` | `-v` | 详细输出 | 否 |
| `--quiet` | `-q` | 静默模式（仅输出错误） | 否 |
| `--exif` | — | EXIF 模式：仅提取元数据，不提取缩略图 | 否 |
| `--json` | — | 配合 `--exif` 以 JSON 格式输出 | 否 |
| `--extensions <LIST>` | — | 自定义 RAW 扩展名（逗号分隔） | `cr2,cr3,nef,nrw,arw,srf,sr2,raf,orf,rw2,dng` |

### 使用示例

```bash
rawlib photo.NEF                        # 提取到 photo.jpg
rawlib photo.NEF -o thumb.jpg           # 指定输出文件名
rawlib ./photos/ -o ./thumbs/           # 批量处理到指定目录
rawlib ./photos/ -r --progress          # 递归扫描 + 进度条
rawlib ./photos/ --overwrite rename     # 遇到重名文件自动重命名
rawlib photo.cr2 --exif                 # 仅查看 EXIF 信息
rawlib photo.cr2 --exif --json          # JSON 格式输出 EXIF
rawlib ./photos/ --exif -r --json       # 递归提取 EXIF，JSON 输出
RUST_LOG=debug rawlib photo.nef -v      # 调试模式
```

## 库 API 参考

### 模块与公开导出

`src/lib.rs` 公开导出的符号：

```rust
// raw_processor 模块
pub use raw_processor::{RawProcessor, ThumbnailData, ImageFormat, RawError};

// parallel 模块
pub use parallel::{ParallelProcessor, ProcessResult, ParallelConfig};
// 注意：ProcessingStats 未在顶层重导出，需通过 rawlib::parallel::ProcessingStats 访问

// exif 模块
pub use exif::{ExifData, ExifError, extract_exif, extract_exif_parallel};

// 便捷函数（顶层）
pub fn extract_thumbnail(path) -> Result<Vec<u8>, RawError>;
pub fn extract_thumbnail_with_info(path) -> Result<ThumbnailData, RawError>;
```

### 核心类型详解

#### `RawProcessor` — 主处理器

```rust
impl RawProcessor {
    pub fn new() -> Result<Self>;                                // 创建实例
    pub fn open_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()>;
    pub fn unpack_thumb(&mut self) -> Result<()>;                // 解包缩略图
    pub fn get_thumbnail(&self) -> Result<ThumbnailData>;        // 获取缩略图数据
    pub fn extract_thumbnail<P: AsRef<Path>>(path: P) -> Result<ThumbnailData>; // 一键提取
    pub fn unpack(&mut self) -> Result<()>;                      // 解包完整 RAW
    pub fn dcraw_process(&mut self) -> Result<()>;               // 去马赛克等处理
    pub fn recycle(&mut self);                                   // 回收缓冲区
    pub fn version() -> String;                                  // 版本字符串
    pub fn version_number() -> i32;                              // 整数版本号
}
```

#### `ThumbnailData` — 缩略图数据

```rust
pub struct ThumbnailData {
    pub format: ImageFormat,  // JPEG / Bitmap / Unknown(i32)
    pub width: u16,
    pub height: u16,
    pub colors: u16,
    pub bits: u16,
    pub data: Vec<u8>,        // 原始图像字节
}
```

#### `ImageFormat` — 图像格式

```rust
pub enum ImageFormat {
    Jpeg,
    Bitmap,
    Unknown(i32),
}

impl ImageFormat {
    pub fn mime_type(&self) -> &'static str;  // "image/jpeg" / "image/bmp" / "application/octet-stream"
}
```

#### `ParallelProcessor` — 并行处理器

```rust
impl ParallelProcessor {
    pub fn process_files<P>(files: &[P], config: &ParallelConfig) -> Vec<ProcessResult>;
    pub fn process_with_stats<P>(files: &[P], config: &ParallelConfig) -> (Vec<ProcessResult>, ProcessingStats);
    pub fn process_single<P: AsRef<Path>>(path: P) -> ProcessResult;  // 单文件便捷方法
}
```

#### `ProcessResult` — 单文件处理结果

```rust
pub struct ProcessResult {
    pub path: PathBuf,
    pub thumbnail: Result<ThumbnailData, RawError>,
    pub elapsed: Duration,
    pub input_size: u64,
}

impl ProcessResult {
    pub fn is_success(&self) -> bool;
    pub fn is_error(&self) -> bool;
    pub fn thumbnail(&self) -> Option<&ThumbnailData>;
    pub fn error(&self) -> Option<&RawError>;
}
```

#### `ProcessingStats` — 批处理统计

```rust
pub struct ProcessingStats {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
    pub total_elapsed: Duration,
    pub total_input_bytes: u64,
    pub total_output_bytes: u64,
}

impl ProcessingStats {
    pub fn files_per_second(&self) -> f64;     // 吞吐率（文件/秒）
    pub fn ms_per_file(&self) -> f64;          // 平均耗时（毫秒/文件）
    pub fn compression_ratio(&self) -> f64;    // 压缩率百分比
}
```

#### `ParallelConfig` — 并行配置

```rust
pub struct ParallelConfig {
    pub jobs: Option<usize>,  // None = 自动检测 CPU 核心数
    pub verbose: bool,
}
```

### EXIF API

#### `extract_exif(path)` — 单文件提取

```rust
pub fn extract_exif<P: AsRef<Path>>(path: P) -> Result<ExifData, ExifError>;
```

#### `extract_exif_parallel(paths, jobs)` — 并行提取

```rust
pub fn extract_exif_parallel<P: AsRef<Path> + Send + Sync>(
    paths: &[P],
    jobs: Option<usize>,
) -> Vec<(PathBuf, Result<ExifData, ExifError>)>;
```

#### `ExifData` — EXIF 数据

```rust
pub struct ExifData {
    pub make: Option<String>,              // 相机制造商
    pub model: Option<String>,             // 相机型号
    pub lens_model: Option<String>,        // 镜头型号
    pub date_time_original: Option<String>,// 拍摄时间
    pub exposure_time: Option<String>,     // 快门速度
    pub f_number: Option<String>,          // 光圈值
    pub iso: Option<u32>,                  // ISO 感光度
    pub focal_length: Option<String>,      // 焦距
    pub image_width: Option<u32>,          // 图像宽度
    pub image_height: Option<u32>,         // 图像高度
    pub orientation: Option<u16>,          // 方向
    pub gps_latitude: Option<(f64, f64, f64)>,  // GPS 纬度（度,分,秒）
    pub gps_longitude: Option<(f64, f64, f64)>, // GPS 经度（度,分,秒）
    pub gps_altitude: Option<f64>,         // GPS 海拔
    pub raw_fields: HashMap<String, String>,// 所有原始 EXIF 字段
}

impl ExifData {
    pub fn summary(&self) -> String;                     // 格式化的中文摘要
    pub fn has_gps(&self) -> bool;                       // 是否含 GPS 数据
    pub fn gps_coordinates(&self) -> Option<(f64, f64)>; // (纬度, 经度) 十进制格式
}
```

#### `GpsInfo` — GPS 信息（内部使用）

```rust
pub struct GpsInfo {
    pub latitude: Option<(f64, f64, f64)>,
    pub longitude: Option<(f64, f64, f64)>,
    pub altitude: Option<f64>,
}
```

### 应用级错误类型（`src/error.rs`）

```rust
pub enum Error {
    LibRaw(rawlib::RawError),                    // LibRaw 处理错误
    Io(std::io::Error),                          // IO 错误
    FileNotFound { path: PathBuf },              // 文件不存在
    InvalidExtension { path: PathBuf },          // 无效扩展名
    NoRawFiles,                                  // 未找到 RAW 文件
    InvalidOverwritePolicy { policy: String },   // 无效覆盖策略
    InvalidOutputFormat { format: String },      // 无效输出格式
    CannotCreateOutput { path: PathBuf, attempts: u32 }, // 无法创建输出
    Config { message: String },                  // 配置错误
}
```

## 支持的 RAW 格式

工具支持 100+ 种相机 RAW 格式，包括：

| 品牌 | 格式 |
|------|------|
| Canon 佳能 | CR2, CR3 |
| Nikon 尼康 | NEF, NRW |
| Sony 索尼 | ARW, SRF, SR2 |
| Fujifilm 富士 | RAF |
| Olympus 奥林巴斯 | ORF |
| Panasonic 松下 | RW2 |
| Adobe | DNG |

可通过 `--extensions` 参数自定义扩展名过滤列表。

## 关键实现细节

### 并行处理
- 使用 Rayon 库实现数据并行（`par_iter`）
- 自动检测 CPU 核心数，默认使用全部核心
- 支持通过 `-j` 参数自定义线程数
- 单文件或 `-j 1` 时自动回退到串行模式
- 批量模式使用 `MultiProgress` 支持多线程进度条显示
- EXIF 提取同样支持并行（`extract_exif_parallel`）

### 内存安全
- `RawProcessor` 使用 RAII 模式（`Drop` 中调用 `libraw_close`）
- FFI 层：从 LibRaw 分配的内存立即拷贝到 Rust `Vec<u8>` 后释放
- `RawProcessor` 实现 `Send` trait，支持跨线程传递
- 零拷贝设计：缩略图数据一次性读取并返回

### 平台支持
- 跨平台支持（Windows、Linux、macOS）
- Windows：使用宽字符 API（`libraw_open_wfile`）支持 Unicode 文件名
- Unix：使用 UTF-8 CString
- 构建脚本自动检测目标平台并选择正确的链接方式
- 静态链接打包库，支持单文件分发

### 错误处理
- 三层错误体系：
  1. `ffi.rs` — LibRaw C 错误码常量（`LIBRAW_SUCCESS`、`LIBRAW_FILE_UNSUPPORTED` 等）
  2. `raw_processor.rs` — `RawError`（code + message），封装 LibRaw 错误
  3. `error.rs` — 应用级 `Error` 枚举，包含 IO、配置、文件等错误
- 使用 `thiserror` 派生 `Display` 和 `Error` trait
- 所有 FFI 返回值均检查错误码

### EXIF 提取
- 基于 `kamadak-exif`（`exif` crate）解析 EXIF 数据
- 支持从 RAW 文件容器中直接读取 EXIF
- 提取常用字段：相机/镜头型号、拍摄参数、GPS 坐标
- GPS 坐标支持度分秒 → 十进制转换
- 支持单文件和批量并行提取

### CLI 架构
- 使用 `clap` 4.5 derive 模式定义参数
- 两种运行模式：缩略图提取（默认）和 EXIF 查看（`--exif`）
- `cli.rs` 负责配置解析、文件发现、输出策略确定
- `processor.rs` 负责实际批处理执行（`BatchProcessor`）
- `cli.rs::ProcessStats` — CLI 层统计（含中文化格式化输出）
- `parallel.rs::ProcessingStats` — 库层统计（纯数据）

## 性能特性

- 发布构建启用 `lto = true` 和 `codegen-units = 1` 优化
- 缩略图提取仅解包嵌入的 JPEG，无需完整解码 RAW
- 最小化外部依赖，启动快速
- 批量处理自动选择并行策略
- 大文件的内存优化处理（只读取必要的缩略图数据）

## 测试说明

- `src/main.rs` 中的 `img` 测试需要真实 RAW 文件在硬编码路径（`C:\Users\huang\...`）
- 如果文件不存在，测试会被跳过（`return` 而非 `panic`）
- `src/utils.rs` 和 `src/cli.rs` 中的单元测试使用 `tempfile` 创建临时目录
- `src/parallel.rs` 和 `src/exif.rs` 包含不依赖外部文件的单元测试
- 运行全部测试：`cargo test`
- 贡献代码前请确保 `cargo test`、`cargo fmt --check` 和 `cargo clippy` 均通过

## 已知问题

- `ffi.rs` 中常量 `LIBRAW_OPIONS_NO_MEMERR_CALLBACK` 和 `LIBRAW_OPIONS_NO_DATAERR_CALLBACK` 的拼写为 `OPIONS`（少了一个 T），与 LibRaw 官方头文件中的 `LIBRAW_OPTIONS_*` 不一致。这是历史遗留问题，如果这些常量未被实际使用，建议修复或移除
