# RawLib - RAW 图像缩略图提取工具

一个快速、高效的命令行工具，用于从相机 RAW 文件中提取内嵌缩略图。支持批量处理、递归目录扫描和多种输出选项。

基于 [LibRaw](https://www.libraw.org/) 库，使用 Rust 开发，具有出色的性能和内存安全性。

## ✨ 特性

- 🚀 **高性能** - 快速提取内嵌缩略图，无需完整解码 RAW 文件
- 📁 **批量处理** - 一次处理整个目录的 RAW 文件
- 🔄 **递归扫描** - 自动扫描子目录中的所有 RAW 文件
- 📊 **进度显示** - 实时显示批量处理进度
- 🎯 **智能命名** - 自动处理文件名冲突（跳过/覆盖/重命名）
- 🌏 **中文支持** - 完美支持中文路径和文件名（Windows）
- 📦 **单文件分发** - 无需安装依赖，直接运行

## 📷 支持的 RAW 格式

支持几乎所有主流相机的 RAW 格式，包括：

| 品牌 | 格式 |
|------|------|
| Canon 佳能 | CR2, CR3 |
| Nikon 尼康 | NEF, NRW |
| Sony 索尼 | ARW, SRF, SR2 |
| Fujifilm 富士 | RAF |
| Olympus 奥林巴斯 | ORF |
| Panasonic 松下 | RW2 |
| Adobe | DNG |
| 更多... | 100+ 种格式 |

## 🚀 快速开始

### 安装

#### 方式 1: 下载预编译版本（推荐）

从 [Releases](https://github.com/yourusername/rawlib/releases) 页面下载最新版本的 `rawlib.exe`，解压后即可使用。

#### 方式 2: 从源码编译

需要安装 [Rust](https://www.rust-lang.org/)：

```bash
git clone https://github.com/yourusername/rawlib.git
cd rawlib
cargo build --release
```

编译完成后，可执行文件位于 `target/release/rawlib.exe`

### 基本用法

```bash
# 提取单个文件的缩略图
rawlib photo.NEF

# 指定输出文件名
rawlib photo.NEF -o thumbnail.jpg

# 批量处理整个目录
rawlib ./photos/ -o ./thumbnails/

# 递归处理所有子目录
rawlib ./photos/ -r --progress

# 显示详细信息
rawlib photo.NEF -v
```

## 📖 使用指南

### 命令行选项

```
用法: rawlib [选项] <输入文件或目录>...

参数:
  <INPUT>...                输入的 RAW 文件或目录

选项:
  -o, --output <路径>       输出文件或目录（默认：与输入同名但扩展名为 .jpg）
      --overwrite <策略>    文件覆盖策略 [可选值: skip, overwrite, rename]
  -r, --recursive           递归扫描子目录
  -v, --verbose             显示详细信息
  -q, --quiet               静默模式，仅显示错误
      --progress            显示进度条
  -f, --format <格式>       输出格式 [可选值: auto, jpg, jpeg, bmp]
      --extensions <扩展名>  指定 RAW 文件扩展名（逗号分隔）
  -j, --jobs <N>            并行工作线程数（默认：CPU核心数）
  -h, --help                显示帮助信息
  -V, --version             显示版本信息
```

### 使用示例

#### 1. 提取单个文件

```bash
# 提取到同目录，自动命名为 photo.jpg
rawlib photo.NEF

# 指定输出文件名
rawlib photo.NEF -o thumb.jpg

# 显示缩略图详细信息
rawlib photo.NEF -v
```

**输出示例**：
```
[INFO] 开始处理: photo.NEF
缩略图格式: JPEG
尺寸: 1920x1280
大小: 456789 字节
✓ 已保存: photo.jpg
```

#### 2. 批量处理目录

```bash
# 处理当前目录所有 RAW 文件，输出到 thumbnails 文件夹
rawlib ./ -o ./thumbnails/

# 递归处理所有子目录，显示进度
rawlib ./photos/ -r --progress -o ./thumbs/

# 处理指定格式的文件
rawlib ./photos/ --extensions nef,cr2

# 使用多线程并行处理（提升大批量处理速度）
rawlib ./photos/ -r --progress -j 8

# 限制为单线程（用于调试或资源受限环境）
rawlib ./photos/ -j 1
```

**进度显示示例**：
```
提取缩略图: [████████████████████] 158/158 (100%)
✓ 成功: 158 个文件
⊘ 跳过: 0 个文件
✗ 失败: 0 个文件
```

#### 3. 文件冲突处理

```bash
# 跳过已存在的文件（默认）
rawlib ./photos/ --overwrite skip

# 覆盖已存在的文件
rawlib ./photos/ --overwrite overwrite

# 自动重命名（photo.jpg → photo_1.jpg → photo_2.jpg）
rawlib ./photos/ --overwrite rename
```

#### 4. 中文路径支持

```bash
# 完美支持中文路径和文件名
rawlib "C:\Users\张三\图片\2024\旅行照片" -o "C:\Users\张三\桌面\缩略图"
```

#### 5. 调试模式

```bash
# 显示详细日志信息
set RUST_LOG=debug
rawlib photo.NEF -v

# 或使用一行命令（PowerShell）
$env:RUST_LOG="debug"; rawlib photo.NEF -v
```

## 🎯 典型工作流程

### 摄影师批量预览工作流

```bash
# 1. 从相机导入 RAW 文件到目录
# 例如: D:\Photos\2024-12-09-活动\

# 2. 快速提取所有缩略图用于预览
rawlib "D:\Photos\2024-12-09-活动\" -r --progress -o "D:\Thumbnails\2024-12-09"

# 3. 在文件管理器中快速浏览缩略图
# 4. 根据缩略图选择需要精修的照片
```

### 照片库管理工作流

```bash
# 为整个照片库生成缩略图索引
rawlib "E:\PhotoLibrary\" -r --progress --overwrite skip

# 定期更新（仅处理新增照片）
rawlib "E:\PhotoLibrary\" -r --overwrite skip
```

## 🔧 高级功能

### 自定义输出格式

```bash
# 输出为 BMP 格式（如果 RAW 文件中包含 BMP 缩略图）
rawlib photo.NEF -f bmp -o thumb.bmp

# 自动检测格式（默认）
rawlib photo.NEF -f auto
```

### 批量处理脚本示例

**Windows 批处理脚本** (`extract_all.bat`):
```batch
@echo off
echo 正在提取 RAW 文件缩略图...
rawlib "D:\Photos\2024\" -r --progress -o "D:\Thumbnails\" --overwrite skip
echo 完成！
pause
```

**PowerShell 脚本** (`extract_all.ps1`):
```powershell
Write-Host "开始提取缩略图..." -ForegroundColor Green
& rawlib "D:\Photos\2024\" -r --progress -o "D:\Thumbnails\" --overwrite skip
Write-Host "提取完成！" -ForegroundColor Green
```

## 📊 性能参考

在 AMD Ryzen 7 8745H + NVMe SSD 上的测试结果：

| 任务 | 文件数 | 总大小 | 耗时 | 速度 | 线程数 |
|------|--------|--------|------|------|--------|
| 单文件提取 | 1 | 25 MB | 0.1 秒 | - | 1 |
| 批量提取（串行） | 100 | 2.5 GB | 8 秒 | 12.5 文件/秒 | 1 |
| 批量提取（并行） | 100 | 2.5 GB | 2 秒 | 50 文件/秒 | 8 |
| 大批量提取（并行） | 1000 | 25 GB | 20 秒 | 50 文件/秒 | 8 |

**注意**：实际性能取决于硬盘速度、CPU 核心数、文件大小和 RAW 格式。并行处理在 SSD 和多核 CPU 上效果最佳。

## ❓ 常见问题

### Q: 提取的缩略图质量如何？

A: RawLib 提取的是相机内嵌的原始缩略图（通常为 JPEG 格式），质量与相机生成的缩略图相同。大多数相机会嵌入高质量的缩略图（1920x1280 或更高分辨率）。

### Q: 为什么比 Lightroom/Capture One 快这么多？

A: RawLib 只提取内嵌缩略图，不进行 RAW 解码、色彩管理或渲染。这使得速度提升了 10-100 倍，但功能仅限于快速预览。

### Q: 支持编辑 RAW 文件吗？

A: 不支持。RawLib 是只读工具，专注于快速提取缩略图。如需编辑，请使用 Lightroom、Capture One、Darktable 等专业软件。

### Q: 所有 RAW 文件都包含缩略图吗？

A: 绝大多数相机会在 RAW 文件中嵌入缩略图。极少数情况下，如果文件损坏或格式特殊，可能无法提取。

### Q: 可以提取全尺寸 JPEG 预览吗？

A: 部分相机（如 Nikon）会嵌入全尺寸 JPEG 预览。RawLib 提取的是文件中最大的可用缩略图，通常就是全尺寸预览。

### Q: 为什么显示"错误 -100009"？

A: 这通常是文件路径或文件损坏导致的。请确保：
- 文件路径正确且文件存在
- 文件未损坏（可以用相机软件打开）
- 具有读取权限

### Q: Windows 上中文路径无法识别怎么办？

A: 版本 0.2.0+ 已完全支持中文路径。如果遇到问题，请更新到最新版本。

## 🛠 故障排除

### 问题：无法找到文件

```bash
# 检查路径是否正确
rawlib "完整路径\photo.NEF" -v

# 使用引号包裹含空格的路径
rawlib "D:\My Photos\photo.NEF"
```

### 问题：批量处理时部分文件失败

```bash
# 使用详细模式查看具体错误
rawlib ./photos/ -v

# 启用调试日志
set RUST_LOG=debug
rawlib ./photos/ -v
```

### 问题：缺少 DLL 文件

RawLib 使用静态链接，理论上不需要额外 DLL。如果遇到问题：

1. 确保使用的是 Release 版本
2. 下载 [Visual C++ Redistributable](https://aka.ms/vs/17/release/vc_redist.x64.exe)（Windows）
3. 重新下载最新版本的 RawLib

## 📝 开发者文档

### 作为 Rust 库使用

RawLib 也可以作为 Rust 库集成到其他项目中：

```toml
[dependencies]
rawlib = "0.2.0"
```

**简单提取**：
```rust
use rawlib::extract_thumbnail;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let thumb_bytes = extract_thumbnail("photo.NEF")?;
    std::fs::write("thumbnail.jpg", &thumb_bytes)?;
    Ok(())
}
```

**带元数据提取**：
```rust
use rawlib::extract_thumbnail_with_info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let thumb = extract_thumbnail_with_info("photo.NEF")?;
    println!("格式: {:?}", thumb.format);
    println!("尺寸: {}x{}", thumb.width, thumb.height);
    std::fs::write("thumbnail.jpg", &thumb.data)?;
    Ok(())
}
```

**并行批处理**（高性能多线程）：
```rust
use rawlib::parallel::{ParallelProcessor, ParallelConfig};
use std::path::PathBuf;

fn main() {
    let files = vec![
        PathBuf::from("photo1.NEF"),
        PathBuf::from("photo2.CR2"),
        PathBuf::from("photo3.ARW"),
    ];

    // 使用默认配置（自动使用所有 CPU 核心）
    let results = ParallelProcessor::process_files(&files, &ParallelConfig::default());

    // 处理结果
    for result in &results {
        match &result.thumbnail {
            Ok(thumb) => println!("✓ {}: {} bytes", result.path.display(), thumb.data.len()),
            Err(e) => println!("✗ {}: {}", result.path.display(), e),
        }
    }

    // 或者获取详细统计信息
    let (results, stats) = ParallelProcessor::process_with_stats(&files, &ParallelConfig::default());
    println!("处理速度: {:.1} 文件/秒", stats.files_per_second());
    println!("总耗时: {:?}", stats.total_elapsed);
}
```

详细 API 文档请参考 [docs.rs/rawlib](https://docs.rs/rawlib)

### 从源码编译

**前置要求**：
- Rust 1.70+ (`rustup` 安装)
- LibRaw 库（项目已包含）

**编译步骤**：
```bash
# 克隆仓库
git clone https://github.com/yourusername/rawlib.git
cd rawlib

# 开发编译
cargo build

# 发布编译（优化版本）
cargo build --release

# 运行测试
cargo test

# 生成文档
cargo doc --open
```

## 📄 许可证

本项目采用双许可证：

- MIT License
- Apache License 2.0

您可以选择其中任何一个许可证使用本软件。

LibRaw 库采用 LGPL-2.1 或 CDDL-1.0 许可证。

## 🙏 致谢

- [LibRaw](https://www.libraw.org/) - 强大的 RAW 图像处理库
- [Clap](https://github.com/clap-rs/clap) - Rust 命令行解析库
- [Indicatif](https://github.com/console-rs/indicatif) - 终端进度条库

## 📮 反馈与支持

- **问题反馈**: [GitHub Issues](https://github.com/yourusername/rawlib/issues)
- **功能建议**: [GitHub Discussions](https://github.com/yourusername/rawlib/discussions)
- **邮件联系**: your.email@example.com

## 🗺 路线图

- [x] 多线程并行处理
- [ ] 支持批量调整缩略图尺寸
- [ ] 支持输出 WebP 格式
- [ ] 添加 GUI 图形界面
- [ ] macOS 和 Linux 支持
- [ ] 支持从 RAW 文件提取元数据（EXIF）

## 📊 项目结构

```
rawlib/
├── src/
│   ├── main.rs          # CLI 入口点
│   ├── lib.rs           # 库入口（公共 API）
│   ├── cli.rs           # 命令行配置和文件收集
│   ├── processor.rs     # 批处理逻辑
│   ├── raw_processor.rs # LibRaw 封装
│   ├── error.rs         # 错误类型定义
│   ├── utils.rs         # 工具函数
│   └── ffi.rs           # C FFI 绑定
├── libraw/              # LibRaw 库文件
│   ├── lib/             # 静态库
│   └── libraw/          # 头文件
├── examples/            # 使用示例
├── build.rs             # 构建脚本
├── Cargo.toml           # 项目配置
└── README.md            # 本文件
```

---

**版本**: 0.2.0  
**最后更新**: 2024-12-09  
**作者**: [Your Name]

如果觉得这个项目有用，请给个 ⭐️ Star！
