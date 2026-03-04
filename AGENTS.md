# AGENTS.md

此文件为 Claude Code (claude.ai/code) 在此代码库中工作时提供指导。

## 项目概述

RawLib 是一个用 Rust 构建的高性能 RAW 图像缩略图提取器。它使用 LibRaw 库从相机 RAW 文件中提取嵌入的缩略图，支持批量处理、递归目录扫描和多种输出选项。

## 架构设计

### 核心组件

- **`src/lib.rs`** - 库入口点和公共 API 导出
- **`src/main.rs`** - CLI 应用程序入口点和命令行解析
- **`src/raw_processor.rs`** - LibRaw C++ 库的高级安全封装
- **`src/ffi.rs`** - LibRaw 的底层 C FFI 绑定
- **`src/processor.rs`** - CLI 批处理逻辑和文件处理（支持多线程并行）
- **`src/parallel.rs`** - 库 API 并行处理模块，供其他应用集成使用
- **`src/exif.rs`** - EXIF 元数据提取模块（库和 CLI 共用）
- **`src/cli.rs`** - CLI 配置、文件收集和统计
- **`src/error.rs`** - 错误类型定义
- **`src/utils.rs`** - 工具函数
- **`build.rs`** - 用于链接 LibRaw 库的构建脚本

### LibRaw 集成

项目包含针对不同平台的预编译 LibRaw 库：
- `libraw/msvc/` - Windows MSVC 库和头文件
- `libraw/gnu/` - Windows MinGW/Unix-like 库和头文件

构建脚本自动检测目标平台并按以下优先级链接库：
1. **Linux/Mac**: 优先使用系统 libraw 动态库（如果存在），否则使用 bundled 静态库
2. **Windows GNU**: 使用 bundled 静态库（需要用 -fPIC 编译）
3. **Windows MSVC**: 使用 bundled 静态库

**注意**: GNU 版本的静态库需要用 `-fPIC` 选项编译，否则会在 x86_64 系统上遇到链接错误。

## 常用开发命令

### 构建项目

```bash
# 开发构建
cargo build

# 发布构建（优化版本）
cargo build --release

# 构建示例
cargo build --examples
```

### 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_name

# 运行测试并显示输出
cargo test -- --nocapture
```

### 运行程序

```bash
# 运行 CLI 工具
cargo run -- --help

# 带参数运行
cargo run -- photo.cr2 -o thumb.jpg

# 运行示例
cargo run --example usage
```

### 开发工具

```bash
# 检查代码
cargo check

# 格式化代码
cargo fmt

# 运行 clippy 检查
cargo clippy

# 生成文档
cargo doc --open
```

## 库 API

### 主要函数

- **`extract_thumbnail(path)`** - 简单提取，返回 `Vec<u8>`
- **`extract_thumbnail_with_info(path)`** - 提取并返回元数据的 `ThumbnailData`
- **`parallel::process_files_parallel(files)`** - 并行处理多个文件
- **`parallel::ParallelProcessor::process_files(files, config)`** - 带配置的并行处理
- **`parallel::ParallelProcessor::process_with_stats(files, config)`** - 并行处理并返回统计信息
- **`exif::extract_exif(path)`** - 提取 EXIF 元数据
- **`exif::extract_exif_parallel(paths, jobs)`** - 并行提取多个文件的 EXIF

### 核心类型

- **`RawProcessor`** - 高级用法的主要处理器结构体
- **`ThumbnailData`** - 包含格式、尺寸和原始图像数据
- **`ImageFormat`** - 支持的图像格式枚举（JPEG、Bitmap）
- **`RawError`** - LibRaw 操作的错误类型
- **`parallel::ParallelConfig`** - 并行处理配置（线程数等）
- **`parallel::ProcessResult`** - 单个文件处理结果
- **`parallel::ProcessingStats`** - 批处理统计信息
- **`exif::ExifData`** - EXIF 元数据结构
- **`exif::ExifError`** - EXIF 提取错误类型

## 支持的 RAW 格式

工具支持 100+ 种相机 RAW 格式，包括：
- 佳能：CR2, CR3
- 尼康：NEF, NRW
- 索尼：ARW, SRF, SR2
- 富士：RAF
- 奥林巴斯：ORF
- 松下：RW2
- Adobe：DNG

## 测试说明

- `src/main.rs` 中的主要测试需要真实 RAW 文件在硬编码路径
- 更新测试中的路径以匹配你本地的 RAW 文件位置
- 如果文件不存在，测试将被跳过

## 关键实现细节

### 并行处理
- 使用 Rayon 库实现数据并行处理
- 自动检测 CPU 核心数，默认使用全部核心
- 支持通过 `-j` 参数自定义线程数
- 单文件处理时自动回退到串行模式
- 使用 MultiProgress 支持多线程进度条显示

### 内存安全
- 使用 RAII 模式自动清理资源
- 尽可能使用零拷贝设计
- 线程安全实现（Send trait）

### 平台支持
- 跨平台支持（Windows、Linux、macOS）
- 基于目标平台自动链接库
- 静态链接用于分发

### 错误处理
- 包含详细信息的综合错误类型
- 从 LibRaw C++ 库正确传播错误
- 常见问题的用户友好错误消息

## 性能特性

- 使用 LTO 和 codegen-units=1 优化速度
- 最小化外部依赖
- 高效的批处理和进度跟踪
- 大文件的内存优化处理