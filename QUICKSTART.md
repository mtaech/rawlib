# 快速开始指南

## 功能实现完成 ✓

已成功实现从 RAW 文件提取缩略图并返回字节流的功能。

## 核心功能

### 1. 简单提取（推荐）

```rust
use rawlib::extract_thumbnail;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 传入文件路径，返回字节流
    let thumb_bytes = extract_thumbnail("photo.cr2")?;
    
    // 保存到文件
    std::fs::write("thumbnail.jpg", &thumb_bytes)?;
    
    Ok(())
}
```

### 2. 获取详细信息

```rust
use rawlib::extract_thumbnail_with_info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 获取缩略图和元数据
    let thumb = extract_thumbnail_with_info("photo.nef")?;
    
    println!("格式: {:?}", thumb.format);          // Jpeg/Bitmap
    println!("尺寸: {}x{}", thumb.width, thumb.height);
    println!("字节数: {}", thumb.data.len());
    
    // 字节流在 thumb.data 中
    std::fs::write("output.jpg", &thumb.data)?;
    
    Ok(())
}
```

### 3. 使用处理器（高级）

```rust
use rawlib::RawProcessor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut processor = RawProcessor::new()?;
    processor.open_file("image.arw")?;
    processor.unpack_thumb()?;
    
    let thumb = processor.get_thumbnail()?;
    // thumb.data 是字节流
    
    Ok(())
}
```

## API 参考

### 便捷函数

- **`extract_thumbnail(path)`** - 提取缩略图，返回 `Vec<u8>` 字节流
- **`extract_thumbnail_with_info(path)`** - 提取缩略图，返回 `ThumbnailData` 结构

### ThumbnailData 结构

```rust
pub struct ThumbnailData {
    pub format: ImageFormat,    // 图片格式
    pub width: u16,             // 宽度
    pub height: u16,            // 高度
    pub colors: u16,            // 颜色通道数
    pub bits: u16,              // 位深度
    pub data: Vec<u8>,          // 字节流数据 ★
}
```

### ImageFormat 枚举

```rust
pub enum ImageFormat {
    Jpeg,           // JPEG 格式
    Bitmap,         // 位图格式
    Unknown(i32),   // 未知格式
}
```

方法：
- `mime_type()` - 返回 MIME 类型字符串

## 支持的 RAW 格式

- Canon: CR2, CR3
- Nikon: NEF, NRW
- Sony: ARW, SRF, SR2
- Fujifilm: RAF
- Olympus: ORF
- Panasonic: RW2
- Adobe: DNG
- 以及更多...

## 命令行使用

```bash
# 构建项目
cargo build --release

# 提取缩略图
cargo run --release -- photo.cr2

# 指定输出文件
cargo run --release -- image.nef output.jpg
```

## 错误处理

```rust
match extract_thumbnail("photo.cr2") {
    Ok(bytes) => {
        println!("成功提取 {} 字节", bytes.len());
        std::fs::write("thumb.jpg", &bytes)?;
    }
    Err(e) => {
        eprintln!("提取失败: {}", e);
    }
}
```

## 常见错误

- `LIBRAW_FILE_UNSUPPORTED` - 不支持的文件格式
- `LIBRAW_NO_THUMBNAIL` - 文件中没有缩略图
- `LIBRAW_IO_ERROR` - 文件读取错误

## 性能特点

- ✓ 零拷贝设计
- ✓ 内存安全（RAII）
- ✓ 线程安全（Send trait）
- ✓ 自动资源清理
- ✓ 最小依赖（仅 libc）

## 完整示例

```rust
use rawlib::extract_thumbnail;
use std::fs;

fn extract_and_save(raw_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 一行代码提取缩略图
    let thumbnail_bytes = extract_thumbnail(raw_path)?;
    
    // 保存字节流到文件
    fs::write(output_path, &thumbnail_bytes)?;
    
    println!("成功: {} -> {} ({} 字节)", 
             raw_path, output_path, thumbnail_bytes.len());
    
    Ok(())
}

fn main() {
    if let Err(e) = extract_and_save("photo.cr2", "thumb.jpg") {
        eprintln!("错误: {}", e);
    }
}
```

## 批量处理示例

```rust
use rawlib::extract_thumbnail;

fn process_batch(files: &[&str]) {
    for (i, file) in files.iter().enumerate() {
        match extract_thumbnail(file) {
            Ok(bytes) => {
                let output = format!("thumb_{}.jpg", i);
                std::fs::write(&output, &bytes).ok();
                println!("✓ {} -> {}", file, output);
            }
            Err(e) => eprintln!("✗ {}: {}", file, e),
        }
    }
}

fn main() {
    let files = vec!["img1.cr2", "img2.nef", "img3.arw"];
    process_batch(&files);
}
```

## 集成到项目

在 `Cargo.toml` 中添加：

```toml
[dependencies]
rawlib = { path = "../rawlib" }
```

然后使用：

```rust
use rawlib::extract_thumbnail;

let bytes = extract_thumbnail("photo.cr2")?;
// bytes 是 Vec<u8> 类型，可以直接使用
```

## 总结

✓ 核心功能已实现：传入文件地址，提取缩略图，返回字节流
✓ 提供三种使用方式：简单函数、详细信息函数、处理器类
✓ 完整的错误处理和类型安全
✓ 支持所有主流 RAW 格式
✓ 包含命令行工具和示例代码
